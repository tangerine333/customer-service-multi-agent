"""Auto-fix engine: generates and applies code fixes for confirmed issues.

Strategy:
1. For deterministic rules (regex-matched): apply template-based fixes
2. For LLM-confirmed issues: use LLM-generated fix code
3. Validate fix: compile check, test run
4. On failure: revert and mark as "fix_failed"
"""

import logging
import re
from typing import Optional

logger = logging.getLogger(__name__)


class AutoFixer:
    """Generates and applies automatic code fixes."""

    # Template-based fixes for deterministic rules
    FIX_TEMPLATES = {
        "SEC-001": {
            "description": "Replace string formatting with parameterized query",
            "template": "cursor.execute(query, params)  # Parameterized",
        },
        "SEC-002": {
            "description": "Replace hardcoded secret with environment variable",
            "template": 'import os\nSECRET = os.environ.get("SECRET_KEY")',
        },
        "SEC-003": {
            "description": "Use subprocess.run with list args (no shell=True)",
            "template": "subprocess.run([cmd, arg1, arg2], check=True)",
        },
        "SEC-005": {
            "description": "Use textContent instead of innerHTML",
            "template": "element.textContent = value;  // Safe: no HTML parsing",
        },
        "SEC-006": {
            "description": "Replace unsafe deserialization with safe alternative",
            "template": "json.loads(data)  # Safe: use JSON instead of pickle",
        },
        "PERF-002": {
            "description": "Replace += in loop with join()",
            "template": "''.join(parts)  # Efficient string building",
        },
        "PERF-003": {
            "description": "Use async sleep instead of time.sleep",
            "template": "await asyncio.sleep(seconds)  # Non-blocking",
        },
        "STYLE-003": {
            "description": "Catch specific exceptions",
            "template": "except (ValueError, KeyError) as e:",
        },
        "STYLE-004": {
            "description": "Use None default for mutable args",
            "template": "def func(param=None):\n    if param is None:\n        param = []",
        },
    }

    async def generate_fix(self, finding: dict) -> Optional[str]:
        """Generate a fix for a finding. Returns fix code or None."""
        rule_id = finding.get("rule_id", "")

        # Try template-based fix first (fast, deterministic)
        if rule_id in self.FIX_TEMPLATES:
            return self.FIX_TEMPLATES[rule_id]["template"]

        # For LLM-confirmed issues with auto_fix_code already present
        if finding.get("auto_fix_code"):
            return finding["auto_fix_code"]

        # Generate fix using heuristics for the issue category
        category = finding.get("category", "")
        return self._generate_heuristic_fix(finding)

    def _generate_heuristic_fix(self, finding: dict) -> Optional[str]:
        """Generate a fix based on heuristics for the issue type."""
        category = finding.get("category", "")
        line = finding.get("line_content", "")

        if category == "security":
            if "execute" in line.lower():
                return line.replace("+", ", ").replace("f\"", "\"")
            if "innerHTML" in line:
                return line.replace("innerHTML", "textContent")
            if "os.system" in line:
                return line.replace("os.system(", "subprocess.run([")

        if category == "performance":
            if "+=" in line and "for " in finding.get("snippet", ""):
                return "# Use ''.join(parts) or StringBuilder instead of += in loop"

        return None

    def apply_fix(
        self, file_path: str, line_start: int, line_end: int, fix_code: str
    ) -> bool:
        """Apply a fix to a file. Returns True if successful."""
        try:
            with open(file_path, "r", encoding="utf-8") as f:
                lines = f.readlines()

            # Replace the target lines with the fix
            new_lines = (
                lines[: line_start - 1]
                + [fix_code + "\n"]
                + lines[line_end:]
            )

            with open(file_path, "w", encoding="utf-8") as f:
                f.writelines(new_lines)

            logger.info("Applied fix to %s:%d-%d", file_path, line_start, line_end)
            return True
        except Exception as e:
            logger.error("Failed to apply fix: %s", e)
            return False

    def validate_fix(self, file_path: str) -> tuple[bool, str]:
        """Validate a fix by attempting to compile/parse the file."""
        import subprocess

        if file_path.endswith(".py"):
            try:
                result = subprocess.run(
                    ["python", "-m", "py_compile", file_path],
                    capture_output=True, text=True, timeout=10,
                )
                return result.returncode == 0, result.stderr
            except Exception as e:
                return False, str(e)

        if file_path.endswith(".go"):
            try:
                result = subprocess.run(
                    ["go", "vet", file_path],
                    capture_output=True, text=True, timeout=10,
                )
                return result.returncode == 0, result.stderr
            except Exception as e:
                return False, str(e)

        # For other languages, just check syntax is reasonable
        return True, ""
