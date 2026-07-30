-- Seed data: built-in review rules
INSERT INTO rules (rule_id, name, category, severity, language, description, pattern, is_deterministic, cwe_id, owasp_category) VALUES
-- Security Rules
('SEC-001', 'SQL Injection via String Concatenation', 'security', 'critical', 'all',
 'Detects SQL queries built with string concatenation/user input. Matches patterns like: query = "SELECT * FROM users WHERE id=" + user_input',
 '"SELECT" + .* + (input|request|param|body|query)', true, 'CWE-89', 'A03:2021-Injection'),
('SEC-002', 'Hardcoded Secret/API Key', 'security', 'critical', 'all',
 'Detects hardcoded API keys, tokens, passwords in source code.',
 '(api_key|secret|password|token|api_secret)\s*=\s*["\''][a-zA-Z0-9_-]{20,}["\'''], true, 'CWE-798', 'A07:2021-Identification Failures'),
('SEC-003', 'Command Injection via os.system/subprocess', 'security', 'critical', 'all',
 'Detects shell command injection through user input in os.system() or subprocess calls.',
 '(os\.system|subprocess\.call|subprocess\.Popen)\(.*(input|request|param)', true, 'CWE-78', 'A03:2021-Injection'),
('SEC-004', 'Path Traversal Vulnerability', 'security', 'critical', 'all',
 'Detects file path construction from user input without sanitization.',
 '(open|file|read|write)\(.*\.\.\/.*', true, 'CWE-22', 'A01:2021-Broken Access Control'),
('SEC-005', 'XSS via Unsanitized Output', 'security', 'major', 'javascript',
 'Detects potential XSS when user input is directly injected into HTML.',
 '(innerHTML|outerHTML|document\.write)\(.*', true, 'CWE-79', 'A03:2021-Injection'),
('SEC-006', 'Missing CSRF Protection', 'security', 'major', 'all',
 'Detects endpoints that modify state without CSRF token.',
 '@app\.route.*methods.*POST.*\n(?!.*@csrf)', false, 'CWE-352', 'A01:2021-Broken Access Control'),

-- Performance Rules
('PERF-001', 'N+1 Query Pattern in Loop', 'performance', 'major', 'all',
 'Detects database queries inside for/while loops that could cause N+1 problems.',
 'for.*:\n\s*.*\.(query|execute|fetch|find)', true, NULL, NULL),
('PERF-002', 'Inefficient String Concatenation in Loop', 'performance', 'minor', 'all',
 'Detects string += inside loops - use StringBuilder or join() instead.',
 'for.*:\n\s*.*\+=\s*["\''], true, NULL, NULL),
('PERF-003', 'Synchronous I/O in Async Context', 'performance', 'major', 'python',
 'Detects blocking I/O calls inside async functions (e.g., time.sleep in async def).',
 'async def.*:\n.*time\.sleep|open\(|requests\.', true, NULL, NULL),
('PERF-004', 'Missing Database Index Hint', 'performance', 'minor', 'all',
 'Detects WHERE/ORDER BY on columns without database indexes (requires schema context).',
 NULL, false, NULL, NULL),

-- Logic Rules
('LOGIC-001', 'Possible Null Pointer / None Reference', 'logic', 'major', 'all',
 'Detects potential null dereference when variable is checked for null above but used below without guard.',
 'if.*is None.*return.*\n.*\.(\w+)\(', false, 'CWE-476', NULL),
('LOGIC-002', 'Missing Boundary Check', 'logic', 'major', 'all',
 'Detects array/list access without bounds checking in loops.',
 '(list|array|vec)\[.*\].*\n(?!.*if.*len)', false, 'CWE-129', NULL),
('LOGIC-003', 'Unhandled Exception / Error', 'logic', 'major', 'all',
 'Detects function calls that may raise exceptions without try/except wrapping.',
 '(json\.loads|int\(|float\(|open\().*(?!.*except)', false, 'CWE-754', NULL),

-- Style Rules
('STYLE-001', 'Function Too Complex (Cyclomatic Complexity > 15)', 'style', 'minor', 'all',
 'Functions with cyclomatic complexity exceeding 15 are hard to test and maintain.',
 NULL, true, NULL, NULL),
('STYLE-002', 'Too Many Function Parameters (> 5)', 'style', 'minor', 'all',
 'Functions with more than 5 parameters should consider using a parameter object.',
 'def \w+\(.*,.*,.*,.*,.*,', true, NULL, NULL),
('STYLE-003', 'Variable Naming Too Short', 'style', 'info', 'all',
 'Single-letter variable names (except loop counters i, j, x, y) reduce readability.',
 '(?<!\w)[a-z&&[^ijkxy]]\s*=(?!.*(?:lambda|for))\"', true, NULL, NULL),

-- API Compatibility Rules
('API-001', 'Deprecated API Usage Detected', 'api_compat', 'major', 'all',
 'Detects usage of APIs marked as deprecated in the codebase.',
 '@deprecated|# deprecated|// deprecated', true, NULL, NULL),
('API-002', 'Breaking Change: Function Signature Modified', 'api_compat', 'critical', 'all',
 'Detects changes to public function signatures that may break callers.',
 NULL, true, NULL, NULL),

-- Test Quality Rules
('TEST-001', 'Test Missing Assertion', 'test_quality', 'major', 'all',
 'Test function that does not contain any assert/expect statement.',
 'def test_.*:\n(?!.*assert)', true, NULL, NULL),
('TEST-002', 'Hardcoded Test Values Without Meaning', 'test_quality', 'minor', 'all',
 'Test using magic numbers without explaining their significance.',
 'assert.*==\s*\d+(?!.*#)', false, NULL, NULL);
