import axios from 'axios'

const API_BASE = '/api/v1'

const api = axios.create({
  baseURL: API_BASE,
  timeout: 30000,
  headers: { 'Content-Type': 'application/json' },
})

export interface ReviewRequest {
  diff_content: string
  language?: string
  mr_id?: string
  repo_url?: string
  rules?: string[]
  max_files?: number
}

export interface Finding {
  id: number
  category: string
  severity: string
  title: string
  description?: string
  file_path?: string
  line_start?: number
  line_end?: number
  suggestion?: string
  auto_fix_code?: string
  auto_fix_applied: boolean
  llm_confidence?: number
}

export interface ReviewDetail {
  review_id: string
  mr_id?: string
  status: string
  total_issues: number
  critical_count: number
  major_count: number
  minor_count: number
  info_count: number
  files_analyzed: number
  analysis_time_ms?: number
  findings: Finding[]
  created_at: string
  completed_at?: string
}

export interface Rule {
  rule_id: string
  name: string
  category: string
  severity: string
  language?: string
  description?: string
  is_enabled: boolean
}

export interface EvaluationResult {
  recall: number
  precision: number
  f1_score: number
  false_positive_rate: number
  total_samples: number
  true_positives: number
  false_positives: number
  false_negatives: number
}

export interface MetricsData {
  period: string
  period_start: string
  recall_rate: number
  precision_rate: number
  false_positive_rate: number
  avg_review_time_ms: number
  p50_review_time_ms: number
  p99_review_time_ms: number
  auto_fix_pass_rate: number
  adoption_rate: number
  total_reviews: number
  total_findings: number
}

export const submitReview = (data: ReviewRequest) =>
  api.post<{ review_id: string; status: string }>('/review', data)

export const getReview = (id: string) =>
  api.get<ReviewDetail>(`/review/${id}`)

export const listReviews = (params?: { mr_id?: string; status?: string; limit?: number }) =>
  api.get<ReviewDetail[]>('/reviews', { params })

export const getRules = (params?: { category?: string; language?: string }) =>
  api.get<Rule[]>('/rules', { params })

export const toggleRule = (ruleId: string, enabled: boolean) =>
  api.post(`/rules/${ruleId}/toggle`, { enabled })

export const runEvaluation = (testsetPath: string, rules?: string[]) =>
  api.post<EvaluationResult>('/evaluate', { testset_path: testsetPath, rules })

export const getMetrics = (period: string = 'weekly') =>
  api.get<MetricsData[]>('/metrics', { params: { period } })

export const healthCheck = () =>
  api.get<{ status: string }>('/health')
