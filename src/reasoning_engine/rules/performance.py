"""Performance review rules.

Detect common performance anti-patterns:
- N+1 queries
- Blocking I/O in async contexts
- Inefficient string operations in loops
- Missing connection pooling
- Unnecessary allocations
"""

PERFORMANCE_RULES = [
    {
        "rule_id": "PERF-001",
        "name": "N+1 Query Pattern in Loop",
        "category": "performance",
        "severity": "major",
        "language": "all",
        "description": "Database query executed inside a loop — potential N+1 query problem.",
        "pattern": r'for\s+\w+\s+in\s+.+:\s*\n\s*(?:\.execute|\.query|\.fetch|\.find|\.get\()',
        "is_deterministic": True,
        "fix_suggestion": "Use eager loading (e.g., JOINs, select_related, prefetch_related, includes()) to load related data in a single query.",
    },
    {
        "rule_id": "PERF-002",
        "name": "String Concatenation in Loop",
        "category": "performance",
        "severity": "minor",
        "language": "all",
        "description": "String += inside a loop creates many intermediate string objects.",
        "pattern": r'for\s+\w+\s+in\s+.+:\s*\n\s*\w+\s*\+=\s*["\']',
        "is_deterministic": True,
        "fix_suggestion": "Use ''.join() in Python, StringBuilder in Java, strings.Builder in Go, or collect into a Vec and join.",
    },
    {
        "rule_id": "PERF-003",
        "name": "Blocking I/O in Async Context",
        "category": "performance",
        "severity": "major",
        "language": "python",
        "description": "Synchronous blocking call inside an async function — blocks the event loop.",
        "pattern": r'async\s+def\s+\w+.*:\s*\n.*(?:time\.sleep|requests\.(?:get|post)|open\(|file\.read\()',
        "is_deterministic": True,
        "fix_suggestion": "Use asyncio.sleep() instead of time.sleep(), httpx/aiohttp instead of requests, aiofiles for file I/O.",
    },
    {
        "rule_id": "PERF-004",
        "name": "Unnecessary List Copy",
        "category": "performance",
        "severity": "minor",
        "language": "python",
        "description": "Creating a full list copy when an iterator would suffice — wastes memory.",
        "pattern": r'(?:list|sorted|reversed)\s*\(\s*\w+\s*\)\s*\[',
        "is_deterministic": True,
        "fix_suggestion": "Use itertools.islice for slicing iterators, or pass the iterator directly if you only need to iterate once.",
    },
    {
        "rule_id": "PERF-005",
        "name": "Missing Database Connection Pooling",
        "category": "performance",
        "severity": "major",
        "language": "all",
        "description": "Creating a new database connection per request instead of using connection pooling.",
        "pattern": r'(?:connect|create_connection|MySQLdb\.connect|sqlite3\.connect)\s*\(\s*\)',
        "is_deterministic": True,
        "fix_suggestion": "Use SQLAlchemy connection pooling, or a connection pool library like asyncpg's built-in pool.",
    },
    {
        "rule_id": "PERF-006",
        "name": "Unbuffered I/O in Loop",
        "category": "performance",
        "severity": "minor",
        "language": "all",
        "description": "Small reads/writes in a loop without buffering — excessive system calls.",
        "pattern": r'for\s+\w+\s+in\s+.+:\s*\n\s*\.write\(',
        "is_deterministic": True,
        "fix_suggestion": "Use buffered I/O (io.BufferedReader, bufio.NewWriter) or batch writes with writelines().",
    },
    {
        "rule_id": "PERF-007",
        "name": "Regex Compiled Repeatedly in Loop",
        "category": "performance",
        "severity": "minor",
        "language": "all",
        "description": "re.compile() or new Regex() inside a loop — should be hoisted out.",
        "pattern": r'for\s+\w+\s+in\s+.+:\s*\n\s*re\.(?:compile|match|search)\s*\(',
        "is_deterministic": True,
        "fix_suggestion": "Move regex compilation outside the loop and reuse the compiled pattern object.",
    },
]
