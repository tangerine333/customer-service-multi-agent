"""Review rule engine: loads and executes deterministic rules.

Implements Stage 1 of the 3-layer funnel:
1. Load rules from database (or built-in defaults)
2. Compile regex/AST patterns
3. Match against code changes
4. Output findings with deterministic confidence
"""

import logging
import re
from typing import Optional

logger = logging.getLogger(__name__)

from .security import SECURITY_RULES
from .performance import PERFORMANCE_RULES
from .style import STYLE_RULES


class RuleEngine:
    """Compiles and executes code review rules."""

    def __init__(self):
        self.rules: list[dict] = []
        self._compile_rules()

    def _compile_rules(self):
        """Compile all built-in rules and their regex patterns."""
        self.rules = SECURITY_RULES + PERFORMANCE_RULES + STYLE_RULES
        for rule in self.rules:
            if pattern := rule.get("pattern"):
                try:
                    rule["_compiled"] = re.compile(pattern, re.IGNORECASE | re.MULTILINE)
                except re.error as e:
                    logger.warning("Invalid regex in rule %s: %s", rule.get("rule_id"), e)
                    rule["_compiled"] = None

    def match_deterministic(self, files: list[dict]) -> list[dict]:
        """Run all deterministic rules against parsed files.

        Returns findings that pass Stage 1 (deterministic pattern matching).

        Note: Rules with multi-line patterns (containing \\n) are matched against
        the full file content, not individual lines. Single-line patterns are
        matched line-by-line for precise location reporting.
        """
        findings = []

        for file_info in files:
            language = file_info.get("language", "unknown")
            file_path = file_info.get("path", "")

            lines = file_info.get("added_lines", [])
            if not lines:
                continue
            code = "\n".join(lines)

            for rule in self.rules:
                if not rule.get("is_enabled", True):
                    continue
                if not rule.get("is_deterministic", True):
                    continue

                rule_lang = rule.get("language", "all")
                if rule_lang != "all" and rule_lang != language:
                    continue

                compiled = rule.get("_compiled")
                if not compiled:
                    continue

                pattern = rule.get("pattern", "")

                # Multi-line patterns: match against full file content
                if "\\n" in pattern or "\n" in pattern:
                    for match in compiled.finditer(code):
                        line_num = code[:match.start()].count("\n") + 1
                        findings.append(self._make_finding(rule, file_path, line_num,
                                                            lines[min(line_num - 1, len(lines) - 1)] if lines else "",
                                                            code, language))
                else:
                    # Single-line patterns: match line by line
                    for i, line in enumerate(lines):
                        if compiled.search(line):
                            findings.append(self._make_finding(rule, file_path, i + 1, line, code, language))

        return findings

    def _make_finding(self, rule, file_path, line_num, line_content, full_code, language):
        return {
            "rule_id": rule["rule_id"],
            "category": rule["category"],
            "severity": rule["severity"],
            "title": f"[{rule['rule_id']}] {rule['name']}",
            "description": rule.get("description", ""),
            "file_path": file_path,
            "line_start": line_num,
            "line_content": line_content,
            "snippet": full_code,
            "language": language,
            "is_deterministic": True,
            "filter_stage": "stage1_deterministic",
            "llm_confidence": 1.0,
            "cwe_id": rule.get("cwe_id"),
            "owasp_category": rule.get("owasp_category"),
        }

    def get_rule(self, rule_id: str) -> Optional[dict]:
        """Retrieve a single rule by ID."""
        for rule in self.rules:
            if rule["rule_id"] == rule_id:
                return rule
        return None

    def list_rules(self, category: Optional[str] = None, language: Optional[str] = None) -> list[dict]:
        """List rules, optionally filtered."""
        result = self.rules
        if category:
            result = [r for r in result if r["category"] == category]
        if language:
            result = [r for r in result if r.get("language", "all") in ("all", language)]
        return result
