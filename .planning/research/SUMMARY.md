# Project Research Summary

**Project:** ANRSM — AI Native Repo Semantic Mirror
**Domain:** Rust CLI — 代码语义镜像系统（Code Semantic Mirror System）
**Researched:** 2026-03-21
**Confidence:** HIGH

## Executive Summary

ANRSM 是一个 Rust CLI 工具，为 AI Agent 生成与源码一一映射的持久化语义镜像。它填补了一个结构性空白：现有工具要么面向人类（Doxygen/TypeDoc），要么不持久化中间层（Cursor/Copilot），要么聚焦搜索而非可验证的语义新鲜度（Sourcegraph/CocoIndex）。ANRSM 的核心差异化在于——**语义指纹 + 漂移检测**让"文档是否过期"变成可验证的数学问题，而非主观判断。

技术栈已收敛到高信心方案：**clap 4.6**（CLI 事实标准）+ **tree-sitter 0.26**（多语言统一解析框架，容错/增量解析）+ **gix 0.80**（纯 Rust Git 操作，无 C 依赖）+ **serde 生态**（序列化层）。架构遵循控制面/数据面分离 + Trait 驱动语言适配器 + 流水线模式，以 workspace crates 组织代码。参考项目 codegraph 和 NekoCode 验证了 Rust + Tree-sitter 多语言代码分析的可行性。

最大的风险不在技术选型，而在**价值闭环验证**：Agent 是否真的会改变读取行为、门禁误报率能否控制在可接受范围、指纹跨版本是否稳定。研究明确识别了 6 个 Critical Pitfall，其中"C1 镜像沦为装饰"和"C2 门禁误报导致绕过"是最可能杀死项目的两个失败模式。预防策略已到位：强制读取顺序、warn-only 初期模式、指纹版本化协议、golden test suite。

## Key Findings

### Recommended Stack

详见 [STACK.md](./STACK.md)。

**核心技术：**
- **clap 4.6**: CLI 参数解析与子命令 — Rust CLI 事实标准，722M+ 下载，Derive API 声明式定义，自动生成 help/shell completions
- **tree-sitter 0.26**: 多语言统一解析（CST）— 多语言适配器架构的基础，容错解析处理不完整源码，无 serde 序列化开销
- **gix 0.80**: 纯 Rust Git 操作 — 无 C/OpenSSL 依赖，跨平台编译简单，读操作已 production-grade（7.4M+ 下载）
- **serde + serde_json + yaml_serde 0.10**: 序列化统一层 — yaml_serde 由 YAML 官方组织维护，serde_yaml 已停止维护
- **sha2 0.10**: SHA-256 语义指纹计算 — RustCrypto 稳定 API，支持增量 hashing

**关键决策：tree-sitter > SWC > Oxc**
- tree-sitter 胜出原因：多语言统一 API、容错解析、无序列化开销、生态验证（Neovim/Helix/GitHub/ast-grep）
- SWC 作为备选（若 tree-sitter 提取不足时切换）
- Oxc 关注中（2026 H2 重新评估，MSRV 1.92 过高，API 不稳定）

### Expected Features

详见 [FEATURES.md](./FEATURES.md)。

**Must have（基线特性，缺失 = 用户离开）：**
- AST 级代码解析 — 所有竞品都有
- 符号提取与依赖图 — Sourcegraph/Cursor/CocoIndex 均提供符号级索引
- 多语言支持架构 — v1 聚焦 TS/JS，架构必须支持插件化适配器
- 增量更新 — 基于 git diff 的变更集驱动，全量重建会杀死性能
- CLI 可用性 + 配置文件驱动 — CLI init/generate/diff/check 子命令
- JSON 结构化输出 — Agent 生态期望结构化数据

**Should have（差异化特性，定义品类）：**
- 语义镜像持久化层 — 与源码一一映射的可版本管理工件（竞品无此能力）
- 语义指纹 + 漂移检测 — C0/C1/C2/C3 四级语义变化分类（核心差异点）
- Agent 读取顺序契约 — 强制"先路由 → 镜像 → 源码"行为约束
- Freshness State 机器 — fresh/stale/invalid/unknown 四态
- 两段式生成流水线 — 结构化提取 → 语义压缩（公理 A4）
- 路由索引 — 预计算任务→模块映射，vs Sourcegraph 实时查询

**Defer（v2+）：**
- Web UI / Dashboard — v1 纯 CLI
- 多仓库全局图 — v1 聚焦单仓
- 向量搜索 / 嵌入 — 确定性路由优于概率搜索
- MCP Server — CLI + JSON 更通用
- 实时代码监控 — 增量基于 git diff 而非文件系统 watcher

### Architecture Approach

详见 [ARCHITECTURE.md](./ARCHITECTURE.md)。

架构采用**控制面/数据面分离 + Trait 驱动语言适配器 + 流水线模式**。参考项目 codegraph 验证了相同技术组合的可行性。

**主要组件（Cargo workspace crates）：**
1. **anrsm-core** — 核心类型、LanguageAdapter trait、语义模型定义、配置结构
2. **anrsm-scope** — Scope Manager（受管文件枚举）+ Repo Scanner（Git 集成）
3. **anrsm-adapters** — 语言适配器集合（TS 适配器 + 正则回退），编译时注册，不使用动态加载
4. **anrsm-fingerprint** — 双指纹引擎（source_fingerprint + semantic_fingerprint）
5. **anrsm-mirror** — 文件镜像 + 模块镜像生成器（模板驱动）
6. **anrsm-index** — 路由/符号/依赖索引构建
7. **anrsm-drift** — 漂移分析（C0-C3 分级）+ 漂移报告
8. **anrsm-cli** — 二进制入口，纯命令路由和输出格式化

**关键架构模式：**
- Trait-Based Adapter：编译时类型安全，无 ABI 风险，v1 不做动态加载
- Pipeline Architecture：全量 init 流 + 增量 update 流，各阶段可独立测试
- Semantic Fingerprinting：双指纹区分格式化改动 vs 语义改动

### Critical Pitfalls

详见 [PITFALLS.md](./PITFALLS.md)。以下是最可能杀死项目的 5 个关键陷阱：

1. **C1: 镜像沦为装饰 — Agent 行为未改变** — 路由索引无效或 Agent 工作流没有"先读镜像"步骤导致系统价值归零。预防：CLI 输出包含 `read_order` 字段，Agent 集成强制先读镜像，dogfood 阶段监控实际读取路径。
2. **C2: 门禁误报过高导致团队绕过** — 语义指纹对格式过于敏感，C0 噪声阻塞开发者。预防：指纹 normalize whitespace，C0 不阻断门禁，初期 warn-only 模式至少 2 周，建立失败样本回归集。
3. **C3: AST 解析的隐藏地雷** — tree-sitter 增量解析可能产生不稳定结果（ERROR 节点、无限循环、GLR 丢弃正确分支）。预防：指纹基于全量解析结果，建立 golden test suite，解析超时机制，失败文件显式输出 low confidence。
4. **C4: 指纹跨版本/跨环境不稳定** — 工具升级后大面积假漂移。预防：指纹包含 `fp_version` 版本号，normalize 行尾符/编码，提供 `anrsm migrate` 命令。
5. **C5: 镜像变成第二真相源** — 团队先改镜像再改代码。预防：门禁阻断"只改镜像不改源码"的提交，知识层必须从镜像编译。

## Implications for Roadmap

基于四份研究文件的交叉验证，建议 6 个阶段，与 ARCHITECTURE.md 的 Build Order 高度一致：

### Phase 1: Foundation（核心类型 + 解析基础）
**Rationale:** 所有下游组件依赖核心类型。必须先冻结类型系统再做任何实现。这也是 Pitfall M2（无结构化提取）的预防节点——确保两段式流水线的结构化提取基础到位。
**Delivers:** anrsm-core crate（类型、trait、配置）+ anrsm-adapters（TS 适配器原型）
**Addresses:** 基线特性中的 AST 解析、配置文件驱动
**Avoids:** C3 AST 解析地雷（在此阶段建立 golden test suite）、M2 无结构化提取、m2 适配器过度工程化
**Uses:** clap 4.6, tree-sitter 0.26, serde, yaml_serde 0.10

### Phase 2: Scoping + Parsing Pipeline（范围 + 解析流水线）
**Rationale:** 验证从配置到解析结果的完整链路。Scope Manager 是性能的第一道防线（Pitfall C6/M3）。
**Delivers:** anrsm-scope（Scope Manager + Repo Scanner）+ Semantic Extractor（最小版本）
**Addresses:** 符号提取、依赖图、增量更新基础
**Avoids:** C6 全量重建杀死性能（增量从 M1 默认）、M3 等强度镜像（Scope 优先级分层）
**Uses:** gix 0.80, ignore 0.4

### Phase 3: Mirror Generation + Fingerprint（镜像生成 + 指纹）
**Rationale:** 这是用户/Agent 看到的第一个有形产出。指纹是漂移检测的前提。
**Delivers:** Fingerprint Engine（双指纹）+ Mirror Generator（文件 + 模块）+ `anrsm init` / `anrsm build`
**Addresses:** 语义镜像持久化层、文件级镜像生成、机器可读文档头
**Avoids:** C2 门禁误报（指纹 normalize）、C4 指纹不稳定（fp_version 协议）、M1 空话镜像（严格模板 schema）
**Uses:** sha2 0.10, pulldown-cmark 0.13, tempfile 3
**Research flag:** 语义指纹算法的具体设计需要深入研究（C1 vs C2 阈值如何定？）

### Phase 4: Drift + Gate（漂移检测 + CI 门禁）
**Rationale:** 需要先有基线镜像和指纹才能检测漂移。门禁是价值闭环的关键——没有门禁，镜像就是过时的文档。
**Delivers:** Drift Analyzer（C0-C3 分级）+ Policy Gate + `anrsm update` / `anrsm check`
**Addresses:** 语义指纹 + 漂移检测、Material Change 分类、CI/CD 集成
**Avoids:** C2 门禁误报（warn-only 初期模式）、C5 第二真相源（门禁阻断镜像无源码变更）、A2 额外负担
**Uses:** colored 2（人类友好输出）、tracing（诊断日志）
**Research flag:** 需要实际测试仓库验证 C0/C1/C2 分类边界

### Phase 5: Index + Query（索引 + Agent 查询）
**Rationale:** 索引是镜像的派生物，依赖镜像存在。Agent 集成是价值闭环的验证点。
**Delivers:** Index Builder（路由/符号/依赖）+ `anrsm query`（Agent 读取接口）
**Addresses:** 路由索引、Agent 读取顺序契约、压缩效率指标
**Avoids:** C1 镜像沦为装饰（强制读取顺序）、M5 忽略展开源码（`expand_condition` 字段）
**Research flag:** Agent 实际使用 ANRSM 的行为数据验证（是否真的改变了读取顺序？）

### Phase 6: Knowledge + Polish（知识层 + 打磨）
**Rationale:** 知识层是最高层级的派生，依赖所有下层。推迟到此阶段避免 Pitfall M4（知识层提前平台化）。
**Delivers:** Knowledge Compiler + 更多语言适配器 + CLI 输出优化 + JSON Schema 稳定化
**Addresses:** 人类知识层编译、多语言适配器扩展
**Avoids:** M4 知识层提前平台化（严格阶段顺序保证）
**Research flag:** 第二个试点仓库验证（Pitfall m5: dogfood 仓库太特殊）

### Phase Ordering Rationale

- **依赖驱动：** AST 解析 → 符号提取 → 语义指纹 → 漂移检测 → 门禁 → 索引 → Agent 查询 → 知识层。这是 FEATURES.md 中明确的关键路径。
- **风险递增：** Phase 1-3 处理技术风险（解析稳定性、指纹算法），Phase 4-5 处理产品风险（门禁误报率、Agent 行为改变），Phase 6 处理扩展风险。
- **价值递进：** Phase 3 结束时已有可运行的镜像生成，Phase 4 结束时已有 CI 集成，Phase 5 结束时 Agent 可直接使用。
- **Pitfall 防御：** 每个阶段的前置条件都对应了关键陷阱的预防窗口——例如指纹版本化协议必须在 Phase 3 冻结（C4），门禁必须先 warn-only（C2）。

### Research Flags

Phases likely needing deeper research during planning:
- **Phase 3:** 语义指纹算法的具体设计（C1 vs C2 阈值、归一化规则）— 需要对真实 TS/JS 仓库做 PoC
- **Phase 4:** C0/C1/C2 分类边界的实际验证 — 需要真实 diff 数据集
- **Phase 5:** Agent 行为变更验证 — 需要实际使用数据，当前是推断性的

Phases with standard patterns（skip research-phase）:
- **Phase 1:** 核心类型定义 + CLI 框架 — 标准 Rust workspace 实践，clap derive API 文档完善
- **Phase 2:** 文件遍历 + Git 集成 — ignore crate + gix API 文档成熟

## Confidence Assessment

| Area | Confidence | Notes |
|------|------------|-------|
| Stack | HIGH | 基于 Context7 官方文档 + crates.io 版本验证 + 多源交叉验证。tree-sitter/gix/clap 选型有明确的技术论据和竞品参考 |
| Features | HIGH | 竞品功能基于 2026 年多来源交叉验证。差异化判断有设计文档直接支撑。MVP 分阶段建议务实 |
| Architecture | HIGH | 控制面/数据面分离 + Trait Adapter + Pipeline 是 Rust 社区标准实践。参考项目 codegraph/NekoCode 验证了相同技术组合。组件边界清晰 |
| Pitfalls | HIGH | 内部反模式文档 + 外部失败模式研究高度一致。6 个 Critical Pitfall 有明确的预警信号和预防策略 |

**Overall confidence:** HIGH

### Gaps to Address

- **AST 提取效果验证：** 需要对真实 TS/JS 仓库做 PoC，验证 tree-sitter CST 提取能否满足精确的类型签名/泛型信息需求。若不足，需要切换到 SWC（已在 STACK.md 中规划）。
- **语义指纹算法设计：** C0/C1/C2/C3 的阈值和归一化规则需要实际数据支撑，当前是推断性的。Phase 3 需要深入研究。
- **Agent 行为变更数据：** "读取顺序契约"是否真的改变 Agent 行为没有实证数据。Phase 5 需要 dogfooding 验证。
- **压缩效率基准：** 镜像 vs 源码的 Token 比例在真实场景中是多少？当前没有量化数据。
- **MSRV 统一：** clap 4.6 MSRV 1.85, jsonschema 0.45 MSRV 1.83, gix 0.80 MSRV 1.82。需要在 Phase 1 统一解决。

## Sources

### Primary (HIGH confidence)
- Context7: `/websites/rs_clap` — clap 4.x derive API, features
- Context7: `/websites/rs_tree-sitter` — tree-sitter Rust bindings, parser usage
- Context7: `/websites/rs_gix` — gix API, feature flags, git2 migration guide
- crates.io 直接版本验证：yaml_serde 0.10, gix 0.80, sha2 0.10, jsonschema 0.45, tree-sitter 0.26, tree-sitter-typescript 0.23
- 项目规范文档：04_Architecture_and_Component_Model.md, 05_Information_Model_and_Repository_Layout.md, 07_Drift_Detection_Synchronization_and_CI.md, 11_AntiPatterns_FailureModes_and_Decision_Record_Guide.md

### Secondary (MEDIUM confidence)
- [codegraph](https://github.com/optave/codegraph) — Rust + Tree-sitter 多语言代码分析参考实现
- [NekoCode](https://github.com/moe-charm/nekocode-rust) — Rust + Tree-sitter 多语言分析
- [ZeroClaw trait-driven architecture](https://zeroclaws.io/blog/trait-driven-architecture-extensible-agents/) — 编译时 trait 方案 vs 动态插件安全对比
- WebSearch: "gix vs git2 2025", "SWC vs tree-sitter", "Oxc vs SWC 2026"
- GitHub: nushell/nushell#14985 (serde_yml removal), tree-sitter issues #4001, #322, #3243
- CocoIndex Code, Swimm Deep Index — 竞品功能验证

### Tertiary (LOW confidence)
- Agent 行为改变的推断性判断 — 没有实证数据，需要 Phase 5 验证
- 压缩效率的量化 — 当前是理论推算，需要真实场景基准测试

---
*Research completed: 2026-03-21*
*Ready for roadmap: yes*
