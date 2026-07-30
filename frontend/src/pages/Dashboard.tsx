import React, { useState, useEffect, useCallback } from 'react'
import { useNavigate } from 'react-router-dom'
import { submitReview, listReviews, ReviewDetail, healthCheck } from '../services/api'
import FindingsList from '../components/FindingsList'

const Dashboard: React.FC = () => {
  const navigate = useNavigate()
  const [diffContent, setDiffContent] = useState('')
  const [language, setLanguage] = useState('')
  const [mrId, setMrId] = useState('')
  const [loading, setLoading] = useState(false)
  const [reviews, setReviews] = useState<ReviewDetail[]>([])
  const [health, setHealth] = useState<string>('checking...')

  const fetchReviews = useCallback(async () => {
    try {
      const { data } = await listReviews({ limit: 20 })
      setReviews(data)
    } catch {
      // API not available yet
    }
  }, [])

  useEffect(() => {
    healthCheck()
      .then(({ data }) => setHealth(data.status))
      .catch(() => setHealth('unavailable'))
    fetchReviews()
  }, [fetchReviews])

  const handleSubmit = async () => {
    if (!diffContent.trim()) return
    setLoading(true)
    try {
      const { data } = await submitReview({
        diff_content: diffContent,
        language: language || undefined,
        mr_id: mrId || undefined,
      })
      navigate(`/review/${data.review_id}`)
    } catch (err) {
      console.error('Submit failed:', err)
    } finally {
      setLoading(false)
    }
  }

  const severityColor = (sev: string) => {
    const map: Record<string, string> = {
      critical: '#dc3545', major: '#fd7e14', minor: '#ffc107', info: '#17a2b8',
    }
    return map[sev] || '#6c757d'
  }

  return (
    <div className="dashboard">
      <header className="page-header">
        <h1>Dashboard</h1>
        <span className={`health-badge ${health === 'healthy' ? 'ok' : 'err'}`}>
          API: {health}
        </span>
      </header>

      <div className="review-form card">
        <h3>Submit Code Review</h3>
        <div className="form-row">
          <input
            type="text"
            placeholder="MR/PR ID (optional)"
            value={mrId}
            onChange={(e) => setMrId(e.target.value)}
          />
          <input
            type="text"
            placeholder="Language (auto-detect if empty)"
            value={language}
            onChange={(e) => setLanguage(e.target.value)}
          />
        </div>
        <textarea
          placeholder="Paste unified diff or source code here..."
          value={diffContent}
          onChange={(e) => setDiffContent(e.target.value)}
          rows={12}
          className="diff-input"
        />
        <button
          onClick={handleSubmit}
          disabled={loading || !diffContent.trim()}
          className="btn-primary"
        >
          {loading ? 'Submitting...' : 'Submit Review'}
        </button>
      </div>

      <div className="recent-reviews card">
        <h3>Recent Reviews</h3>
        {reviews.length === 0 ? (
          <p className="empty">No reviews yet. Submit a diff to get started.</p>
        ) : (
          <table className="data-table">
            <thead>
              <tr>
                <th>Review ID</th>
                <th>MR</th>
                <th>Status</th>
                <th>Issues</th>
                <th>Files</th>
                <th>Time</th>
                <th>Created</th>
              </tr>
            </thead>
            <tbody>
              {reviews.map((r) => (
                <tr
                  key={r.review_id}
                  onClick={() => navigate(`/review/${r.review_id}`)}
                  style={{ cursor: 'pointer' }}
                >
                  <td><code>{r.review_id}</code></td>
                  <td>{r.mr_id || '-'}</td>
                  <td>
                    <span className={`status-tag ${r.status}`}>{r.status}</span>
                  </td>
                  <td>
                    <span style={{ color: severityColor('critical') }}>{r.critical_count}</span>
                    {' / '}
                    <span style={{ color: severityColor('major') }}>{r.major_count}</span>
                    {' / '}
                    <span style={{ color: severityColor('minor') }}>{r.minor_count}</span>
                  </td>
                  <td>{r.files_analyzed}</td>
                  <td>{r.analysis_time_ms ? `${(r.analysis_time_ms / 1000).toFixed(1)}s` : '-'}</td>
                  <td>{new Date(r.created_at).toLocaleString()}</td>
                </tr>
              ))}
            </tbody>
          </table>
        )}
      </div>
    </div>
  )
}

export default Dashboard
