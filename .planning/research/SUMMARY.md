# Project Research Summary

**Project:** WTCD v0.2.0 Polyglot Adapters（9 语言扩展）
**Domain:** Rust CLI 代码语义提取与跨语言适配器体系
**Researched:** 2026-03-22
**Confidence:** MEDIUM-HIGH

## Executive Summary

这是一个“以 Rust 单二进制为约束”的多语言代码语义提取系统扩展项目：在不改 CLI 交互模型（`run/check/route`）的前提下，把 Rust、Dart、Java、Kotlin、Swift、C++、C#、C、Zig 接入现有 adapter/registry 管线。研究结论非常一致：行业里这类能力在首版应优先走 **tree-sitter + 统一语义契约**，先保证可扫描、可解析、可提取、可回归，而不是一步跳到编译器级类型求解。

推荐路径是“先打通再加深”：以 `tree-sitter@0.26.7` + 各语言 grammar crate 为核心，沿用现有 `LanguageAdapter` 模式扩展 9 个适配器，稳定产出 `exports/imports/signatures/side_effects + confidence` 四元组，随后修正跨语言 depgraph 归一与聚合噪声。这样能在 v0.2.0 内完成高价值覆盖，同时保持后续向深语义演进的空间。

主要风险不在“能否 parse”，而在“语义错配导致下游误判”：可见性规则跨语言不一致、条件编译导致指纹抖动、宏/生成代码造成静默漏提取、C/C++ 头文件语言歧义。缓解策略是：前置契约（visibility/context/confidence）、language-aware import normalization、generated/uncertain 明确标注、以及每语言全链路 E2E 回归门禁。

## Key Findings

### Recommended Stack

本次技术栈建议高度聚焦于“最小架构改动 + 最大语言覆盖”：保持现有 `wtcd-core::LanguageAdapter + AdapterRegistry` 主干不变，在 workspace 统一管理 tree-sitter runtime 与 9 个语法库版本，避免隐式漂移。对 v0.2.0 来说，这是风险最低且可验证性最强的路径。

**Core technologies:**
- `tree-sitter@0.26.7`：统一增量解析 runtime — 具备显式 ABI 兼容校验，便于可诊断失败。
- `tree-sitter-language@0.1.7`：grammar 与 runtime 的桥接层 — 降低语法库版本耦合风险。
- `LanguageAdapter + AdapterRegistry`（现有）：统一语言接入边界 — 无需重构 CLI 主流程，仅扩展模块与注册。

**关键版本要求（必须锁定）**
- Runtime/Bridge：`tree-sitter@0.26.7`、`tree-sitter-language@0.1.7`
- Grammar：`rust 0.24.1`、`dart 0.1.0`、`java 0.23.5`、`kotlin-ng 1.1.0`、`swift 0.7.1`、`cpp 0.23.4`、`c-sharp 0.23.1`、`c 0.24.1`、`zig 1.1.2`

### Expected Features

v0.2.0 的可交付定义清晰：不是“语义最深”，而是“跨 9 语言的可用一致性”。先把用户默认期望的能力做完整，再把语义等深作为 v0.2.x/0.3+ 的增强路线。

**Must have (table stakes):**
- 扩展名扫描命中 + AdapterRegistry 正确路由。
- AST 解析 + confidence 分级（错误容忍但可降级）。
- 四类事实稳定输出：`exports/imports/signatures/side_effects`。
- 每语言 fixtures + 单测 + `run/check/route` 集成回归。
- CLI/MCP 无新增心智负担（原命令直接可用）。

**Should have (competitive):**
- 跨语言语义归一层（统一 ExportKind 语义映射）。
- 语言特有元信息标签化输出（不污染核心 schema）。
- 能力矩阵（每语言支持项）与 broken-fixture 韧性基准。

**Defer (v2+ / v0.3+):**
- 编译器级类型求解与完整符号解析。
- 宏/模板/预处理完整展开分析。
- 深度 FFI/interop 边精准建模（先规则化起步）。

### Architecture Approach

架构建议是“保守演进而非重构”：新增 9 个 `wtcd-adapters/src/<lang>.rs`，在 `lib.rs` 集中注册；扩展 `wtcd-scope::SUPPORTED_EXTENSIONS`；在 `wtcd-core` 修正跨语言 depgraph（从 TS-only 变为 language-aware）；可选但强烈建议在 `FileResult` 增加 `language` 字段，避免下游猜测。核心模式为 Adapter-per-language + Progressive extraction contract，先保证四元组稳定输出，再逐语言提精度。

**Major components:**
1. `wtcd-scope::scanner` — 负责文件进入管线的扩展名白名单筛选。
2. `wtcd-adapters`（LanguageAdapter 实现集）— 负责每语言 parse 与语义抽取。
3. `wtcd-core::depgraph/index` — 负责增量影响面、路由检索与下游消费一致性。

### Critical Pitfalls

1. **可见性/导出语义被过度统一** — 在适配器层引入语言可见性模型，聚合层只消费归一化导出集合。
2. **条件编译与预处理引发假漂移** — 指纹输入绑定 `analysis_context`，定义单一门禁基准上下文。
3. **宏/生成机制导致静默漏提取** — 区分 `explicit_symbols` 与 `generated_or_uncertain_symbols`，并强制降级置信度。
4. **C/C++ 头文件识别歧义** — 实施“扩展名+邻接编译单元+配置覆盖”三级识别，支持 `language_overrides`。
5. **只测 adapter 不测全链路** — 每语言必须做 `parse→mirror→aggregate→route→drift` E2E 快照回归。

## Implications for Roadmap

Based on research, suggested phase structure:

### Phase 1: 基线与契约冻结（Stack + Contract Baseline）
**Rationale:** 先锁 runtime/grammar/置信度/可见性契约，避免后续并行开发返工。  
**Delivers:** 版本锁（含 parser-lock 策略）、扩展名清单、统一提取契约、错误与置信度规范。  
**Addresses:** Table stakes 的“扫描命中”“confidence 分级”。  
**Avoids:** 可见性错配、parser 漂移、低置信策略缺位。

### Phase 2: 9 语言适配器并行实现（Adapter Batches）
**Rationale:** 依赖已冻结契约后可并行；按语言族分批能降低上下文切换。  
**Delivers:** 9 个新 adapter + 注册接线 + 每语言最小完整 fixture/单测。  
**Uses:** `tree-sitter@0.26.7` 与 9 个 grammar crate。  
**Implements:** `wtcd-adapters` 新模块、`scanner` 白名单扩展。  
**Avoids:** “只加 adapter 不改 scanner”、C/C++ 识别歧义、JVM 包边界失真。

### Phase 3: 跨语言聚合一致性（Depgraph & Aggregation Hardening）
**Rationale:** 适配器可用后，必须先修下游一致性，否则 run/check/route 结果不可信。  
**Delivers:** language-aware import normalization、（推荐）`FileResult.language`、聚合边界修正。  
**Addresses:** Table stakes 的依赖边质量与签名可检索性。  
**Avoids:** TS-only depgraph 误扩散/漏扩散、跨语言误聚合、假漂移。

### Phase 4: Route/Knowledge 语义质量收口（Consumer Reliability）
**Rationale:** 让“可解析”升级为“可决策”，确保 Agent 端读取质量。  
**Delivers:** route 索引增强（language/namespace/confidence 维度）、知识层低置信标注、generated/uncertain 展示。  
**Addresses:** Differentiators（语义归一、元信息标签化、能力矩阵雏形）。  
**Avoids:** 低置信事实被当成确定结论、route 命中面扩大但精度下降。

### Phase 5: CI 门禁与发布稳定化（Regression & Performance Gates）
**Rationale:** 新增 9 语言后，质量问题主要在回归与性能，不在功能点本身。  
**Delivers:** 多语言混仓 E2E 回归矩阵、跨平台漂移阈值、性能预算与超时降级、升级迁移流程。  
**Addresses:** Table stakes 的“可持续迭代”与“不破坏既有链路”。  
**Avoids:** parser 升级全红、噪声漂移、性能断崖、仅 unit 无 E2E。

### Phase Ordering Rationale

- 先“契约和版本”再“并行实现”，是避免 9 语言并发返工的唯一低风险顺序。
- 适配器完成后必须先做 depgraph/聚合修正，再做 route/knowledge，否则下游优化建立在错误输入上。
- 把 CI/性能门禁放在最后收口，但其规范（基准上下文、置信度策略）需在 Phase 1 前置定义。

### Research Flags

Phases likely needing deeper research during planning:
- **Phase 3:** 跨语言 import normalization 规则复杂，需补充语言族路径语义与边界用例研究。
- **Phase 4:** 语义归一与元标签 schema 设计涉及消费者兼容性，建议做一次专门 schema 评审。
- **Phase 5:** 性能预算与跨平台漂移阈值需要真实仓库样本基准，不宜凭经验拍值。

Phases with standard patterns (skip research-phase):
- **Phase 1:** 版本锁定、置信度分层、契约冻结均为成熟工程模式。
- **Phase 2:** Adapter-per-language + fixtures 回归模式在现有 ts/py/go 已被验证。

## Confidence Assessment

| Area | Confidence | Notes |
|------|------------|-------|
| Stack | HIGH | runtime 与核心接入路径有官方文档与现有仓库模式双重支撑；仅个别 grammar 活跃度需编译实证。 |
| Features | MEDIUM-HIGH | P1/P2 边界清晰；竞品外部证据因缺少 Tavily API 未做深检索。 |
| Architecture | HIGH | 基于现有代码结构与明确改动点，依赖关系和构建顺序可执行性强。 |
| Pitfalls | HIGH | 风险点与预防策略具备语言规范与 tree-sitter 官方依据，且与现有 pipeline 痛点高度对齐。 |

**Overall confidence:** MEDIUM-HIGH

### Gaps to Address

- **Kotlin grammar 选型实证缺口：** `tree-sitter-kotlin-ng@1.1.0` 需在仓库 CI 做编译+fixture smoke 才能最终定版。
- **跨平台漂移阈值尚未量化：** 需在 Linux/macOS 基准仓运行样本后确定门禁阈值。
- **C/C++ 头文件判定策略未工程化：** 需明确 `language_overrides` 配置格式与默认优先级。
- **竞品对标证据不足：** 因外部检索受限，后续可补公开案例数据增强 roadmap 说服力。

## Sources

### Primary (HIGH confidence)
- Context7 `/tree-sitter/tree-sitter` — Parser API、language ABI 兼容、ERROR/MISSING 与查询机制
- 官方文档 `https://docs.rs/tree-sitter/latest/tree_sitter/struct.Parser.html` — `set_language`/`parse` 语义
- 官方文档 `https://tree-sitter.github.io/tree-sitter/using-parsers/` — parser 使用基线
- 项目源码：
  - `crates/wtcd-core/src/{adapter.rs,types.rs,depgraph.rs}`
  - `crates/wtcd-adapters/src/{ts.rs,py.rs,go.rs}`
  - `crates/wtcd-scope/src/scanner.rs`
  - `crates/wtcd-cli/src/commands/{run.rs,check.rs}`

### Secondary (MEDIUM confidence)
- crates 元数据与 docs：`tree-sitter-rust/dart/java/kotlin-ng/swift/cpp/c-sharp/c/zig` 版本可用性
- 各语言规范文档（Rust/Dart/Java/Kotlin/Swift/C/C++/C#/Zig）用于 pitfalls 约束校验

### Tertiary (LOW confidence)
- 竞品横向对比的抽象结论（未完成外部检索量化，需后续补证）

---
*Research completed: 2026-03-22*
*Ready for roadmap: yes*
