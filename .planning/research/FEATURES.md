# Feature Research

**Domain:** WTCD v0.2.0 多语言适配器（Rust / Dart / Java / Kotlin / Swift / C++ / C# / C / Zig）  
**Researched:** 2026-03-22  
**Confidence:** MEDIUM-HIGH（核心模式 HIGH；各语言细粒度语义边界 MEDIUM）

## Feature Landscape

> 范围限定：仅覆盖本里程碑新增 9 种语言适配器。TS/JS、Python、Go、模块聚合、知识层为既有能力，不重复定义。

### Table Stakes (Users Expect These)

Features users assume exist. Missing these = product feels incomplete.

| Feature | Why Expected | Complexity | Notes |
|---------|--------------|------------|-------|
| **语言识别与扩展名命中** | 用户会默认 `.rs/.java/.kt/.swift/.cpp/.cs/.c/.zig/.dart` 被扫描并命中对应适配器 | MEDIUM | 依赖现有 `wtcd-scope::SUPPORTED_EXTENSIONS` 与 `AdapterRegistry::find_adapter`；这是所有后续提取的前置条件。 |
| **AST 解析 + 置信度分级（High/Low/None）** | 现有 TS/Python/Go 已有“错误容忍但可降级”体验，新语言必须一致 | LOW | 复用现有 `FileResult.confidence` 与 `error_message` 语义；每个适配器都应保持 `root.has_error()` 降级策略。 |
| **顶层符号提取（函数/类型/常量/变量）** | 用户最核心问题是“这个文件暴露什么” | MEDIUM | 复用 `ExportedSymbol` + `ExportKind`；按语言做规范映射：如 Java/Kotlin/C# 的 class/interface，Rust/C++/C/Zig 的 fn/struct/enum。 |
| **导入/依赖边提取** | 路由、模块聚合、知识层都依赖 import/use/include edges | MEDIUM | 复用 `DependencyEdge`；C/C++ 主要来自 `#include`，Rust 来自 `use`，JVM 系来自 `import`。 |
| **签名提取（参数 + 返回）** | Agent 定位修改点时首先看签名，不看实现细节 | MEDIUM | 复用 `FunctionSignature`/`Parameter`；对无显式类型语言点位（Dart `dynamic` 等）允许降级为字符串占位。 |
| **副作用提取（IO/Network/Storage/Log）** | 现有镜像把副作用当一等事实，新语言要可比较 | MEDIUM | 复用 `SideEffectKind` 四分类；语言特征先映射到现有枚举（避免里程碑中途扩枚举导致全链路回归）。 |
| **每语言 fixtures + 单元回归 + 集成回归** | 无回归就无法保证“新增语言不破坏旧链路” | HIGH | 依赖现有 `wtcd-tests`；每语言至少覆盖：正常语法、损坏语法、导入导出、副作用、命令链路（run/check/route）。 |
| **CLI/MCP 无感接入** | 用户不接受“新语言需要新命令” | LOW | 复用现有命令面；只允许通过注册表扩展，不改 CLI 交互模型。 |

### Differentiators (Competitive Advantage)

Features that set the product apart. Not required, but valuable.

| Feature | Value Proposition | Complexity | Notes |
|---------|-------------------|------------|-------|
| **跨语言统一语义归一层（同一 ExportKind 语义对齐）** | Agent 可以跨语言复用同一阅读/修改策略，而不是每语言重学一套 | HIGH | 依赖现有 `wtcd-core::types`；建议先做“映射规范文档 + golden fixtures”，再做实现。 |
| **语言特有元信息以“可机器过滤标签”输出** | 保留语言特性但不污染核心 schema（例如 Rust `pub(crate)`、Kotlin `suspend`、C++ `template`） | MEDIUM | 参考现有 Python/Go 的 `meta:` 方式；短期放到 `side_effects.target` 元标签，后续可演进独立字段。 |
| **适配器能力矩阵（每语言支持哪些提取项）** | 降低误解：用户可预期“这门语言目前能提取到什么粒度” | LOW | 作为文档/JSON 清单输出；依赖现有 knowledge layer，可自动生成。 |
| **损坏代码韧性基准（broken fixtures score）** | 实际仓库常有半完成代码，韧性高会显著提升可用性 | MEDIUM | 利用 tree-sitter 错误恢复能力；每语言定义最低“可提取率”门槛。 |
| **跨语言依赖热点标注（FFI/interop 边）** | 多语言仓库最痛点是边界复杂；标注 interop 边提升路由质量 | HIGH | 依赖模块聚合与知识层；可先从关键词规则起步（JNI、cgo、extern、P/Invoke、FFI）。 |

### Anti-Features (Commonly Requested, Often Problematic)

Features that seem good but create problems.

| Feature | Why Requested | Why Problematic | Alternative |
|---------|---------------|-----------------|-------------|
| **编译器级类型求解（完整 name/type resolution）** | 用户希望“像 IDE 一样精确” | 复杂度和依赖爆炸（需引入 rustc/JDT/Roslyn/SourceKit/clang frontend），违背当前 Rust CLI 轻量约束 | 坚持 AST 级结构提取；把“类型求解”列为后续独立里程碑。 |
| **执行源码来推断动态行为** | 试图提高动态语言/宏系统准确率 | 安全风险高、不可复现、CI 不稳定 | 仅静态提取 + 低置信标记；动态行为以 `Unknown/Meta` 标注。 |
| **在 v0.2.0 扩展核心枚举覆盖所有语言细粒度语义** | 看起来“更完整” | 会牵连 mirror/drift/route/knowledge 全链路 schema 与回归，大幅拖慢交付 | 先做稳定映射到现有枚举，语言特性走元标签。 |
| **宏/预处理器完整展开（Rust macro_rules/C 预处理/C++ template 深推导）** | 用户认为“展开后才真实” | 需要编译上下文，跨工具链依赖重，且结果不稳定 | 只提取声明层事实；检测到宏/预处理器时输出能力边界提示。 |
| **一次性追求“9 语言语义等深”** | 希望体验完全一致 | 现实中语言语法/生态差异大，硬拉齐会导致普遍低质量 | 分层目标：先“可用一致”（table stakes），再“语义等深”（v0.2.x+）。 |

## Feature Dependencies

```text
[扩展名扫描支持]
    └──requires──> [AdapterRegistry 注册 9 语言]
                         └──requires──> [每语言 Parser 初始化 + parse 降级]
                                              └──requires──> [exports/imports/signatures/side_effects 提取]
                                                                   └──requires──> [fixtures + 单测回归]
                                                                                        └──requires──> [run/check/route 集成回归]

[跨语言语义归一层] ──enhances──> [路由质量 + 知识层质量]

[类型求解/宏展开] ──conflicts──> [v0.2.0 交付节奏与轻量架构]
```

### Dependency Notes

- **扩展名扫描支持 requires AdapterRegistry 注册：** 扫描不到或注册不到，后续全部特性都不会触发。
- **Registry 注册 requires Parser 初始化：** `Parser::set_language` 失败会导致该语言适配器不可用，必须在启动期暴露明确错误。
- **解析层 requires 结构化提取层：** 只有 parse 无提取，无法产出镜像语义价值。
- **提取层 requires fixtures/测试：** 多语言最容易“看似能跑，语义错位”，必须用 golden fixtures 锁行为。
- **跨语言语义归一 enhances 路由/知识层：** 这是区分“能解析”与“能让 Agent 快速决策”的关键。
- **类型求解/宏展开 conflicts v0.2.0：** 会直接把里程碑从“适配器扩展”变成“编译器前端项目”。

## MVP Definition

### Launch With (v0.2.0)

Minimum viable product — what's needed to validate the concept.

- [ ] **9 语言均可被扫描并命中适配器** — 没有命中就没有任何下游价值。
- [ ] **每语言稳定产出四类事实（exports/imports/signatures/side_effects）+ confidence** — 与现有 TS/Py/Go 语义一致。
- [ ] **每语言最小但完整的 fixtures+测试闭环** — 保证可持续迭代，不是一次性脚本。
- [ ] **CLI/MCP 链路零新增心智负担** — 用户继续用原命令即可获得新语言支持。

### Add After Validation (v0.2.x)

Features to add once core is working.

- [ ] **跨语言语义归一规范文档 + 自动校验** — 当 9 语言基础可用后再做统一质量提升。
- [ ] **FFI/interop 边识别（JNI、P/Invoke、extern 等）** — 当用户出现跨语言路由痛点时加入。
- [ ] **语言能力矩阵自动生成** — 当适配器数量继续增长时降低认知负担。

### Future Consideration (v0.3+)

Features to defer until product-market fit is established.

- [ ] **编译器级语义补强（类型求解/符号解析）** — 高收益但高成本，需独立架构决策。
- [ ] **宏/模板/预处理展开级分析** — 仅在用户确有强需求且能接受性能成本时推进。

## Feature Prioritization Matrix

| Feature | User Value | Implementation Cost | Priority |
|---------|------------|---------------------|----------|
| 扩展名扫描 + 适配器命中 | HIGH | MEDIUM | P1 |
| 四类结构化提取 + confidence | HIGH | MEDIUM | P1 |
| 每语言 fixtures + 回归测试 | HIGH | HIGH | P1 |
| CLI/MCP 无感接入 | HIGH | LOW | P1 |
| 跨语言语义归一层 | HIGH | HIGH | P2 |
| 语言元信息标签化输出 | MEDIUM | MEDIUM | P2 |
| FFI/interop 边识别 | MEDIUM | HIGH | P3 |
| 编译器级类型求解 | MEDIUM | VERY HIGH | P3（当前拒绝） |

**Priority key:**
- P1: Must have for launch
- P2: Should have, add when possible
- P3: Nice to have, future consideration

## Competitor Feature Analysis

| Feature | Competitor A (源码索引类工具常见做法) | Competitor B (LSP/IDE 类能力) | Our Approach |
|---------|--------------------------------------|----------------------------------|--------------|
| 多语言解析 | 倾向 AST 级语法提取，强调速度与覆盖 | 倾向编译器级语义，精度高但依赖重 | 先 AST 级全覆盖（v0.2.0），后续按需增量引入深语义。 |
| 错误容忍 | 允许部分解析并降级 | 依赖项目可编译性，失败时常“不可用” | 强制 confidence 分级，损坏代码也尽量产出可用镜像。 |
| 语义统一 | 常出现“每语言各说各话” | IDE 内部统一但对外可移植性弱 | 以 `wtcd-core::types` 为单一语义契约，语言差异通过映射层处理。 |

## Milestone-Specific Language Notes (Table Stakes Cut)

- **Rust**：`fn/struct/enum/trait/impl/use/mod` 提取是最低线；`pub*` 可见性作为元标签。  
- **Dart**：`class/mixin/enum/extension/import/export/part` 提取是最低线；`async/await` 记副作用提示。  
- **Java**：`class/interface/enum/record/method/import/package` 提取是最低线；仅做源级签名，不做字节码级解析。  
- **Kotlin**：`class/object/interface/data class/sealed/extension function/import` 提取是最低线；`suspend` 作为元标签。  
- **Swift**：`struct/class/enum/protocol/func/import/extension` 提取是最低线；`async throws` 进签名字符串。  
- **C++**：`class/struct/enum/function/template/include/namespace` 提取是最低线；模板深度语义不做。  
- **C#**：`class/interface/record/enum/method/using/namespace` 提取是最低线；`async Task` 作为签名文本。  
- **C**：`function/struct/enum/typedef/include/macro(仅标记)` 提取是最低线；预处理展开不做。  
- **Zig**：`pub fn/const/struct/enum/union/use` 提取是最低线；comptime 深语义不做。

## Sources

- `.planning/PROJECT.md`（里程碑范围与约束，HIGH）
- `crates/wtcd-core/src/adapter.rs`（适配器契约，HIGH）
- `crates/wtcd-core/src/types.rs`（输出 schema 能力边界，HIGH）
- `crates/wtcd-adapters/src/ts.rs`（现有成熟基线，HIGH）
- `crates/wtcd-adapters/src/py.rs`（元信息与降级策略实践，HIGH）
- `crates/wtcd-adapters/src/go.rs`（可见性/语言特性映射实践，HIGH）
- `crates/wtcd-scope/src/scanner.rs`（扫描入口与扩展名门槛，HIGH）
- Context7 `/tree-sitter/tree-sitter`（Parser API、Query、ERROR/MISSING 节点，HIGH）
- 官方文档 `https://docs.rs/tree-sitter/latest/tree_sitter/struct.Parser.html`（`set_language`/`parse` 行为，HIGH）
- 官方文档 `https://tree-sitter.github.io/tree-sitter/using-parsers/` 与 Query Syntax 页面（解析与查询通用模式，HIGH）

> 说明：本次未能使用 Tavily（环境缺少 `TAVILY_API_KEY`），因此“竞品横向对比”部分保持保守抽象表述；如需更强市场证据，建议在下阶段补充公开案例检索。

---
*Feature research for: WTCD v0.2.0 polyglot adapters*
*Researched: 2026-03-22*
