# Phase 11: Route & Knowledge Signal Quality - Context

**Gathered:** 2026-03-22
**Status:** Ready for planning

<domain>
## Phase Boundary

用户可在 route/knowledge 输出中直接判断语言覆盖边界与置信度风险，降低误命中。

</domain>

<decisions>
## Implementation Decisions

### Route Confidence Filtering
- 过滤 ConfidenceBand::None 的文件（不参与路由）
- ConfidenceBand::Low 文件降权参与路由（不优先推荐）
- Full extraction 语言的文件在路由评分中获得加权

### Knowledge Layer Language Matrix
- 在 KnowledgeResult 中添加 language_matrix 字段
- 生成 markdown 表格展示每种语言的提取能力
- 在知识文档中添加 low_confidence_modules 区域

</decisions>

<code_context>
## Existing Code Insights

### Reusable Assets
- `query_route()` in `crates/wtcd-core/src/index.rs` — 路由查询引擎，需要添加置信度过滤
- `KnowledgeResult` in `crates/wtcd-core/src/types.rs` — 需要添加 language_matrix 字段
- `build_knowledge_result()` in `crates/wtcd-mirror/src/knowledge.rs` — 需要计算语言矩阵
- `generate_repo_overview()` in same file — 需要添加语言覆盖信息

### Established Patterns
- 路由查询返回 Vec<RouteMatch>，按 score 排序
- 知识层输出 markdown 格式
- ModuleResult 已有 confidence 字段（Phase 10 添加）

### Integration Points
- `crates/wtcd-core/src/types.rs` — KnowledgeResult 结构变更
- `crates/wtcd-core/src/index.rs` — query_route() 添加置信度过滤
- `crates/wtcd-mirror/src/knowledge.rs` — 语言矩阵生成

</code_context>

<specifics>
## Specific Ideas

- 语言矩阵包含：语言名、支持的提取项（exports/imports/signatures/side_effects）、置信度分布
- 路由评分中 confidence=High 加 1.0 权重，Low 加 0.5，None 不参与

</specifics>

<deferred>
## Deferred Ideas

- 基于语言覆盖率的动态路由策略选择 — 未来需求
- 交互式语言能力探索 — 未来需求

</deferred>
