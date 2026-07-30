# Code Review Agent API 接口清单

## 基础信息

- **Base URL**: `http://localhost:8080/api/v1`
- **协议**: REST/JSON
- **认证**: 暂不启用（内网部署）

---

## 接口总览

| 方法 | 路径 | 说明 |
|------|------|------|
| GET | `/health` | 健康检查 |
| POST | `/review` | 提交代码审查任务 |
| GET | `/review/{id}` | 获取审查结果详情 |
| GET | `/reviews` | 查询审查记录列表 |
| GET | `/rules` | 获取规则列表 |
| POST | `/rules/{id}/toggle` | 启用/禁用规则 |
| POST | `/evaluate` | 运行离线评估 |
| GET | `/metrics` | 获取性能指标 |

---

## 1. 健康检查

```
GET /api/v1/health
```

**响应示例**:
```json
{
  "status": "healthy",
  "version": "0.1.0",
  "timestamp": "2026-07-30T12:00:00Z"
}
```

---

## 2. 提交代码审查

```
POST /api/v1/review
Content-Type: application/json

{
  "repo_url": "https://github.com/example/repo",
  "diff_content": "diff --git a/main.py b/main.py\n+query = 'SELECT * FROM users WHERE id=' + user_id",
  "language": "python",
  "mr_id": "MR-1234",
  "base_branch": "main",
  "head_branch": "feature/new-api",
  "rules": ["SEC-001", "SEC-003", "PERF-001"],
  "max_files": 500
}
```

**请求参数**:

| 参数 | 类型 | 必填 | 说明 |
|------|------|------|------|
| diff_content | string | 是 | Unified diff 或完整源码 |
| language | string | 否 | 主要语言，自动检测 |
| mr_id | string | 否 | MR/PR 编号 |
| repo_url | string | 否 | 仓库地址 |
| base_branch | string | 否 | 目标分支，默认 main |
| rules | string[] | 否 | 指定启用的规则 ID 列表 |
| max_files | int | 否 | 最大分析文件数，默认 500 |

**响应** (202 Accepted):
```json
{
  "review_id": "a1b2c3d4e5f6",
  "status": "pending",
  "created_at": "2026-07-30T12:00:00Z"
}
```

---

## 3. 获取审查结果

```
GET /api/v1/review/a1b2c3d4e5f6
```

**响应**:
```json
{
  "review_id": "a1b2c3d4e5f6",
  "mr_id": "MR-1234",
  "status": "completed",
  "total_issues": 12,
  "critical_count": 2,
  "major_count": 5,
  "minor_count": 3,
  "info_count": 2,
  "files_analyzed": 15,
  "analysis_time_ms": 8700,
  "findings": [
    {
      "id": 1,
      "category": "security",
      "severity": "critical",
      "title": "[SEC-001] SQL Injection via String Concatenation",
      "description": "SQL query built with string concatenation from user input",
      "file_path": "src/api/users.py",
      "line_start": 42,
      "line_end": 42,
      "suggestion": "Use parameterized queries with placeholders",
      "auto_fix_code": "cursor.execute('SELECT * FROM users WHERE id = ?', (user_id,))",
      "auto_fix_applied": false,
      "llm_confidence": 0.95
    }
  ],
  "created_at": "2026-07-30T12:00:00Z",
  "completed_at": "2026-07-30T12:00:09Z"
}
```

**状态说明**:
- `pending` - 等待处理
- `analyzing` - 分析中
- `completed` - 完成
- `failed` - 失败

---

## 4. 查询审查记录

```
GET /api/v1/reviews?mr_id=MR-1234&status=completed&limit=20
```

| 参数 | 类型 | 默认 | 说明 |
|------|------|------|------|
| mr_id | string | - | 过滤 MR 编号 |
| status | string | - | 过滤状态 |
| limit | int | 20 | 返回数量 (1-100) |

---

## 5. 获取规则列表

```
GET /api/v1/rules?category=security&language=python&enabled_only=true
```

| 参数 | 类型 | 默认 | 说明 |
|------|------|------|------|
| category | string | - | security / performance / logic / style / api_compat / test_quality |
| language | string | - | python / javascript / go / java / rust / all |
| enabled_only | bool | true | 只返回已启用的规则 |

**响应**:
```json
[
  {
    "rule_id": "SEC-001",
    "name": "SQL Injection via String Concatenation",
    "category": "security",
    "severity": "critical",
    "language": "all",
    "description": "Detects SQL queries built with string concatenation",
    "is_enabled": true
  }
]
```

---

## 6. 启用/禁用规则

```
POST /api/v1/rules/SEC-001/toggle
Content-Type: application/json

{
  "enabled": false
}
```

---

## 7. 运行离线评估

```
POST /api/v1/evaluate
Content-Type: application/json

{
  "testset_path": "./tests/fixtures/eval_dataset.json",
  "rules": ["SEC-001", "SEC-003"]
}
```

**响应**:
```json
{
  "recall": 0.763,
  "precision": 0.815,
  "f1_score": 0.788,
  "false_positive_rate": 0.185,
  "total_samples": 800,
  "true_positives": 610,
  "false_positives": 138,
  "false_negatives": 190
}
```

---

## 8. 获取性能指标

```
GET /api/v1/metrics?period=weekly
```

| 参数 | 类型 | 默认 | 说明 |
|------|------|------|------|
| period | string | weekly | daily / weekly / monthly |

**响应**:
```json
[
  {
    "period": "weekly",
    "period_start": "2026-07-24",
    "recall_rate": 76.3,
    "precision_rate": 81.5,
    "false_positive_rate": 18.5,
    "avg_review_time_ms": 8700,
    "p50_review_time_ms": 8700,
    "p99_review_time_ms": 21300,
    "auto_fix_pass_rate": 65.2,
    "adoption_rate": 72.1,
    "total_reviews": 150,
    "total_findings": 1800
  }
]
```
