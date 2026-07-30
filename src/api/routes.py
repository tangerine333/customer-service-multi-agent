"""API routes for the Code Review Agent."""

import uuid
from datetime import datetime, timezone
from typing import Optional

from fastapi import APIRouter, BackgroundTasks, HTTPException, Query
from pydantic import BaseModel, Field

router = APIRouter()


# --- Request/Response Models ---

class ReviewRequest(BaseModel):
    """Submit code for review."""
    repo_url: Optional[str] = None
    diff_content: str = Field(..., description="Unified diff or full source code")
    language: Optional[str] = Field(None, description="Primary language (python, javascript, go, etc.)")
    mr_id: Optional[str] = None
    base_branch: Optional[str] = "main"
    head_branch: Optional[str] = None
    rules: Optional[list[str]] = Field(None, description="Specific rule IDs to apply")
    max_files: int = Field(500, description="Maximum files to analyze")

class ReviewResponse(BaseModel):
    review_id: str
    status: str
    created_at: str

class FindingItem(BaseModel):
    id: int
    category: str
    severity: str
    title: str
    description: Optional[str]
    file_path: Optional[str]
    line_start: Optional[int]
    line_end: Optional[int]
    suggestion: Optional[str]
    auto_fix_code: Optional[str]
    auto_fix_applied: bool
    llm_confidence: Optional[float]

class ReviewDetail(BaseModel):
    review_id: str
    mr_id: Optional[str]
    status: str
    total_issues: int
    critical_count: int
    major_count: int
    minor_count: int
    info_count: int
    files_analyzed: int
    analysis_time_ms: Optional[int]
    findings: list[FindingItem]
    created_at: str
    completed_at: Optional[str]

class RuleInfo(BaseModel):
    rule_id: str
    name: str
    category: str
    severity: str
    language: Optional[str]
    description: Optional[str]
    is_enabled: bool

class EvaluationRequest(BaseModel):
    testset_path: str = Field(..., description="Path to annotated test dataset")
    rules: Optional[list[str]] = None

class EvaluationResponse(BaseModel):
    recall: float
    precision: float
    f1_score: float
    false_positive_rate: float
    total_samples: int
    true_positives: int
    false_positives: int
    false_negatives: int


# --- Routes ---

@router.get("/health")
async def health_check():
    """Health check endpoint."""
    return {
        "status": "healthy",
        "version": "0.1.0",
        "timestamp": datetime.now(timezone.utc).isoformat(),
    }


@router.post("/review", response_model=ReviewResponse)
async def submit_review(request: ReviewRequest, background_tasks: BackgroundTasks):
    """Submit a code review task. Processing happens asynchronously."""
    from ..reasoning_engine.reasoning_loop import ReviewOrchestrator

    review_id = str(uuid.uuid4())[:12]

    orchestrator = ReviewOrchestrator()
    background_tasks.add_task(
        orchestrator.run_review,
        review_id=review_id,
        diff_content=request.diff_content,
        language=request.language,
        mr_id=request.mr_id,
    )

    return ReviewResponse(
        review_id=review_id,
        status="pending",
        created_at=datetime.now(timezone.utc).isoformat(),
    )


@router.get("/review/{review_id}", response_model=ReviewDetail)
async def get_review(review_id: str):
    """Get the results of a completed or in-progress review."""
    from ..graph_store.postgres import get_review_by_id

    review = await get_review_by_id(review_id)
    if not review:
        raise HTTPException(status_code=404, detail="Review not found")

    return review


@router.get("/reviews", response_model=list[ReviewDetail])
async def list_reviews(
    mr_id: Optional[str] = None,
    status: Optional[str] = None,
    limit: int = Query(20, ge=1, le=100),
):
    """List recent reviews, optionally filtered."""
    from ..graph_store.postgres import list_reviews

    reviews = await list_reviews(mr_id=mr_id, status=status, limit=limit)
    return reviews


@router.get("/rules", response_model=list[RuleInfo])
async def get_rules(
    category: Optional[str] = None,
    language: Optional[str] = None,
    enabled_only: bool = True,
):
    """List available review rules."""
    from ..graph_store.postgres import get_rules

    rules = await get_rules(
        category=category,
        language=language,
        enabled_only=enabled_only,
    )
    return rules


@router.post("/rules/{rule_id}/toggle")
async def toggle_rule(rule_id: str, enabled: bool):
    """Enable or disable a specific rule."""
    from ..graph_store.postgres import toggle_rule

    success = await toggle_rule(rule_id, enabled)
    if not success:
        raise HTTPException(status_code=404, detail="Rule not found")
    return {"rule_id": rule_id, "enabled": enabled}


@router.post("/evaluate", response_model=EvaluationResponse)
async def run_evaluation(request: EvaluationRequest):
    """Run offline evaluation against a labeled test dataset."""
    from ..evaluation.evaluator import run_offline_evaluation

    result = await run_offline_evaluation(
        testset_path=request.testset_path,
        rules=request.rules,
    )
    return result


@router.get("/metrics")
async def get_metrics(period: str = Query("weekly", regex="^(daily|weekly|monthly)$")):
    """Get evaluation metrics for the given period."""
    from ..graph_store.postgres import get_metrics

    metrics = await get_metrics(period=period)
    return metrics
