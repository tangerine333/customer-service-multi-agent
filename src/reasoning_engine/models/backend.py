"""LLM model backend for semantic code review judgment.

Supports:
- vLLM (self-hosted, primary)
- OpenAI-compatible API (fallback)
- Model circuit-breaker (熔断): skip LLM on timeout or consecutive failures
"""

import asyncio
import logging
import os
from typing import Optional

import httpx

logger = logging.getLogger(__name__)


class CircuitBreakerOpen(Exception):
    """Raised when LLM calls are skipped due to circuit breaker."""


class ModelBackend:
    """Manages LLM calls for semantic code review judgment."""

    def __init__(self):
        self.api_url = os.getenv("VLLM_API_URL", "http://localhost:8000/v1")
        self.model_name = os.getenv("VLLM_MODEL_NAME", "deepseek-coder-33b-instruct")
        self.timeout = int(os.getenv("LLM_TIMEOUT_SECONDS", "5"))
        self.max_retries = int(os.getenv("LLM_MAX_RETRIES", "3"))

        # Circuit breaker state
        self._failure_count = 0
        self._circuit_open = False

    async def judge(self, finding: dict) -> dict:
        """Ask LLM to judge whether a finding is a real issue.

        Implements the circuit breaker pattern:
        - Timeout > 5s: skip LLM
        - 3 consecutive failures: open circuit, skip all subsequent LLM calls
        """
        if self._circuit_open:
            # Circuit open: output deterministic results only, marked low confidence
            return {"is_issue": finding.get("is_deterministic", False), "confidence": 0.3}

        try:
            prompt = self._build_judgment_prompt(finding)
            response = await self._call_llm(prompt)
            self._failure_count = 0  # Reset on success
            return response
        except asyncio.TimeoutError:
            self._failure_count += 1
            logger.warning("LLM timeout (%d/%d)", self._failure_count, self.max_retries)
            if self._failure_count >= self.max_retries:
                self._circuit_open = True
                logger.error("Circuit breaker OPEN - skipping LLM judgments")
            return {"is_issue": False, "confidence": 0.0}
        except Exception as e:
            self._failure_count += 1
            logger.warning("LLM error: %s (%d/%d)", e, self._failure_count, self.max_retries)
            if self._failure_count >= self.max_retries:
                self._circuit_open = True
            return {"is_issue": False, "confidence": 0.0}

    async def generate_fix(self, finding: dict) -> Optional[str]:
        """Generate auto-fix code for a confirmed issue."""
        prompt = self._build_fix_prompt(finding)
        try:
            response = await self._call_llm(prompt, max_tokens=1024)
            return response.get("fix_code")
        except Exception:
            return None

    async def _call_llm(self, prompt: str, max_tokens: int = 512) -> dict:
        """Make an LLM API call with timeout and retry."""
        async with httpx.AsyncClient(timeout=self.timeout) as client:
            for attempt in range(self.max_retries):
                try:
                    resp = await client.post(
                        f"{self.api_url}/chat/completions",
                        json={
                            "model": self.model_name,
                            "messages": [
                                {"role": "system", "content": "You are a code review expert. Respond with JSON only."},
                                {"role": "user", "content": prompt},
                            ],
                            "max_tokens": max_tokens,
                            "temperature": 0.1,
                        },
                    )
                    resp.raise_for_status()
                    data = resp.json()
                    content = data["choices"][0]["message"]["content"]
                    # Parse JSON from response
                    import json
                    # Extract JSON block if wrapped in markdown
                    if "```json" in content:
                        content = content.split("```json")[1].split("```")[0]
                    elif "```" in content:
                        content = content.split("```")[1].split("```")[0]
                    return json.loads(content.strip())
                except (httpx.TimeoutException, asyncio.TimeoutError):
                    if attempt == self.max_retries - 1:
                        raise
                    await asyncio.sleep(0.5 * (attempt + 1))
                except Exception as e:
                    logger.warning("LLM API error (attempt %d): %s", attempt + 1, e)
                    if attempt == self.max_retries - 1:
                        raise
                    await asyncio.sleep(0.5 * (attempt + 1))

        return {"is_issue": False, "confidence": 0.0}

    def _build_judgment_prompt(self, finding: dict) -> str:
        """Build a focused prompt for LLM semantic judgment."""
        category = finding.get("category", "unknown")
        title = finding.get("title", "")
        description = finding.get("description", "")
        snippet = finding.get("snippet", "")
        language = finding.get("language", "")

        return f"""You are a code review expert. Analyze the following code and determine if there is a real issue.

**Issue Category**: {category}
**Issue Description**: {title} - {description}

**Code Snippet** ({language}):
```
{snippet}
```

**Instructions**:
1. Determine if this is a genuine issue or a false positive
2. Consider: Is the suspicious pattern actually safe in this context?
3. Consider: Does the code have proper sanitization/validation before the suspicious operation?
4. Rate your confidence (0.0 to 1.0)

Respond with JSON:
```json
{{"is_issue": true/false, "confidence": 0.0-1.0, "reasoning": "brief explanation"}}
```"""

    def _build_fix_prompt(self, finding: dict) -> str:
        """Build prompt for auto-fix generation."""
        snippet = finding.get("snippet", "")
        language = finding.get("language", "")
        title = finding.get("title", "")

        return f"""Generate a fix for the following code issue.

**Issue**: {title}
**Language**: {language}

**Original Code**:
```
{snippet}
```

**Instructions**: Generate the corrected code. Keep the fix minimal and focused on the specific issue.

Respond with JSON:
```json
{{"fix_code": "the corrected code here"}}
```"""

    def reset_circuit(self):
        """Reset the circuit breaker (e.g., after a new model is deployed)."""
        self._failure_count = 0
        self._circuit_open = False
        logger.info("Circuit breaker reset")
