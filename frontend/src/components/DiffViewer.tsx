import React from 'react'

interface DiffViewerProps {
  oldCode?: string
  newCode?: string
  language?: string
}

const DiffViewer: React.FC<DiffViewerProps> = ({ oldCode, newCode, language }) => {
  if (!oldCode && !newCode) {
    return <div className="diff-viewer empty">No code to display</div>
  }

  // Simple line-by-line diff visualization
  const oldLines = (oldCode || '').split('\n')
  const newLines = (newCode || '').split('\n')
  const maxLines = Math.max(oldLines.length, newLines.length)

  return (
    <div className="diff-viewer">
      <div className="diff-pane">
        <div className="diff-pane-header">Original</div>
        {Array.from({ length: maxLines }).map((_, i) => {
          const oldLine = oldLines[i]
          const newLine = newLines[i]
          const isRemoved = oldLine !== undefined && oldLine !== newLine
          const isAdded = newLine !== undefined && oldLine !== newLine

          return (
            <div key={i} className="diff-row">
              <span className="line-number">{i + 1}</span>
              <span
                className={`line-content ${isRemoved ? 'removed' : ''} ${isAdded && !oldLine ? 'empty' : ''}`}
              >
                {oldLine || ''}
              </span>
            </div>
          )
        })}
      </div>
      <div className="diff-pane">
        <div className="diff-pane-header">Changed</div>
        {Array.from({ length: maxLines }).map((_, i) => {
          const oldLine = oldLines[i]
          const newLine = newLines[i]
          const isAdded = newLine !== undefined && newLine !== oldLine

          return (
            <div key={i} className="diff-row">
              <span className="line-number">{i + 1}</span>
              <span className={`line-content ${isAdded ? 'added' : ''}`}>
                {newLines[i] || ''}
              </span>
            </div>
          )
        })}
      </div>
    </div>
  )
}

export default DiffViewer
