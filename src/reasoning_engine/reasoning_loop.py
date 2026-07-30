"""ReAct (Reasoning + Acting) loop for code review orchestration.

The core reasoning engine that:
1. Parses the input (diff / source code)
2. Runs deterministic rules (Stage 1 & 2 of the funnel)
3. Invokes LLM for ambiguous cases (Stage 3)
4. Generates fix suggestions and auto-fix patches

Architecture note:
- Symbol extraction & call graph: delegates to Rust engine via rust_bridge
  (when compiled PyO3 lib is available; falls back to Python regex otherwise)
- Rule matching & LLM orchestration: Python native
"""

import logging
import re
import time
from datetime import datetime, timezone
from typing import Optional

from .rules.engine import RuleEngine
from .models.backend import ModelBackend
from .rust_bridge import get_analyzer, is_rust_available
from ..graph_store.postgres import save_review, save_findings, update_review_status
from ..fixer.auto_fixer import AutoFixer

logger = logging.getLogger(__name__)


class ReviewOrchestrator:
    """Orchestrates the full review pipeline: Rules → Context → LLM."""

    def __init__(self):
        self.rule_engine = RuleEngine()
        self.model = ModelBackend()
        self.fixer = AutoFixer()
        self.analyzer = get_analyzer()  # Rust-backed (falls back to Python)
        logger.info("ReviewOrchestrator initialized (rust=%s)", is_rust_available())

    async def run_review(
        self,
        review_id: str,
        diff_content: str,
        language: Optional[str] = None,
        mr_id: Optional[str] = None,
    ):
        """Execute the complete review pipeline."""
        start_time = time.monotonic()
        logger.info("Starting review %s (%d bytes)", review_id, len(diff_content))

        await save_review(
            review_id=review_id,
            mr_id=mr_id,
            status="analyzing",
        )

        try:
            # Parse diff into structured representation
            files = self._parse_diff(diff_content, language)

            # Extract symbols via Rust engine (or Python fallback)
            for file_info in files:
                full_source = "\n".join(file_info.get("added_lines", []))
                if full_source.strip():
                    symbols = self.analyzer.parse_file(file_info["path"], full_source)
                    file_info["symbols"] = symbols
                    logger.debug("File %s: %d symbols extracted", file_info["path"], len(symbols))

            # === Stage 1: Deterministic rule matching ===
            stage1_findings = self.rule_engine.match_deterministic(files)
            logger.info("Stage 1 (deterministic): %d findings", len(stage1_findings))

            # === Stage 2: Context-aware filtering ===
            stage2_findings = self._apply_context_filter(stage1_findings)
            logger.info("Stage 2 (context filter): %d findings retained", len(stage2_findings))

            # === Stage 3: LLM semantic judgment (only for ambiguous cases) ===
            llm_count = 0
            final_findings = []

            for finding in stage2_findings:
                if finding.get("is_deterministic"):
                    # Already confirmed by deterministic rules
                    final_findings.append(finding)
                else:
                    # Ask LLM for final judgment
                    llm_count += 1
                    judgment = await self.model.judge(finding)
                    if judgment.get("is_issue", False):
                        finding["llm_confidence"] = judgment.get("confidence", 0.5)
                        finding["filter_stage"] = "stage3_llm"
                        final_findings.append(finding)

            logger.info("Stage 3 (LLM): %d calls, %d confirmed issues", llm_count, len(final_findings))

            # === Auto-fix generation ===
            for finding in final_findings:
                if finding["severity"] in ("critical", "major"):
                    try:
                        fix = await self.fixer.generate_fix(finding)
                        if fix:
                            finding["auto_fix_code"] = fix
                    except Exception as e:
                        logger.warning("Auto-fix failed for %s: %s", finding.get("title"), e)

            # === Save results ===
            elapsed_ms = int((time.monotonic() - start_time) * 1000)

            severity_counts = {"critical": 0, "major": 0, "minor": 0, "info": 0}
            for f in final_findings:
                sev = f.get("severity", "info")
                severity_counts[sev] = severity_counts.get(sev, 0) + 1

            await update_review_status(
                review_id=review_id,
                status="completed",
                total_issues=len(final_findings),
                critical_count=severity_counts["critical"],
                major_count=severity_counts["major"],
                minor_count=severity_counts["minor"],
                info_count=severity_counts["info"],
                files_analyzed=len(files),
                analysis_time_ms=elapsed_ms,
                llm_calls=llm_count,
                completed_at=datetime.now(timezone.utc),
            )

            await save_findings(review_id=review_id, findings=final_findings)

            logger.info(
                "Review %s completed: %d issues, %dms",
                review_id, len(final_findings), elapsed_ms,
            )

        except Exception as e:
            logger.exception("Review %s failed: %s", review_id, e)
            await update_review_status(review_id=review_id, status="failed")

    def _parse_diff(self, diff_content: str, language: Optional[str] = None) -> list[dict]:
        """Parse unified diff into structured file representations.

        Handles the standard unified diff format:
            diff --git a/path.py b/path.py
            --- a/path.py
            +++ b/path.py
            @@ -1,3 +1,4 @@
             unchanged line
            +added line
            -removed line
        """
        files = []
        current_file = None
        current_additions = []
        current_removals = []

        for line in diff_content.split("\n"):
            # New file section: save previous and reset
            if line.startswith("diff --git "):
                if current_file is not None:
                    files.append(self._make_file_entry(current_file, current_additions,
                                                       current_removals, language))
                current_file = None
                current_additions = []
                current_removals = []

            # Target file path
            elif line.startswith("+++ b/"):
                current_file = line[6:]

            # Added line
            elif line.startswith("+") and not line.startswith("+++"):
                current_additions.append(line[1:])

            # Removed line
            elif line.startswith("-") and not line.startswith("---"):
                current_removals.append(line[1:])

        # Don't forget the last file
        if current_file is not None:
            files.append(self._make_file_entry(current_file, current_additions,
                                               current_removals, language))

        return files

    def _make_file_entry(self, path, additions, removals, language):
        return {
            "path": path,
            "added_lines": additions,
            "removed_lines": removals,
            "language": language or self._guess_language(path),
        }

    def _guess_language(self, filepath: str) -> str:
        ext_map = {
            ".py": "python", ".js": "javascript", ".ts": "typescript",
            ".tsx": "typescript", ".go": "go", ".java": "java",
            ".rs": "rust", ".c": "c", ".cpp": "cpp", ".h": "c",
            ".rb": "ruby", ".php": "php", ".swift": "swift",
            ".kt": "kotlin", ".scala": "scala",
        }
        for ext, lang in ext_map.items():
            if filepath.endswith(ext):
                return lang
        return "unknown"

    def _apply_context_filter(self, findings: list[dict]) -> list[dict]:
        """Stage 2: Context-aware filtering.

        - Skip findings in test files (lower severity)
        - Skip findings with //nolint comments
        - Prioritize findings introduced in current MR
        """
        filtered = []
        for finding in findings:
            file_path = finding.get("file_path", "")

            # Test file exemption: downgrade security rules for test files
            is_test_file = any(marker in file_path for marker in
                ["_test.", "test_", "__tests__", "spec.", ".spec.", "/tests/", "/test/"])

            if is_test_file and finding.get("category") == "security":
                finding["severity"] = "minor"  # Downgrade

            # Check for nolint directives
            line_content = finding.get("line_content", "")
            if "nolint" in line_content or "no-lint" in line_content:
                continue  # Skip developer-annotated exceptions

            filtered.append(finding)

        return filtered
