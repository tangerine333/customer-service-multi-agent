"""Code style review rules.

Detect code style issues:
- Excessive function complexity
- Too many parameters
- Deep nesting
- Inconsistent naming
"""

STYLE_RULES = [
    {
        "rule_id": "STYLE-001",
        "name": "Function Too Complex (Too Many Lines)",
        "category": "style",
        "severity": "minor",
        "language": "all",
        "description": "Function body exceeds 50 lines — consider refactoring into smaller functions.",
        "pattern": None,  # Requires AST-level analysis, not regex
        "is_deterministic": False,  # Needs AST
        "fix_suggestion": "Break the function into smaller, single-responsibility functions with descriptive names.",
    },
    {
        "rule_id": "STYLE-002",
        "name": "Too Many Function Parameters (> 5)",
        "category": "style",
        "severity": "minor",
        "language": "all",
        "description": "Function has more than 5 parameters — hard to understand and call correctly.",
        "pattern": r'def\s+\w+\s*\([^)]*,[^)]*,[^)]*,[^)]*,[^)]*,',
        "is_deterministic": True,
        "fix_suggestion": "Group related parameters into a dataclass/struct/Pydantic model, or use the builder pattern.",
    },
    {
        "rule_id": "STYLE-003",
        "name": "Bare Except Clause",
        "category": "style",
        "severity": "minor",
        "language": "python",
        "description": "Bare 'except:' catches all exceptions including KeyboardInterrupt and SystemExit.",
        "pattern": r'except\s*:',
        "is_deterministic": True,
        "fix_suggestion": "Catch specific exception types: except ValueError, except (TypeError, KeyError). At minimum use except Exception.",
    },
    {
        "rule_id": "STYLE-004",
        "name": "Mutable Default Argument",
        "category": "style",
        "severity": "major",
        "language": "python",
        "description": "Mutable default argument (list/dict) is shared across all function calls.",
        "pattern": r'def\s+\w+\s*\(.*=\s*\[\s*\]|def\s+\w+\s*\(.*=\s*\{\s*\}',
        "is_deterministic": True,
        "fix_suggestion": "Use None as the default and initialize the mutable object inside the function body.",
    },
    {
        "rule_id": "STYLE-005",
        "name": "Deeply Nested Code (> 3 levels)",
        "category": "style",
        "severity": "minor",
        "language": "all",
        "description": "Code nesting exceeds 3 levels — hard to follow the logic.",
        "pattern": None,  # Requires AST
        "is_deterministic": False,
        "fix_suggestion": "Use early returns (guard clauses), extract nested logic into helper functions, or use pattern matching.",
    },
    {
        "rule_id": "STYLE-006",
        "name": "Magic Number Without Explanation",
        "category": "style",
        "severity": "info",
        "language": "all",
        "description": "Numeric literal used without being assigned to a named constant — unclear meaning.",
        "pattern": r'(?:\b\d{4,}\b|\b\d{2,}\.\d{2,}\b)',  # Heuristic: long numbers
        "is_deterministic": False,  # High false positive rate with regex
        "fix_suggestion": "Assign the number to a named constant (e.g., MAX_RETRIES = 3, TIMEOUT_SECONDS = 30).",
    },
    {
        "rule_id": "STYLE-007",
        "name": "Unused Import",
        "category": "style",
        "severity": "info",
        "language": "all",
        "description": "Imported module that is never used in the file.",
        "pattern": None,  # Requires symbol resolution
        "is_deterministic": False,
        "fix_suggestion": "Remove the unused import to keep the codebase clean.",
    },
]
