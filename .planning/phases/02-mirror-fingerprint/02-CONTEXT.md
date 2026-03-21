# Phase 2: Mirror + Fingerprint - Context

**Gathered:** 2026-03-21
**Status:** Ready for planning

<domain>
## Phase Boundary

用户运行 `anrsm run` 后，在 `mirror/file/` 目录下生成版本可控的 Markdown 镜像文件，包含 YAML Front Matter 头部和 8 个标准段落。双指纹机制（source_fingerprint 和 semantic_fingerprint）区分内容变更和语义变更。

</domain>

<decisions>
## Implementation Decisions

### Mirror Content Generation
- **D-01:** 镜像正文使用 Rule-based 模板生成 — 将 FileResult 字段映射到模板文本，无需外部 API
- **D-02:** 镜像正文使用结构化模板 — 固定标题 + 事实列表，确定性输出
- **D-03:** 职责描述（Responsibilities）从导出符号 + 文件路径推断（如 `src/auth/login.ts` → "认证：用户登录逻辑"）
- **D-04:** 变更影响（Change Impact）段落基于 import 依赖图推断受影响邻域

### Semantic Fingerprint Scope
- **D-05:** 指纹算法使用 SHA-256 — 与 mirror-header.schema.json 的 `sha256:` 前缀一致
- **D-06:** 语义指纹排除：空白 + 注释 + 尾随分号变化
- **D-07:** fingerprint 的 canonical 序列化使用 serde_json → 字符串 → SHA-256
- **D-08:** fp_version 初始值为 `"1"`，后续算法升级自然递增

### Mirror File Lifecycle
- **D-09:** 镜像文件已存在时全量覆写镜像正文 — 确定性输出，始终反映最新语义
- **D-10:** 保留用户添加的 manual_appendix 内容，其余段落覆写
- **D-11:** 镜像文件默认 git 跟踪 — 镜像文件是版本工件，需要 diff 和 review
- **D-12:** 删除的源文件对应的镜像文件自动删除 — `anrsm run` 时同步清理

### Crate Architecture
- **D-13:** 指纹计算和镜像生成合并为 `wtcd-mirror` crate — MVP 阶段指纹仅被镜像使用，耦合紧密
- **D-14:** 新 crate `wtcd-mirror` — 职责清晰，与 wtcd-core/wtcd-adapters 解耦
- **D-15:** `anrsm run` 内置镜像生成 — 自动执行 解析→指纹→镜像→写文件 全流程
- **D-16:** wtcd-mirror 不直接依赖 wtcd-scope — 接收 `Vec<FileResult>` 即可，scope 由 CLI 层串联

### the agent's Discretion
- 镜像模板的具体段落格式和排版细节
- 指纹归一化的精确实现（空白处理、注释移除的具体规则）
- anrsm.yaml 中 mirror 块的具体配置字段
- manual_appendix 的解析和合并逻辑

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### 镜像协议与格式
- `docs/06_Semantic_Mirror_Protocol_and_Header_Spec.md` — 镜像协议定义、8 标准段落、新鲜度状态机
- `schemas/mirror-header.schema.json` — 镜像头部 JSON Schema（13 个必填字段 + 6 个可选字段）
- `examples/file_mirror_example.md` — 镜像文件示例

### 核心类型与适配器
- `wtcd-core/src/types.rs` — FileResult、ExportedSymbol、DependencyEdge 等核心类型
- `wtcd-core/src/adapter.rs` — LanguageAdapter trait、AdapterRegistry
- `wtcd-core/src/config.rs` — Config、MirrorConfig（Phase 2 将扩展）

### 技术栈
- `.planning/research/STACK.md` — 技术选型理由、版本兼容
- `.planning/research/ARCHITECTURE.md` — 推荐项目结构、架构模式
- `Cargo.toml` — 当前 workspace 依赖和结构

</canonical_refs>

<code_context>
## Existing Code Insights

### 可复用资产
- **wtcd-core crate** — FileResult、ConfidenceBand、ExportedSymbol、DependencyEdge 等类型直接复用
- **wtcd-adapters crate** — TS/JS tree-sitter 解析适配器，输出 FileResult 供镜像生成使用
- **wtcd-scope crate** — 文件扫描，CLI 层串联 scope → adapters → mirror
- **mirror-header.schema.json** — 定义了镜像头部的完整结构（13 必填 + 6 可选字段）
- **file_mirror_example.md** — 提供了镜像输出的参考模板
- **anrsm.yaml** — Phase 1 预留了空的 `mirror: {}` 块

### 已锁定的技术模式
- Cargo workspace 多 crate 结构 — wtcd-mirror 作为新 crate 加入 workspace
- Trait-based 适配器 — 镜像生成器通过 trait 调用解析能力
- Pipeline 架构（Config → Scope → Scan → Parse → Fingerprint → Mirror）
- serde 序列化 — 所有类型已有 #[derive(Serialize, Deserialize)]

### 集成点
- `Cargo.toml` workspace — 添加 `wtcd-mirror` crate
- `wtcd-cli/src/commands/run.rs` — run 命令将串联 scope → adapters → mirror pipeline
- `wtcd-core/src/config.rs` — MirrorConfig 将扩展镜像生成配置

### 注意事项
- 镜像头部 schema 中指定 `sha256:` 前缀，指纹算法必须与之匹配
- mirror-header.schema.json 的 freshness_state 字段有状态机：fresh → stale → fresh
- manual_appendix 段落需要特殊处理 — 用户手动添加的内容不能被覆写
- 镜像路径映射必须确定性：`mirror/file/<source_relative_path>.md`

</code_context>

<specifics>
## Specific Ideas

- Phase 2 不做 LLM 压缩 — 纯规则模板生成，后续可选增强
- fingerprint 使用 blake3 在 ARCHITECTURE.md 中建议，但 schema 指定 sha256，选择保持 schema 一致
- manual_appendix 的处理方式：在镜像文件末尾保留 `<!-- manual_appendix_start -->` 到 `<!-- manual_appendix_end -->` 之间的内容
- anrsm.yaml 的 mirror 块将包含：output_dir、fp_version、include_manual_appendix 等配置

</specifics>

<deferred>
## Deferred Ideas

- LLM-assisted 语义压缩 — spec 说"LLM 只做压缩"，不在 MVP 范围
- 更激进的语义指纹归一化（import 排序、局部变量重命名）— 首版保守处理
- 镜像 diff 工具 — 对比新旧镜像的语义变化
- 镜像质量评分 — 评估镜像覆盖度和准确度

</deferred>

---

*Phase: 02-mirror-fingerprint*
*Context gathered: 2026-03-21*
