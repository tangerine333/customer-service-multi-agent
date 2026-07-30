"""Git operations tool - git blame, log, diff, and history analysis.

MCP-capable: can be exposed as an MCP Server for standardized tool calling.
"""

import subprocess
from datetime import datetime
from typing import Optional


class GitOpsTool:
    """Perform git operations for code review context."""

    def __init__(self, repo_path: str = "."):
        self.repo_path = repo_path

    def get_blame(self, file_path: str, line: int) -> Optional[dict]:
        """Get git blame info for a specific line."""
        try:
            result = subprocess.run(
                ["git", "blame", "-L", f"{line},{line}", "--porcelain", file_path],
                cwd=self.repo_path,
                capture_output=True,
                text=True,
                timeout=10,
            )
            if result.returncode == 0:
                lines = result.stdout.strip().split("\n")
                if lines:
                    # Parse porcelain format
                    commit_hash = lines[0].split()[0]
                    author = ""
                    author_time = ""
                    for l in lines[1:]:
                        if l.startswith("author "):
                            author = l[7:]
                        elif l.startswith("author-time "):
                            ts = int(l[10:])
                            author_time = datetime.fromtimestamp(ts).isoformat()
                    return {
                        "commit": commit_hash[:8],
                        "author": author,
                        "time": author_time,
                        "line": line,
                        "file": file_path,
                    }
        except (subprocess.TimeoutExpired, FileNotFoundError):
            pass
        return None

    def get_commit_info(self, commit_sha: str) -> Optional[dict]:
        """Get detailed info about a commit."""
        try:
            result = subprocess.run(
                ["git", "show", "--stat", "--format=%H%n%an%n%ae%n%aI%n%s", commit_sha],
                cwd=self.repo_path,
                capture_output=True,
                text=True,
                timeout=10,
            )
            if result.returncode == 0:
                parts = result.stdout.strip().split("\n")
                return {
                    "hash": parts[0][:8] if parts else "",
                    "author": parts[1] if len(parts) > 1 else "",
                    "email": parts[2] if len(parts) > 2 else "",
                    "date": parts[3] if len(parts) > 3 else "",
                    "message": parts[4] if len(parts) > 4 else "",
                }
        except (subprocess.TimeoutExpired, FileNotFoundError):
            pass
        return None

    def get_diff(self, base_branch: str = "main") -> Optional[str]:
        """Get the unified diff of current branch vs base."""
        try:
            result = subprocess.run(
                ["git", "diff", base_branch, "--unified=3"],
                cwd=self.repo_path,
                capture_output=True,
                text=True,
                timeout=30,
            )
            if result.returncode == 0:
                return result.stdout
        except (subprocess.TimeoutExpired, FileNotFoundError):
            pass
        return None

    def get_changed_files(self, base_branch: str = "main") -> list[str]:
        """Get list of files changed vs base branch."""
        try:
            result = subprocess.run(
                ["git", "diff", "--name-only", base_branch],
                cwd=self.repo_path,
                capture_output=True,
                text=True,
                timeout=10,
            )
            if result.returncode == 0:
                return [f for f in result.stdout.strip().split("\n") if f]
        except (subprocess.TimeoutExpired, FileNotFoundError):
            pass
        return []

    def get_file_history(self, file_path: str, max_commits: int = 10) -> list[dict]:
        """Get recent commit history for a file."""
        try:
            result = subprocess.run(
                ["git", "log", "--oneline", "-n", str(max_commits), "--", file_path],
                cwd=self.repo_path,
                capture_output=True,
                text=True,
                timeout=10,
            )
            if result.returncode == 0:
                commits = []
                for line in result.stdout.strip().split("\n"):
                    if line:
                        parts = line.split(" ", 1)
                        commits.append({
                            "hash": parts[0],
                            "message": parts[1] if len(parts) > 1 else "",
                        })
                return commits
        except (subprocess.TimeoutExpired, FileNotFoundError):
            pass
        return []

    def is_test_file(self, file_path: str) -> bool:
        """Check if a file is a test file."""
        test_markers = [
            "_test.", "test_", "__tests__", "spec.", ".spec.",
            "/tests/", "/test/", "/__tests__/",
        ]
        return any(marker in file_path for marker in test_markers)

    def blame_authors_for_lines(
        self, file_path: str, lines: list[int]
    ) -> dict[int, dict]:
        """Get authors for multiple lines (batch blame)."""
        results = {}
        for line in lines:
            blame = self.get_blame(file_path, line)
            if blame:
                results[line] = blame
        return results
