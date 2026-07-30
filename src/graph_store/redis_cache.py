"""Redis-backed summary cache for cross-function taint analysis.

Caches function taint summaries to avoid re-analyzing unchanged functions.
Invalidation is triggered by git diff: only changed functions have their cache
entries rebuilt.
"""

import hashlib
import json
import logging
import os
from typing import Optional

import redis.asyncio as aioredis

logger = logging.getLogger(__name__)

_redis: Optional[aioredis.Redis] = None


async def get_redis() -> aioredis.Redis:
    global _redis
    if _redis is None:
        redis_url = os.getenv("REDIS_URL", "redis://localhost:6379/0")
        _redis = await aioredis.from_url(redis_url, decode_responses=True)
    return _redis


async def get_cached_summary(function_name: str) -> Optional[dict]:
    """Get a cached taint summary for a function. Returns None if not cached or expired."""
    try:
        r = await get_redis()
        key = f"taint_summary:{function_name}"
        data = await r.get(key)
        if data:
            return json.loads(data)
    except Exception as e:
        logger.warning("Redis get failed: %s", e)
    return None


async def cache_summary(
    function_name: str,
    summary: dict,
    ttl: int = 3600,
):
    """Cache a taint summary for a function with TTL."""
    try:
        r = await get_redis()
        key = f"taint_summary:{function_name}"
        await r.setex(key, ttl, json.dumps(summary))
    except Exception as e:
        logger.warning("Redis cache failed: %s", e)


async def invalidate_function(function_name: str):
    """Invalidate cache for a specific function (called when the function is modified)."""
    try:
        r = await get_redis()
        key = f"taint_summary:{function_name}"
        await r.delete(key)
        logger.debug("Invalidated cache for %s", function_name)
    except Exception as e:
        logger.warning("Redis invalidate failed: %s", e)


async def invalidate_functions(function_names: list[str]):
    """Batch invalidate cache for multiple changed functions."""
    try:
        r = await get_redis()
        keys = [f"taint_summary:{name}" for name in function_names]
        if keys:
            await r.delete(*keys)
            logger.debug("Invalidated %d cache entries", len(keys))
    except Exception as e:
        logger.warning("Redis batch invalidate failed: %s", e)


def compute_summary_hash(function_source: str) -> str:
    """Compute a content hash for a function to detect changes."""
    return hashlib.sha256(function_source.encode()).hexdigest()[:16]


async def get_cache_stats() -> dict:
    """Get summary cache statistics."""
    try:
        r = await get_redis()
        keys = await r.keys("taint_summary:*")
        return {
            "cached_functions": len(keys),
            "memory_used": await r.memory_usage("taint_summary:*") if keys else 0,
        }
    except Exception:
        return {"cached_functions": 0, "memory_used": 0}
