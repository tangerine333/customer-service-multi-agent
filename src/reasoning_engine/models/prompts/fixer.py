"""Auto-fix prompt templates.

Generate minimal, focused code fixes for confirmed issues.
"""

FIXER_SYSTEM_PROMPT = """You are an expert code fixer. Generate minimal, correct code fixes for identified issues.

## Fix Guidelines
1. **Minimal change**: Only fix the specific issue, do not refactor unrelated code
2. **Preserve style**: Match the existing code style (indentation, naming, patterns)
3. **Safe fix**: Ensure the fix doesn't introduce new issues or break existing tests
4. **Complete**: Include all necessary imports, error handling, and edge case coverage
5. **Explain**: Provide a brief explanation of what was changed and why

## Response Format
```json
{
  "fixed_code": "the complete fixed function/code block",
  "explanation": "what was changed and why",
  "side_effects": "potential side effects to watch for",
  "confidence": 0.0-1.0
}
```"""


def get_fix_prompt(issue: dict, code_context: str, language: str = "") -> str:
    """Build an auto-fix prompt for a specific issue."""
    return f"""{FIXER_SYSTEM_PROMPT}

**Issue**: {issue.get('title', '')}
**Category**: {issue.get('category', '')}
**Severity**: {issue.get('severity', '')}
**Language**: {language}

**Current Code**:
```
{code_context}
```

**Description**: {issue.get('description', '')}

Generate the fixed version of this code, addressing only the identified issue.
"""
