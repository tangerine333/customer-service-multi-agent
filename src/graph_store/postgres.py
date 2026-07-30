"""PostgreSQL-backed graph store for code knowledge graph.

Stores symbols, call edges, references, taint summaries, review records,
and evaluation metrics using PostgreSQL with recursive CTE for graph traversal.
"""

import logging
from datetime import datetime, timezone
from typing import Optional

import asyncpg

logger = logging.getLogger(__name__)

_pool: Optional[asyncpg.Pool] = None


async def init_db():
    """Initialize the database connection pool and run migrations."""
    global _pool
    import os

    dsn = os.getenv("DATABASE_URL", "postgresql://code_review:code_review@localhost:5432/code_review")
    _pool = await asyncpg.create_pool(dsn, min_size=2, max_size=10)
    logger.info("Database connection pool initialized")


async def get_pool() -> asyncpg.Pool:
    if _pool is None:
        await init_db()
    return _pool


# --- Review CRUD ---

async def save_review(
    review_id: str,
    mr_id: Optional[str] = None,
    status: str = "pending",
    repo_url: Optional[str] = None,
    commit_sha: Optional[str] = None,
):
    pool = await get_pool()
    async with pool.acquire() as conn:
        await conn.execute(
            """INSERT INTO review_records (id, mr_id, review_status, repo_url, commit_sha)
               VALUES (substring($1 from 1 for 12)::bigint, $2, $3, $4, $5)
               ON CONFLICT DO NOTHING""",
            review_id, mr_id, status, repo_url, commit_sha,
        )


async def update_review_status(
    review_id: str,
    status: str,
    total_issues: int = 0,
    critical_count: int = 0,
    major_count: int = 0,
    minor_count: int = 0,
    info_count: int = 0,
    files_analyzed: int = 0,
    analysis_time_ms: int = 0,
    llm_calls: int = 0,
    completed_at: Optional[datetime] = None,
):
    pool = await get_pool()
    async with pool.acquire() as conn:
        await conn.execute(
            """UPDATE review_records SET
                 review_status = $2, total_issues = $3, critical_count = $4,
                 major_count = $5, minor_count = $6, info_count = $7,
                 files_analyzed = $8, analysis_time_ms = $9, llm_calls = $10,
                 completed_at = $11
               WHERE id = substring($1 from 1 for 12)::bigint""",
            review_id, status, total_issues, critical_count,
            major_count, minor_count, info_count,
            files_analyzed, analysis_time_ms, llm_calls,
            completed_at,
        )


async def get_review_by_id(review_id: str) -> Optional[dict]:
    pool = await get_pool()
    async with pool.acquire() as conn:
        row = await conn.fetchrow(
            "SELECT * FROM review_records WHERE id = substring($1 from 1 for 12)::bigint",
            review_id,
        )
        if not row:
            return None

        findings = await conn.fetch(
            "SELECT * FROM findings WHERE review_id = $1",
            row["id"],
        )

        return {
            "review_id": review_id,
            "mr_id": row["mr_id"],
            "status": row["review_status"],
            "total_issues": row["total_issues"],
            "critical_count": row["critical_count"],
            "major_count": row["major_count"],
            "minor_count": row["minor_count"],
            "info_count": row["info_count"],
            "files_analyzed": row["files_analyzed"],
            "analysis_time_ms": row["analysis_time_ms"],
            "findings": [dict(f) for f in findings],
            "created_at": row["created_at"].isoformat() if row["created_at"] else "",
            "completed_at": row["completed_at"].isoformat() if row["completed_at"] else "",
        }


async def list_reviews(
    mr_id: Optional[str] = None,
    status: Optional[str] = None,
    limit: int = 20,
) -> list[dict]:
    pool = await get_pool()
    query = "SELECT * FROM review_records WHERE 1=1"
    params = []

    if mr_id:
        params.append(mr_id)
        query += f" AND mr_id = ${len(params)}"
    if status:
        params.append(status)
        query += f" AND review_status = ${len(params)}"

    query += f" ORDER BY created_at DESC LIMIT {limit}"

    async with pool.acquire() as conn:
        rows = await conn.fetch(query, *params)
        return [
            {
                "review_id": str(r["id"]),
                "mr_id": r["mr_id"],
                "status": r["review_status"],
                "total_issues": r["total_issues"],
                "critical_count": r["critical_count"],
                "major_count": r["major_count"],
                "minor_count": r["minor_count"],
                "info_count": r["info_count"],
                "files_analyzed": r["files_analyzed"],
                "analysis_time_ms": r["analysis_time_ms"],
                "findings": [],
                "created_at": r["created_at"].isoformat() if r["created_at"] else "",
                "completed_at": r["completed_at"].isoformat() if r["completed_at"] else "",
            }
            for r in rows
        ]


async def save_findings(review_id: str, findings: list[dict]):
    pool = await get_pool()
    async with pool.acquire() as conn:
        # Get the internal review id
        row = await conn.fetchrow(
            "SELECT id FROM review_records WHERE id = substring($1 from 1 for 12)::bigint",
            review_id,
        )
        if not row:
            return
        internal_id = row["id"]

        for finding in findings:
            await conn.execute(
                """INSERT INTO findings (
                     review_id, rule_id, category, severity, title, description,
                     file_path, line_start, line_end, suggestion, auto_fix_code,
                     llm_confidence, filter_stage
                   ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13)""",
                internal_id,
                finding.get("rule_id", ""),
                finding.get("category", "unknown"),
                finding.get("severity", "info"),
                finding.get("title", ""),
                finding.get("description", ""),
                finding.get("file_path", ""),
                finding.get("line_start", 0),
                finding.get("line_end", 0),
                finding.get("suggestion", ""),
                finding.get("auto_fix_code"),
                finding.get("llm_confidence", 1.0),
                finding.get("filter_stage", "stage1_deterministic"),
            )


async def get_rules(
    category: Optional[str] = None,
    language: Optional[str] = None,
    enabled_only: bool = True,
) -> list[dict]:
    pool = await get_pool()
    query = "SELECT * FROM rules WHERE 1=1"
    params = []

    if enabled_only:
        query += " AND is_enabled = TRUE"
    if category:
        params.append(category)
        query += f" AND category = ${len(params)}"
    if language:
        params.append(language)
        query += f" AND (language = ${len(params)} OR language = 'all')"

    query += " ORDER BY category, severity, rule_id"

    async with pool.acquire() as conn:
        rows = await conn.fetch(query, *params)
        return [dict(r) for r in rows]


async def toggle_rule(rule_id: str, enabled: bool) -> bool:
    pool = await get_pool()
    async with pool.acquire() as conn:
        result = await conn.execute(
            "UPDATE rules SET is_enabled = $2 WHERE rule_id = $1",
            rule_id, enabled,
        )
        return result != "UPDATE 0"


async def get_metrics(period: str = "weekly") -> list[dict]:
    pool = await get_pool()
    async with pool.acquire() as conn:
        rows = await conn.fetch(
            "SELECT * FROM evaluation_metrics WHERE period = $1 ORDER BY period_start DESC LIMIT 10",
            period,
        )
        return [dict(r) for r in rows]
