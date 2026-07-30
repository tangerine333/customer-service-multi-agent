"""Offline evaluation system for the Code Review Agent.

Computes:
- Recall (检出率): percentage of known issues that were detected
- Precision (精确率): percentage of reported issues that are real
- F1 Score: harmonic mean of recall and precision
- False Positive Rate
- Auto-fix pass rate
- Per-category breakdown
"""

import json
import logging
from typing import Optional

logger = logging.getLogger(__name__)


class Evaluator:
    """Computes evaluation metrics by comparing agent output to labeled data."""

    def __init__(self):
        self.metrics: dict[str, float] = {}

    def evaluate(
        self,
        ground_truth: list[dict],
        predictions: list[dict],
    ) -> dict:
        """Compare predictions against ground truth labels.

        Args:
            ground_truth: List of {"file": str, "line": int, "category": str, "severity": str}
            predictions: List of {"file_path": str, "line_start": int, "category": str, "severity": str}

        Returns:
            Dict with recall, precision, f1, false_positive_rate, and per-category metrics.
        """
        # Match predictions to ground truth by (file, line_range, category)
        matched_gt = set()
        matched_pred = set()

        for pi, pred in enumerate(predictions):
            for gi, gt in enumerate(ground_truth):
                if gi in matched_gt:
                    continue
                if self._is_match(pred, gt):
                    matched_gt.add(gi)
                    matched_pred.add(pi)
                    break

        true_positives = len(matched_pred)
        false_positives = len(predictions) - true_positives
        false_negatives = len(ground_truth) - len(matched_gt)

        recall = true_positives / max(true_positives + false_negatives, 1)
        precision = true_positives / max(true_positives + false_positives, 1)
        f1 = 2 * recall * precision / max(recall + precision, 0.001)
        fpr = false_positives / max(false_positives + true_positives, 1)

        # Per-category breakdown
        categories = {}
        for cat in set(g["category"] for g in ground_truth):
            gt_cat = [g for g in ground_truth if g["category"] == cat]
            pred_cat = [p for p in predictions if p.get("category") == cat]
            cat_result = self.evaluate(gt_cat, pred_cat)
            categories[cat] = {
                "recall": cat_result["recall"],
                "precision": cat_result["precision"],
                "count": len(gt_cat),
            }

        return {
            "recall": round(recall, 4),
            "precision": round(precision, 4),
            "f1_score": round(f1, 4),
            "false_positive_rate": round(fpr, 4),
            "true_positives": true_positives,
            "false_positives": false_positives,
            "false_negatives": false_negatives,
            "total_ground_truth": len(ground_truth),
            "total_predictions": len(predictions),
            "per_category": categories,
        }

    def _is_match(self, pred: dict, gt: dict) -> bool:
        """Check if a prediction matches a ground truth issue."""
        # Same file?
        pred_file = pred.get("file_path", "")
        gt_file = gt.get("file", "")
        if pred_file != gt_file:
            return False

        # Same category?
        if pred.get("category") != gt.get("category"):
            return False

        # Lines overlap? (within 3 lines tolerance)
        pred_line = pred.get("line_start", 0)
        gt_line = gt.get("line", 0)
        if abs(pred_line - gt_line) > 3:
            return False

        return True


async def run_offline_evaluation(
    testset_path: str,
    rules: Optional[list[str]] = None,
) -> dict:
    """Run the full offline evaluation pipeline.

    Args:
        testset_path: Path to JSON file with annotated test data.
        rules: Optional list of rule IDs to evaluate.

    Returns:
        Evaluation metrics dict.
    """
    evaluator = Evaluator()

    try:
        with open(testset_path, "r", encoding="utf-8") as f:
            testset = json.load(f)
    except FileNotFoundError:
        logger.error("Testset not found: %s", testset_path)
        return {"error": f"Testset not found: {testset_path}"}
    except json.JSONDecodeError as e:
        logger.error("Invalid JSON: %s", e)
        return {"error": f"Invalid JSON: {e}"}

    ground_truth = testset.get("ground_truth", [])
    samples = testset.get("samples", [])

    # Run agent on each sample and collect predictions
    from ..reasoning_engine.reasoning_loop import ReviewOrchestrator

    orchestrator = ReviewOrchestrator()
    all_predictions = []

    for sample in samples:
        diff = sample.get("diff", sample.get("code", ""))
        language = sample.get("language", "")

        # Parse diff and run rules (without LLM for speed)
        files = orchestrator._parse_diff(diff, language)
        findings = orchestrator.rule_engine.match_deterministic(files)
        all_predictions.extend(findings)

    # Filter by rules if specified
    if rules:
        all_predictions = [
            p for p in all_predictions
            if p.get("rule_id") in rules
        ]

    return evaluator.evaluate(ground_truth, all_predictions)


def compute_adoption_rate(feedback_data: list[dict]) -> dict:
    """Compute developer adoption rate from feedback data.

    Args:
        feedback_data: List of {"finding_id": int, "feedback": "useful"|"not_useful"|"duplicate"}

    Returns:
        Adoption metrics.
    """
    if not feedback_data:
        return {"adoption_rate": 0.0, "total_feedback": 0}

    useful = sum(1 for f in feedback_data if f.get("feedback") == "useful")
    total = len(feedback_data)

    return {
        "adoption_rate": round(useful / total, 4) if total > 0 else 0.0,
        "total_feedback": total,
        "useful": useful,
        "not_useful": sum(1 for f in feedback_data if f.get("feedback") == "not_useful"),
        "duplicate": sum(1 for f in feedback_data if f.get("feedback") == "duplicate"),
    }
