import React from 'react'
import { Finding } from '../services/api'
import DiffViewer from './DiffViewer'

interface FindingsListProps {
  findings: Finding[]
}

const CATEGORY_COLORS: Record<string, string> = {
  security: '#dc3545', performance: '#fd7e14', logic: '#6f42c1',
  style: '#17a2b8', api_compat: '#20c997', test_quality: '#6c757d',
}

const FindingsList: React.FC<FindingsListProps> = ({ findings }) => {
  const [expandedId, setExpandedId] = React.useState<number | null>(null)

  if (findings.length === 0) {
    return (
      <div className="empty-findings">
        <span className="check-icon">&#10003;</span>
        <p>No issues found!</p>
      </div>
    )
  }

  return (
    <div className="findings-list">
      {findings.map((finding) => (
        <div
          key={finding.id}
          className={`finding-item severity-${finding.severity}`}
        >
          <div
            className="finding-header"
            onClick={() => setExpandedId(expandedId === finding.id ? null : finding.id)}
          >
            <span className="finding-severity">{finding.severity}</span>
            <span
              className="finding-category"
              style={{ backgroundColor: CATEGORY_COLORS[finding.category] || '#6c757d' }}
            >
              {finding.category}
            </span>
            <span className="finding-title">{finding.title}</span>
            {finding.file_path && (
              <span className="finding-location">
                {finding.file_path}:{finding.line_start}
              </span>
            )}
            {finding.llm_confidence !== undefined && (
              <span className="finding-confidence">
                {(finding.llm_confidence * 100).toFixed(0)}%
              </span>
            )}
            <span className="expand-icon">{expandedId === finding.id ? '▲' : '▼'}</span>
          </div>

          {expandedId === finding.id && (
            <div className="finding-body">
              {finding.description && (
                <p className="finding-description">{finding.description}</p>
              )}
              {finding.suggestion && (
                <div className="finding-suggestion">
                  <strong>Suggestion:</strong>
                  <pre>{finding.suggestion}</pre>
                </div>
              )}
              {finding.auto_fix_code && (
                <div className="finding-fix">
                  <strong>Auto-fix:</strong>
                  <pre><code>{finding.auto_fix_code}</code></pre>
                  {finding.auto_fix_applied && (
                    <span className="fix-applied">Applied</span>
                  )}
                </div>
              )}
            </div>
          )}
        </div>
      ))}
    </div>
  )
}

export default FindingsList
