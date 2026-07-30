import React, { useState, useEffect } from 'react'
import { getRules, toggleRule, Rule } from '../services/api'

const CATEGORIES = ['security', 'performance', 'logic', 'style', 'api_compat', 'test_quality']
const CATEGORY_COLORS: Record<string, string> = {
  security: '#dc3545', performance: '#fd7e14', logic: '#6f42c1',
  style: '#17a2b8', api_compat: '#20c997', test_quality: '#6c757d',
}
const SEVERITY_COLORS: Record<string, string> = {
  critical: '#dc3545', major: '#fd7e14', minor: '#ffc107', info: '#17a2b8',
}

const RulesManager: React.FC = () => {
  const [rules, setRules] = useState<Rule[]>([])
  const [filterCategory, setFilterCategory] = useState('')
  const [filterLanguage, setFilterLanguage] = useState('')
  const [loading, setLoading] = useState(true)

  const fetchRules = async () => {
    setLoading(true)
    try {
      const { data } = await getRules({
        category: filterCategory || undefined,
        language: filterLanguage || undefined,
      })
      setRules(data)
    } catch (err) {
      console.error('Failed to load rules:', err)
    } finally {
      setLoading(false)
    }
  }

  useEffect(() => {
    fetchRules()
  }, [filterCategory, filterLanguage])

  const handleToggle = async (ruleId: string, enabled: boolean) => {
    try {
      await toggleRule(ruleId, enabled)
      setRules((prev) =>
        prev.map((r) => (r.rule_id === ruleId ? { ...r, is_enabled: enabled } : r))
      )
    } catch (err) {
      console.error('Toggle failed:', err)
    }
  }

  return (
    <div className="rules-manager">
      <header className="page-header">
        <h1>Rules Manager</h1>
        <span className="rule-count">{rules.length} rules</span>
      </header>

      <div className="filters card">
        <select value={filterCategory} onChange={(e) => setFilterCategory(e.target.value)}>
          <option value="">All Categories</option>
          {CATEGORIES.map((c) => (
            <option key={c} value={c}>{c}</option>
          ))}
        </select>
        <select value={filterLanguage} onChange={(e) => setFilterLanguage(e.target.value)}>
          <option value="">All Languages</option>
          <option value="python">Python</option>
          <option value="javascript">JavaScript</option>
          <option value="typescript">TypeScript</option>
          <option value="go">Go</option>
          <option value="java">Java</option>
          <option value="rust">Rust</option>
        </select>
      </div>

      {loading ? (
        <div className="loading">Loading rules...</div>
      ) : (
        <div className="rules-list">
          {rules.map((rule) => (
            <div key={rule.rule_id} className={`rule-card ${rule.is_enabled ? '' : 'disabled'}`}>
              <div className="rule-header">
                <code className="rule-id">{rule.rule_id}</code>
                <span
                  className="category-tag"
                  style={{ backgroundColor: CATEGORY_COLORS[rule.category] || '#6c757d' }}
                >
                  {rule.category}
                </span>
                <span
                  className="severity-tag"
                  style={{ color: SEVERITY_COLORS[rule.severity] || '#6c757d' }}
                >
                  {rule.severity}
                </span>
                <label className="toggle-switch">
                  <input
                    type="checkbox"
                    checked={rule.is_enabled}
                    onChange={(e) => handleToggle(rule.rule_id, e.target.checked)}
                  />
                  <span className="slider" />
                </label>
              </div>
              <h4 className="rule-name">{rule.name}</h4>
              {rule.description && <p className="rule-desc">{rule.description}</p>}
              <div className="rule-meta">
                <span>Language: {rule.language || 'all'}</span>
              </div>
            </div>
          ))}
        </div>
      )}
    </div>
  )
}

export default RulesManager
