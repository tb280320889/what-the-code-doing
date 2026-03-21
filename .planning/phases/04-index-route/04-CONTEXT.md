# Phase 4: Index + Route - Context

**Gathered:** 2026-03-21
**Status:** Ready for planning

<domain>
## Phase Boundary

Agent 和开发者通过 `anrsm route "任务描述"` 查询镜像系统，获取排序后的候选文件列表（含相关性分数和 freshness 状态）。路由索引在 `anrsm run` 时自动生成，存储为 JSON 文件。

</domain>

<decisions>
## Implementation Decisions

### Index Structure & Storage
- **D-01:** 路由索引存储为 JSON 文件 `mirror/routing_index.json` — 与镜像文件同级，可 diff
- **D-02:** 索引格式为扁平 JSON 数组 — 每个条目 = artifact_id + module_id + source_path + exports[] + keywords[] + freshness + confidence
- **D-03:** `anrsm run` 时自动生成路由索引 — 解析完所有文件后构建并写入
- **D-04:** 只有有镜像的文件入索引 — 保证 freshness 信号可用

### Relevance Scoring & Ranking
- **D-05:** 查询匹配算法为关键词匹配 — 将 query 分词，匹配 exports + source_path + module_id 中的关键词
- **D-06:** freshness 加权 — fresh=1.0, stale=0.7, invalid=0.3, unknown=0.5
- **D-07:** 相关性分数 = 命中关键词数 / query 词数 × freshness 权重 — 简单可解释
- **D-08:** 多文件同分按 confidence_band 降序，再按 source_path 字母序

### Index Data Sources
- **D-09:** 关键词从 source_path 分词 + exports 名称 + module_id 三个维度提取
- **D-10:** risk_tags 入索引 — 可用于过滤（如只查 external_api 相关文件）
- **D-11:** 依赖图入索引 — 存储 dependencies 列表，支持双向查询
- **D-12:** 从 FileResult 构建索引（数据最全），写入 mirror headers 后同步更新索引

### CLI Integration
- **D-13:** `anrsm route` 直接读取磁盘上的 routing_index.json — 无需重新扫描/解析
- **D-14:** `anrsm run` 完成后自动重建 routing_index.json
- **D-15:** 索引不存在时报错 "请先运行 anrsm run" — 与配置不存在时处理一致
- **D-16:** route 命令输出 JSON 格式 — `{results: [{source_path, artifact_id, relevance, freshness, exports}]}`

### the agent's Discretion
- 关键词分词的具体算法（空格/驼峰/下划线分割）
- routing_index.json 的精确 schema 字段定义
- anrsm route 命令的 CLI 参数细节
- relevance 分数的归一化方式

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### 镜像类型
- `crates/wtcd-mirror/src/types.rs` — MirrorHeader（exports, dependencies, module_id, freshness_state）
- `crates/wtcd-mirror/src/io.rs` — read_mirror_header() 读取已有镜像头

### 核心类型
- `crates/wtcd-core/src/types.rs` — FileResult, ExportedSymbol, DependencyEdge
- `crates/wtcd-core/src/depgraph.rs` — ReverseDepGraph（依赖图）

### CLI
- `crates/wtcd-cli/src/main.rs` — 命令注册模式（clap derive）
- `crates/wtcd-cli/src/commands/run.rs` — run 命令 pipeline（索引生成将集成于此）

### 技术栈
- `Cargo.toml` — workspace 依赖

</canonical_refs>

<code_context>
## Existing Code Insights

### 可复用资产
- **MirrorHeader** — exports, dependencies, module_id, freshness_state, artifact_id, source_path 全部已有
- **FileResult** — exports (ExportedSymbol), imports (DependencyEdge), signatures, side_effects
- **ReverseDepGraph** — 已有 reverse dependency graph 实现（Phase 3 构建）
- **wtcd-mirror::io** — read_mirror_header() 可读取已有镜像头
- **wtcd-mirror::template** — derive_risk_tags() 已实现副作用→风险标签映射

### 已锁定的技术模式
- Cargo workspace 多 crate 结构 — 可新增 wtcd-index 或扩展现有 crate
- JSON 结构化输出 — 所有 CLI 命令统一 JSON 格式
- clap derive 命令注册 — Route { query: String, top_k: Option<usize> }
- anrsm.yaml 配置驱动 — 索引配置可放在现有配置结构中

### 集成点
- `wtcd-cli/src/main.rs` — 新增 Route 子命令
- `wtcd-cli/src/commands/run.rs` — run 完成后自动生成索引
- `crates/wtcd-mirror/src/` — 索引构建逻辑可放在这里（或新 crate）

### 注意事项
- 索引构建需要所有 FileResult 数据 — 在 run pipeline 的最后一步执行
- 关键词分词需要处理驼峰命名 (camelCase) 和下划线命名 (snake_case)
- 索引文件需要版本字段（api_version: "1"）以支持未来格式变更
- route 命令需要读取 anrsm.yaml 确定 mirror 目录路径

</code_context>

<specifics>
## Specific Ideas

- routing_index.json 的每个条目包含: artifact_id, module_id, source_path, exports[], keywords[], dependencies[], risk_tags[], freshness, confidence
- 关键词提取: source_path 按 `/` 分割 + exports 直接使用 + module_id 直接使用
- relevance score = (query_tokens ∩ entry_keywords).len() / query_tokens.len() × freshness_weight
- anrsm route 的 top_k 默认值为 10

</specifics>

<deferred>
## Deferred Ideas

- 语义向量搜索 — 需要 embedding 模型，首版不做
- TF-IDF 算法 — 首版用简单关键词匹配
- 索引持久化为 SQLite — 首版用 JSON
- 实时索引更新（watch mode）— 首版 run 时全量重建

</deferred>

---

*Phase: 04-index-route*
*Context gathered: 2026-03-21*
