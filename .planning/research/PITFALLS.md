# Domain Pitfalls — v0.2.0 新增 9 语言适配器

**Domain:** 在现有 WTCD 管线中新增 Rust / Dart / Java / Kotlin / Swift / C++ / C# / C / Zig 适配器  
**Researched:** 2026-03-22  
**Confidence:** HIGH（项目上下文 + 官方语言文档 + tree-sitter 官方文档）

---

## 关键结论（先说结论）

这次里程碑最容易失败的点，不是“能不能解析语法”，而是**把 9 种语言硬塞进同一条语义契约时出现的语义错配**：

1. 语言可见性/导出规则不一致，导致聚合层“看起来有数据、其实不可信”。
2. 条件编译/预处理/宏系统导致“同一文件在不同环境下语义不同”，指纹和漂移会抖动。
3. 生成代码与分部声明（C# partial、宏展开、comptime）让符号归属不稳定，最终污染知识层和 route。

下面的坑全部聚焦“新增适配器接入现有 pipeline”场景，并给出**预警信号 / 预防策略 / 应该在哪个 roadmap phase 处理**。

---

## 建议里程碑 Phase 划分（供 PITFALL 映射）

- **Phase P1 — 适配器契约与版本基线**：统一 AST/语义抽取契约、tree-sitter 版本策略、置信度策略。
- **Phase P2 — 语言批次实现**：按语言族实现与 fixture（Rust+Zig、JVM、Apple、C-family、Dart）。
- **Phase P3 — 跨语言聚合与指纹稳定**：模块聚合、符号身份、指纹稳定性、漂移降噪。
- **Phase P4 — route/knowledge 集成**：读取顺序、知识层摘要、低置信降级与人工复核入口。
- **Phase P5 — CI 门禁与回归矩阵**：跨平台编译、回归语料、性能预算、门禁策略。

---

## Critical Pitfalls

### Pitfall 1: 可见性/导出语义被“统一简化”，导致跨语言错误聚合

**What goes wrong:** 适配器为了统一接口，把不同语言的导出规则压扁成同一套“public=exported”规则，导致导出索引和依赖图错误。  
典型错配：Go 大写导出、Java package/module 可见性、Kotlin `internal`、C# partial 合并后可见性、Rust `pub(crate)`、Dart 库私有 `_`、Swift 访问控制。

**Why it happens:** 过早做语言无关抽象，忽略每种语言的“可见性判定上下文”。

**How to avoid:**
- 在 `LanguageAdapter` 层新增 `visibility_model` 与 `effective_export_set()`（按语言实现）。
- 聚合层只消费“已归一化后的导出集合”，不直接猜可见性。
- 每语言至少 15 个“可见性边界 fixture”（含 internal/package/private/partial 场景）。

**Warning signs:**
- 导出符号数异常飙升（尤其 Java/C# 项目）。
- route 命中候选文件明显增多但命中精度下降。
- 同一仓库换语言过滤后，导出统计无法解释。

**Phase to address:** **P1 定义契约，P2 实现，P3 校验**。

---

### Pitfall 2: 条件编译与预处理导致指纹抖动（假漂移）

**What goes wrong:** 同一提交在不同平台/编译旗标下产出不同语义指纹，CI 漂移报告大量假阳性。  
高风险语言：Rust `cfg`、Swift `#if`、C/C++ 预处理、Zig `comptime` 分支。

**Why it happens:** 将“编译配置结果”混入“语义基线”，却没有显式记录编译上下文。

**How to avoid:**
- 指纹输入必须包含 `analysis_context`（target、feature flags、宏定义）。
- 设“基准上下文”（如 Linux x86_64 + 默认 flags）作为门禁唯一判定源；其他上下文只做补充报告。
- 对条件分支产物输出 `conditional_branches_detected` 元数据。

**Warning signs:**
- 同一 commit 在 macOS 与 Linux 漂移结果不同。
- CI 中 C/C++、Rust 文件 C1/C2 漂移占比异常高。
- 修复一次后下次又复发（非确定性漂移）。

**Phase to address:** **P1 设计上下文协议，P3 指纹实现，P5 门禁落地**。

---

### Pitfall 3: 宏/生成机制覆盖不全，出现“静默漏提取”

**What goes wrong:** 适配器能跑通，但漏掉关键声明（Rust 宏展开生成项、C# source generator + partial、Zig comptime 生成声明、C/C++ 宏拼接结果）。

**Why it happens:** 只按语法节点抓“显式声明”，忽略语言生成机制。

**How to avoid:**
- 契约层明确区分：`explicit_symbols` 与 `generated_or_uncertain_symbols`。
- 对无法可靠还原的生成符号，必须降级置信度并给出 `generation_reason`。
- 在知识层明确标注“该模块存在生成语义，需展开源码/构建产物确认”。

**Warning signs:**
- 代码审查发现“镜像里没有这个公开 API”。
- partial/comptime 项目导出数明显偏低。
- 低置信度长期为 0（不合理，说明你在隐藏不确定性）。

**Phase to address:** **P2 语言实现，P4 知识层标注，P5 回归验证**。

---

### Pitfall 4: C/C++ 头文件与语言识别歧义，适配器路由错误

**What goes wrong:** `.h/.inc` 文件被错误归类为 C 或 C++，导致解析树错误、符号缺失、依赖图污染。

**Why it happens:** 仅靠扩展名判断语言，不看编译单元上下文与 include 关系。

**How to avoid:**
- 语言识别采用“扩展名 + 邻接编译单元 + 配置覆盖”的三级策略。
- `anrsm.yaml` 提供 `language_overrides`（目录级/文件级）强制映射。
- 对歧义文件输出 `language_detection_confidence`，低置信默认不阻断。

**Warning signs:**
- C/C++ 项目里 ERROR 节点集中在 `.h`。
- 同一头文件在不同 run 被分到不同 adapter。
- 依赖图出现不合理跨语言边。

**Phase to address:** **P2 识别实现，P5 回归矩阵**。

---

### Pitfall 5: Java/Kotlin 包与模块边界处理错误，聚合层边界失真

**What goes wrong:** JVM 语言按目录聚合而非按 package/module 语义聚合，导致模块镜像要么过大要么切碎。

**Why it happens:** 复用 TS/Python 的目录即模块思路。

**How to avoid:**
- Java/Kotlin 采用“声明优先”边界：package + module 语义优先于目录。
- 聚合时引入 `declared_namespace` 与 `physical_path` 双字段，避免单一维度。
- route 优先按命名空间命中，再回落目录。

**Warning signs:**
- 单模块镜像暴涨（>3KB 摘要上限）或模块数异常爆炸。
- 同 package 的符号被切到多个无关模块。

**Phase to address:** **P2 适配器、P3 聚合策略**。

---

### Pitfall 6: tree-sitter 升级或语言 grammar 漂移引发全仓重算

**What goes wrong:** 新增 9 语言后，任一 grammar 版本升级触发大面积 AST 变化，导致指纹/漂移几乎全红。

**Why it happens:** 没有 parser 版本锁和升级迁移策略。

**How to avoid:**
- 维护 `parser-lock`（语言→grammar 版本）并写入镜像头。
- 引入 `fp_version`，升级时走显式 `migrate/rebuild` 流程。
- PR 模板要求“升级 parser 时附带漂移基线重建说明”。

**Warning signs:**
- 非业务改动 PR 出现 30%+ 文件漂移。
- parser 版本变更后 CI 波动明显。

**Phase to address:** **P1 版本策略，P3 指纹版本化，P5 发布门禁**。

---

### Pitfall 7: 新适配器仅做“解析成功”，没做“管线一致性”

**What goes wrong:** 单语言测试都过，但接入后在 aggregation / route / knowledge / drift 任一环节掉语义字段。

**Why it happens:** 测试停留在 adapter unit test，缺少端到端契约测试。

**How to avoid:**
- 每语言必须有 E2E fixture：`parse -> file mirror -> module aggregate -> route -> drift` 全链路快照。
- 增加“字段完整性断言”（exports/imports/side_effects/confidence/source_artifacts）。
- 任何语言接入后，统一跑“多语言混仓”回归。

**Warning signs:**
- 文件镜像有数据，但模块镜像为空或 route 命中失败。
- 新语言 CI 只跑了 adapter crate，没跑 integration。

**Phase to address:** **P5（必须）**。

---

## Moderate Pitfalls

### Pitfall 8: 低置信度策略缺位，系统“看起来很自信”

**What goes wrong:** 对解析失败/歧义场景仍输出 high confidence，误导下游决策。

**How to avoid:**
- 统一置信度分层：`high/medium/low` + 明确触发条件。
- 门禁按置信度分级（low 默认告警不阻断，但进入人工复核队列）。

**Warning signs:** 所有语言长期 100% high；但人工抽检错误明显。  
**Phase to address:** **P1 规范，P4 消费，P5 验证**。

---

### Pitfall 9: 生成文件未隔离，噪声淹没有效变化

**What goes wrong:** 把构建产物或自动生成代码直接纳入镜像/漂移，导致告警噪声巨大。

**How to avoid:**
- 默认排除生成目录（可配置白名单纳入）。
- 对纳入的生成文件标注 `generated=true` 并使用独立阈值。

**Warning signs:** 每次构建后漂移报告大量变化但业务代码未改。  
**Phase to address:** **P3 策略 + P5 门禁**。

---

### Pitfall 10: 性能预算缺失，新增语言后体验断崖

**What goes wrong:** C++ 模板、Java 大仓、C# 生成代码使扫描耗时激增，开发者开始绕过工具。

**How to avoid:**
- 建立预算：单文件、增量、全量三档 SLA。
- 引入超时与降级策略（超时->low confidence + partial output）。
- 统计语言维度耗时 TopN，针对性优化。

**Warning signs:** PR 中 WTCD 步骤持续变慢；本地运行 >30s 频发。  
**Phase to address:** **P5**。

---

## 语言特异高风险清单（新增 9 语种）

| 语言 | 最常见接入错误 | 预警信号 | 预防策略 | 应处理 Phase |
|---|---|---|---|---|
| Rust | 忽略 `cfg` / 宏导致符号不稳定 | 同文件跨平台结果不同 | 指纹绑定 analysis context，标注条件分支 | P1/P3 |
| Dart | 只看文件不看库边界（`part`/import 语义） | 同库符号分散、私有 `_` 泄露 | 按 library 语义聚合，私有规则单独处理 | P2/P3 |
| Java | 目录当包名，忽略 module 可见性 | 包边界错乱、导出过多 | package/module 语义优先 | P2 |
| Kotlin | expect/actual 与扩展函数静态分发误判 | 符号重复或“覆盖关系”错判 | 区分声明关系与调用分发关系 | P2/P3 |
| Swift | 条件编译分支未记录 | iOS/macOS 结果不一致 | 记录 target 条件并分上下文比较 | P1/P3 |
| C++ | 预处理与模板导致 AST 不稳定 | ERROR 节点、耗时飙升 | 基线宏配置 + 超时降级 | P2/P5 |
| C# | partial/source generator 合并不完整 | API 缺失或重复 | 显式区分 partial 片段与合并结果 | P2/P3 |
| C | 预处理分支和头文件歧义 | `.h` 解析质量差 | 语言覆盖配置 + 编译上下文 | P2/P5 |
| Zig | comptime 生成声明被忽略 | 导出索引偏低 | 标注 generated/uncertain，必要时降级 | P2/P4 |

---

## Integration Gotchas（与现有能力的耦合风险）

| 现有能力 | 新增适配器常见误接法 | 正确做法 |
|---|---|---|
| Module Aggregation | 直接按路径合并，忽略语言边界 | 先语言内归一化，再跨语言聚合 |
| Drift/Check | 所有语言共用同一噪声阈值 | 按语言族设置阈值与降噪规则 |
| Route Index | 只索引符号名，不含语言/命名空间 | 增加 language + namespace + confidence 维度 |
| Knowledge Layer | 把低置信事实写成确定结论 | 知识层强制展示 confidence 与 source_artifacts |
| CLI/MCP 输出 | 新字段破坏老消费者 | 使用 schema version + 向后兼容字段策略 |

---

## Pitfall-to-Phase Mapping（路线图可直接引用）

| Pitfall | Prevention Phase | Verification |
|---|---|---|
| 可见性语义错配 | P1/P2/P3 | 多语言可见性 fixture 全通过 |
| 条件编译导致假漂移 | P1/P3/P5 | 同 commit 跨平台漂移差异在阈值内 |
| 宏/生成机制静默漏提取 | P2/P4/P5 | generated/uncertain 标注率与人工抽检一致 |
| C/C++ 语言识别歧义 | P2/P5 | `.h` 文件语言判定稳定率达标 |
| JVM 包模块边界失真 | P2/P3 | 模块大小分布与命名空间一致 |
| parser 版本漂移 | P1/P3/P5 | parser 升级走 migrate 流程并有基线重建记录 |
| 只测 adapter 不测全链路 | P5 | 每语言 E2E 快照回归通过 |
| 低置信策略缺位 | P1/P4/P5 | 低置信样本可追踪且不误阻断 |
| 生成文件噪声 | P3/P5 | 漂移报告噪声比例下降 |
| 性能回退 | P5 | 满足单文件/增量/全量预算 |

---

## Sources

### Project Context
- `.planning/PROJECT.md`（v0.2.0 范围、现有能力、约束）

### Official / High-Confidence References
- Tree-sitter docs（ERROR/MISSING、增量解析机制）：https://tree-sitter.github.io/tree-sitter/  
- Rust Reference（`cfg`、macro）：https://doc.rust-lang.org/reference/conditional-compilation.html 、https://doc.rust-lang.org/reference/macros.html  
- Dart libraries/imports（库级私有、import 语义）：https://dart.dev/language/libraries  
- Java Language Spec Chapter 7（packages/modules）：https://docs.oracle.com/javase/specs/jls/se21/html/jls-7.html  
- Kotlin Extensions（静态分发特性）：https://kotlinlang.org/docs/extensions.html  
- Swift Reference Manual（conditional compilation）：https://github.com/swiftlang/swift-book/blob/main/TSPL.docc/ReferenceManual/Statements.md  
- C/C++ 预处理器（翻译阶段与条件编译）：https://en.cppreference.com/w/c/preprocessor 、https://en.cppreference.com/w/cpp/preprocessor  
- C# partial type：https://learn.microsoft.com/en-us/dotnet/csharp/language-reference/keywords/partial-type  
- Zig Language Reference（comptime）：https://ziglang.org/documentation/master/#comptime

---

*Pitfalls research for WTCD v0.2.0 polyglot adapters*  
*Researched: 2026-03-22*
