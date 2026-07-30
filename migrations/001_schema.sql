-- ============================================================
-- Code Review Agent - Database Schema
-- PostgreSQL with Recursive CTE for Code Knowledge Graph
-- ============================================================

-- 1. 代码符号表 (Functions, Classes, Methods)
CREATE TABLE IF NOT EXISTS symbols (
    id              BIGSERIAL PRIMARY KEY,
    name            VARCHAR(512) NOT NULL,
    kind            VARCHAR(64) NOT NULL,  -- function, method, class, variable, module
    language        VARCHAR(32) NOT NULL,
    file_path       VARCHAR(1024) NOT NULL,
    line_start      INTEGER NOT NULL,
    line_end        INTEGER NOT NULL,
    col_start       INTEGER,
    col_end         INTEGER,
    signature       TEXT,
    return_type     VARCHAR(256),
    visibility      VARCHAR(32),           -- public, private, protected, internal
    is_exported     BOOLEAN DEFAULT false,
    commit_sha      VARCHAR(64),
    repo_id         INTEGER,
    created_at      TIMESTAMP DEFAULT NOW(),
    updated_at      TIMESTAMP DEFAULT NOW(),
    UNIQUE(file_path, name, line_start)
);

CREATE INDEX idx_symbols_name ON symbols(name);
CREATE INDEX idx_symbols_file ON symbols(file_path);
CREATE INDEX idx_symbols_kind ON symbols(kind);
CREATE INDEX idx_symbols_lang ON symbols(language);

-- 2. 调用关系表 (Call Graph Edges)
CREATE TABLE IF NOT EXISTS call_edges (
    id              BIGSERIAL PRIMARY KEY,
    caller_id       BIGINT NOT NULL REFERENCES symbols(id) ON DELETE CASCADE,
    callee_id       BIGINT NOT NULL REFERENCES symbols(id) ON DELETE CASCADE,
    call_line       INTEGER,
    call_type       VARCHAR(32),           -- direct, virtual, dynamic, callback
    arg_count       INTEGER,
    is_conditional  BOOLEAN DEFAULT false,
    created_at      TIMESTAMP DEFAULT NOW(),
    UNIQUE(caller_id, callee_id, call_line)
);

CREATE INDEX idx_call_edges_caller ON call_edges(caller_id);
CREATE INDEX idx_call_edges_callee ON call_edges(callee_id);

-- 3. 引用关系表 (Reference/Usage)
CREATE TABLE IF NOT EXISTS references (
    id              BIGSERIAL PRIMARY KEY,
    source_symbol_id BIGINT NOT NULL REFERENCES symbols(id) ON DELETE CASCADE,
    target_symbol_id BIGINT NOT NULL REFERENCES symbols(id) ON DELETE CASCADE,
    ref_type        VARCHAR(32),           -- import, type_ref, var_ref, inherit
    ref_line        INTEGER,
    created_at      TIMESTAMP DEFAULT NOW()
);

CREATE INDEX idx_refs_source ON references(source_symbol_id);
CREATE INDEX idx_refs_target ON references(target_symbol_id);

-- 4. 函数污点摘要缓存表 (Taint Summary Cache)
CREATE TABLE IF NOT EXISTS taint_summaries (
    id              BIGSERIAL PRIMARY KEY,
    symbol_id       BIGINT NOT NULL REFERENCES symbols(id) ON DELETE CASCADE,
    parameter_name  VARCHAR(256) NOT NULL,
    taint_propagates BOOLEAN DEFAULT false,
    taint_sources   JSONB,                 -- [{param, source_type: "user_input"|"file"|"network"}]
    taint_sinks     JSONB,                 -- [{param, sink_type: "sql"|"command"|"file_write"}]
    sanitizers      JSONB,                 -- [{function, location}]
    summary_hash    VARCHAR(64),           -- 用于判断摘要是否过期
    commit_sha      VARCHAR(64),
    created_at      TIMESTAMP DEFAULT NOW(),
    updated_at      TIMESTAMP DEFAULT NOW(),
    UNIQUE(symbol_id, parameter_name)
);

CREATE INDEX idx_taint_symbol ON taint_summaries(symbol_id);

-- 5. 审查记录表 (Review Records)
CREATE TABLE IF NOT EXISTS review_records (
    id              BIGSERIAL PRIMARY KEY,
    mr_id           VARCHAR(256),
    repo_url        VARCHAR(1024),
    branch_name     VARCHAR(256),
    commit_sha      VARCHAR(64),
    review_status   VARCHAR(32) DEFAULT 'pending',  -- pending, analyzing, completed, failed
    total_issues    INTEGER DEFAULT 0,
    critical_count  INTEGER DEFAULT 0,
    major_count     INTEGER DEFAULT 0,
    minor_count     INTEGER DEFAULT 0,
    info_count      INTEGER DEFAULT 0,
    files_analyzed  INTEGER DEFAULT 0,
    analysis_time_ms BIGINT,
    llm_calls       INTEGER DEFAULT 0,
    cache_hit_rate  DECIMAL(5,2),
    created_at      TIMESTAMP DEFAULT NOW(),
    completed_at    TIMESTAMP
);

CREATE INDEX idx_review_mr ON review_records(mr_id);
CREATE INDEX idx_review_status ON review_records(review_status);
CREATE INDEX idx_review_created ON review_records(created_at);

-- 6. 问题发现记录表 (Issue/Findings)
CREATE TABLE IF NOT EXISTS findings (
    id              BIGSERIAL PRIMARY KEY,
    review_id       BIGINT NOT NULL REFERENCES review_records(id) ON DELETE CASCADE,
    rule_id         VARCHAR(128),
    category        VARCHAR(64) NOT NULL,   -- security, performance, logic, style, api_compat, test_quality
    severity        VARCHAR(16) NOT NULL,   -- critical, major, minor, info
    title           VARCHAR(512) NOT NULL,
    description     TEXT,
    file_path       VARCHAR(1024),
    line_start      INTEGER,
    line_end        INTEGER,
    symbol_name     VARCHAR(512),
    taint_source    TEXT,
    taint_sink      TEXT,
    evidence        JSONB,                 -- [{type, location, snippet}]
    suggestion      TEXT,
    auto_fix_code   TEXT,
    auto_fix_applied BOOLEAN DEFAULT false,
    auto_fix_status VARCHAR(32),           -- pending, applied, failed, reverted
    llm_confidence  DECIMAL(5,2),
    filter_stage    VARCHAR(32),           -- stage1_deterministic, stage2_context, stage3_llm
    is_false_positive BOOLEAN DEFAULT false,
    reviewer_feedback VARCHAR(32),         -- useful, not_useful, duplicate, fixed
    created_at      TIMESTAMP DEFAULT NOW()
);

CREATE INDEX idx_findings_review ON findings(review_id);
CREATE INDEX idx_findings_category ON findings(category);
CREATE INDEX idx_findings_severity ON findings(severity);
CREATE INDEX idx_findings_rule ON findings(rule_id);

-- 7. 审查规则表 (Review Rules)
CREATE TABLE IF NOT EXISTS rules (
    id              BIGSERIAL PRIMARY KEY,
    rule_id         VARCHAR(128) UNIQUE NOT NULL,
    name            VARCHAR(256) NOT NULL,
    category        VARCHAR(64) NOT NULL,   -- security, performance, logic, style, api_compat, test_quality
    severity        VARCHAR(16) NOT NULL,
    language        VARCHAR(32),            -- all, python, javascript, go, java, rust
    description     TEXT,
    pattern         TEXT,                   -- AST pattern or regex
    fix_suggestion  TEXT,
    is_enabled      BOOLEAN DEFAULT true,
    is_deterministic BOOLEAN DEFAULT true,  -- 是否可被确定性规则检测
    cwe_id          VARCHAR(32),            -- CWE编号 (安全规则)
    owasp_category  VARCHAR(64),            -- OWASP类别
    created_at      TIMESTAMP DEFAULT NOW(),
    updated_at      TIMESTAMP DEFAULT NOW()
);

CREATE INDEX idx_rules_category ON rules(category);
CREATE INDEX idx_rules_enabled ON rules(is_enabled);

-- 8. 评估指标表 (Evaluation Metrics)
CREATE TABLE IF NOT EXISTS evaluation_metrics (
    id              BIGSERIAL PRIMARY KEY,
    period          VARCHAR(32) NOT NULL,   -- daily, weekly, monthly
    period_start    DATE NOT NULL,
    recall_rate     DECIMAL(5,2),
    precision_rate  DECIMAL(5,2),
    false_positive_rate DECIMAL(5,2),
    avg_review_time_ms  BIGINT,
    p50_review_time_ms  BIGINT,
    p99_review_time_ms  BIGINT,
    auto_fix_pass_rate  DECIMAL(5,2),
    adoption_rate   DECIMAL(5,2),
    total_reviews   INTEGER DEFAULT 0,
    total_findings  INTEGER DEFAULT 0,
    llm_call_count  INTEGER DEFAULT 0,
    created_at      TIMESTAMP DEFAULT NOW(),
    UNIQUE(period, period_start)
);

-- ============================================================
-- Recursive CTE Queries for Code Graph Traversal
-- ============================================================

-- 查询函数的所有上游调用者（N跳以内）
-- CREATE OR REPLACE FUNCTION get_callers(
--     target_name VARCHAR, max_hops INT DEFAULT 3
-- ) RETURNS TABLE(symbol_id BIGINT, name VARCHAR, file_path VARCHAR, hop INT) AS $$
-- BEGIN
--     RETURN QUERY
--     WITH RECURSIVE caller_chain AS (
--         -- Base: direct callers
--         SELECT s.id, s.name, s.file_path, 1 AS hop
--         FROM symbols s
--         JOIN call_edges ce ON s.id = ce.caller_id
--         JOIN symbols target ON ce.callee_id = target.id
--         WHERE target.name = target_name
--         UNION ALL
--         -- Recursive: callers of callers
--         SELECT s.id, s.name, s.file_path, cc.hop + 1
--         FROM symbols s
--         JOIN call_edges ce ON s.id = ce.caller_id
--         JOIN caller_chain cc ON ce.callee_id = cc.symbol_id
--         WHERE cc.hop < max_hops
--     )
--     SELECT DISTINCT ON (caller_chain.id)
--         caller_chain.id, caller_chain.name, caller_chain.file_path, caller_chain.hop
--     FROM caller_chain
--     ORDER BY caller_chain.id, caller_chain.hop;
-- END;
-- $$ LANGUAGE plpgsql;
