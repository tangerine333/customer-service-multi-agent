import React, { useState, useEffect } from 'react'
import { useParams, useNavigate } from 'react-router-dom'
import { getReview, ReviewDetail, Finding } from '../services/api'
import FindingsList from '../components/FindingsList'

const ReviewDetailPage: React.FC = () => {
  const { id } = useParams<{ id: string }>()
  const navigate = useNavigate()
  const [review, setReview] = useState<ReviewDetail | null>(null)
  const [loading, setLoading] = useState(true)
  const [error, setError] = useState('')

  useEffect(() => {
    if (!id) return
    let timer: ReturnType<typeof setInterval>

    const fetch = async () => {
      try {
        const { data } = await getReview(id)
        setReview(data)
        if (data.status === 'completed' || data.status === 'failed') {
          clearInterval(timer)
        }
      } catch (err: any) {
        setError(err?.response?.data?.detail || 'Failed to load review')
        clearInterval(timer)
      } finally {
        setLoading(false)
      }
    }

    fetch()
    timer = setInterval(fetch, 2000)

    return () => clearInterval(timer)
  }, [id])

  if (loading) {
    return <div className="loading">Loading review {id}...</div>
  }

  if (error) {
    return <div className="error-card card"><h3>Error</h3><p>{error}</p></div>
  }

  if (!review) {
    return <div className="error-card card"><h3>Not Found</h3><p>Review not found.</p></div>
  }

  const severityBreakdown = {
    critical: review.critical_count,
    major: review.major_count,
    minor: review.minor_count,
    info: review.info_count,
  }

  return (
    <div className="review-detail">
      <button onClick={() => navigate('/')} className="btn-back">&larr; Back to Dashboard</button>

      <header className="page-header">
        <h1>Review {review.review_id}</h1>
        <span className={`status-tag large ${review.status}`}>{review.status}</span>
      </header>

      <div className="review-summary card">
        <div className="summary-grid">
          <div className="summary-item">
            <span className="label">MR</span>
            <span className="value">{review.mr_id || '-'}</span>
          </div>
          <div className="summary-item">
            <span className="label">Files Analyzed</span>
            <span className="value">{review.files_analyzed}</span>
          </div>
          <div className="summary-item">
            <span className="label">Total Issues</span>
            <span className="value highlight">{review.total_issues}</span>
          </div>
          <div className="summary-item">
            <span className="label">Analysis Time</span>
            <span className="value">
              {review.analysis_time_ms
                ? `${(review.analysis_time_ms / 1000).toFixed(2)}s`
                : '-'}
            </span>
          </div>
        </div>
        <div className="severity-bars">
          {Object.entries(severityBreakdown).map(([sev, count]) => (
            <div key={sev} className="severity-bar">
              <span className={`sev-label ${sev}`}>{sev}</span>
              <div className="bar-track">
                <div
                  className={`bar-fill ${sev}`}
                  style={{ width: `${review.total_issues ? (count / review.total_issues) * 100 : 0}%` }}
                />
              </div>
              <span className="sev-count">{count}</span>
            </div>
          ))}
        </div>
      </div>

      <div className="findings-section card">
        <h3>Findings ({review.findings.length})</h3>
        <FindingsList findings={review.findings} />
      </div>
    </div>
  )
}

export default ReviewDetailPage
