import React, { useState } from 'react'
import { runEvaluation, getMetrics, EvaluationResult, MetricsData } from '../services/api'

const Evaluation: React.FC = () => {
  const [testsetPath, setTestsetPath] = useState('./tests/fixtures/eval_dataset.json')
  const [loading, setLoading] = useState(false)
  const [result, setResult] = useState<EvaluationResult | null>(null)
  const [metrics, setMetrics] = useState<MetricsData[]>([])
  const [error, setError] = useState('')

  const handleEvaluate = async () => {
    setLoading(true)
    setError('')
    setResult(null)
    try {
      const { data } = await runEvaluation(testsetPath)
      setResult(data)
    } catch (err: any) {
      setError(err?.response?.data?.detail || 'Evaluation failed')
    } finally {
      setLoading(false)
    }
  }

  const handleLoadMetrics = async () => {
    try {
      const { data } = await getMetrics('weekly')
      setMetrics(data)
    } catch (err: any) {
      setError(err?.response?.data?.detail || 'Failed to load metrics')
    }
  }

  const percentColor = (val: number) => {
    if (val >= 0.8) return '#28a745'
    if (val >= 0.7) return '#fd7e14'
    return '#dc3545'
  }

  return (
    <div className="evaluation">
      <header className="page-header">
        <h1>Evaluation</h1>
      </header>

      <div className="eval-form card">
        <h3>Run Offline Evaluation</h3>
        <div className="form-row">
          <input
            type="text"
            value={testsetPath}
            onChange={(e) => setTestsetPath(e.target.value)}
            placeholder="Path to test dataset JSON"
          />
          <button onClick={handleEvaluate} disabled={loading} className="btn-primary">
            {loading ? 'Running...' : 'Run Evaluation'}
          </button>
        </div>
        {error && <p className="error-text">{error}</p>}
      </div>

      {result && (
        <div className="eval-results card">
          <h3>Results</h3>
          <div className="metrics-grid">
            <div className="metric-card">
              <span className="metric-label">Recall (检出率)</span>
              <span className="metric-value" style={{ color: percentColor(result.recall) }}>
                {(result.recall * 100).toFixed(1)}%
              </span>
              <span className="metric-target">Target: &gt; 80%</span>
            </div>
            <div className="metric-card">
              <span className="metric-label">Precision (精确率)</span>
              <span className="metric-value" style={{ color: percentColor(result.precision) }}>
                {(result.precision * 100).toFixed(1)}%
              </span>
              <span className="metric-target">Target: &gt; 75%</span>
            </div>
            <div className="metric-card">
              <span className="metric-label">F1 Score</span>
              <span className="metric-value" style={{ color: percentColor(result.f1_score) }}>
                {(result.f1_score * 100).toFixed(1)}%
              </span>
            </div>
            <div className="metric-card">
              <span className="metric-label">False Positive Rate</span>
              <span className="metric-value" style={{ color: result.false_positive_rate < 0.2 ? '#28a745' : '#dc3545' }}>
                {(result.false_positive_rate * 100).toFixed(1)}%
              </span>
            </div>
          </div>
          <div className="eval-detail">
            <span>Total Samples: {result.total_samples}</span>
            <span>True Positives: {result.true_positives}</span>
            <span>False Positives: {result.false_positives}</span>
            <span>False Negatives: {result.false_negatives}</span>
          </div>
        </div>
      )}

      <div className="metrics-section card">
        <div className="metrics-header">
          <h3>Production Metrics</h3>
          <button onClick={handleLoadMetrics} className="btn-secondary">
            Load Metrics
          </button>
        </div>
        {metrics.length > 0 && (
          <table className="data-table">
            <thead>
              <tr>
                <th>Period</th>
                <th>Recall</th>
                <th>Precision</th>
                <th>P50</th>
                <th>P99</th>
                <th>Auto-Fix %</th>
                <th>Adoption</th>
                <th>Reviews</th>
              </tr>
            </thead>
            <tbody>
              {metrics.map((m, i) => (
                <tr key={i}>
                  <td>{m.period_start}</td>
                  <td style={{ color: percentColor(m.recall_rate / 100) }}>{m.recall_rate}%</td>
                  <td style={{ color: percentColor(m.precision_rate / 100) }}>{m.precision_rate}%</td>
                  <td>{(m.p50_review_time_ms / 1000).toFixed(1)}s</td>
                  <td>{(m.p99_review_time_ms / 1000).toFixed(1)}s</td>
                  <td>{m.auto_fix_pass_rate}%</td>
                  <td style={{ color: percentColor(m.adoption_rate / 100) }}>{m.adoption_rate}%</td>
                  <td>{m.total_reviews}</td>
                </tr>
              ))}
            </tbody>
          </table>
        )}
      </div>
    </div>
  )
}

export default Evaluation
