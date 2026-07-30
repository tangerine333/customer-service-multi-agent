"""Code search tool - find functions, classes, patterns in the codebase.

MCP-capable: can be exposed as an MCP Server for standardized tool calling.
"""

import re
from typing import Optional


class CodeSearchTool:
    """Search code across the repository for symbols, patterns, and similar code."""

    def __init__(self):
        self._index: dict[str, list[dict]] = {}  # symbol_name -> [{file, line, snippet}]

    def index_files(self, files: list[dict]):
        """Build a search index from parsed files."""
        for file_info in files:
            path = file_info.get("path", "")
            for i, line in enumerate(file_info.get("added_lines", [])):
                # Index function definitions
                func_match = re.search(
                    r'(?:def|fn|func|function|class)\s+(\w+)', line
                )
                if func_match:
                    name = func_match.group(1)
                    self._index.setdefault(name, []).append({
                        "file": path,
                        "line": i + 1,
                        "snippet": line.strip(),
                        "kind": "function",
                    })

                # Index variable assignments
                var_match = re.search(r'(\w+)\s*[:=]', line)
                if var_match and not line.strip().startswith(("#", "//", "/*")):
                    name = var_match.group(1)
                    if name not in ("if", "for", "while", "return", "import", "from"):
                        self._index.setdefault(name, []).append({
                            "file": path,
                            "line": i + 1,
                            "snippet": line.strip(),
                            "kind": "variable",
                        })

    def search_symbol(
        self, name: str, kind: Optional[str] = None
    ) -> list[dict]:
        """Search for a symbol by name. Returns matching locations."""
        results = self._index.get(name, [])
        if kind:
            results = [r for r in results if r.get("kind") == kind]
        return results

    def search_pattern(self, pattern: str, files: list[dict]) -> list[dict]:
        """Search for a regex pattern across files. Returns matches."""
        compiled = re.compile(pattern, re.IGNORECASE)
        matches = []
        for file_info in files:
            path = file_info.get("path", "")
            for i, line in enumerate(file_info.get("added_lines", [])):
                if compiled.search(line):
                    matches.append({
                        "file": path,
                        "line": i + 1,
                        "snippet": line.strip(),
                    })
        return matches

    def find_similar_code(
        self, snippet: str, files: list[dict], threshold: float = 0.7
    ) -> list[dict]:
        """Find code similar to the given snippet (simplified fingerprint matching)."""
        # Simplified: use token overlap as similarity
        tokens = set(re.findall(r'\w+', snippet.lower()))
        if not tokens:
            return []

        results = []
        for file_info in files:
            path = file_info.get("path", "")
            for i, line in enumerate(file_info.get("added_lines", [])):
                line_tokens = set(re.findall(r'\w+', line.lower()))
                if not line_tokens:
                    continue
                overlap = len(tokens & line_tokens) / len(tokens)
                if overlap >= threshold:
                    results.append({
                        "file": path,
                        "line": i + 1,
                        "snippet": line.strip(),
                        "similarity": round(overlap, 2),
                    })
        return sorted(results, key=lambda r: r["similarity"], reverse=True)
