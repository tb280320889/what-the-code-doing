# Stack Research

**Domain:** WTCD Rust CLI 多语言适配器扩展（v0.2.0）  
**Researched:** 2026-03-22  
**Confidence:** HIGH（栈主干）/ MEDIUM（个别语法库维护活跃度）

## Recommended Stack

> 仅覆盖本里程碑新增能力：Rust / Dart / Java / Kotlin / Swift / C++ / C# / C / Zig 适配器。  
> 明确不重复研究已验证能力：TS/JS、Python、Go、模块聚合、知识层。

### Core Technologies

| Technology | Version | Purpose | Why Recommended |
|------------|---------|---------|-----------------|
| `tree-sitter` | **0.26.7** | 统一增量解析运行时 | 当前生态主线版本；`Parser::set_language` 明确提供语法 ABI 兼容校验（不兼容会显式报错），适合 WTCD 的“可验证”原则。 |
| `tree-sitter-language` | **0.1.7** | 语法库与 runtime 的桥接类型 | 作为 tree-sitter 新语法包装统一桥接层，降低“语法库版本号 != runtime 版本号”带来的耦合风险。建议显式加入 workspace，避免被动漂移。 |
| `wtcd-core::LanguageAdapter` + `AdapterRegistry`（现有） | N/A | 统一接入 9 种新语言 | 现有 trait/registry 架构已验证可扩展，不需要重构核心，只需新增 adapter 模块 + 注册 + fixtures + 回归测试。 |

### Supporting Libraries（新增）

| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| `tree-sitter-rust` | **0.24.1** | Rust 语法解析 | 新增 `.rs` 适配器时使用。 |
| `tree-sitter-dart` | **0.1.0** | Dart 语法解析 | 新增 `.dart` 适配器时使用。 |
| `tree-sitter-java` | **0.23.5** | Java 语法解析 | 新增 `.java` 适配器时使用。 |
| `tree-sitter-kotlin-ng` | **1.1.0** | Kotlin 语法解析 | 新增 `.kt` / `.kts` 适配器时使用；优先 `-ng`（维护与近期下载更强）。 |
| `tree-sitter-swift` | **0.7.1** | Swift 语法解析 | 新增 `.swift` 适配器时使用。 |
| `tree-sitter-cpp` | **0.23.4** | C++ 语法解析 | 新增 `.cpp/.cc/.cxx/.hpp/.hh/.hxx` 适配器时使用。 |
| `tree-sitter-c-sharp` | **0.23.1** | C# 语法解析 | 新增 `.cs` 适配器时使用。 |
| `tree-sitter-c` | **0.24.1** | C 语法解析 | 新增 `.c/.h` 适配器时使用。 |
| `tree-sitter-zig` | **1.1.2** | Zig 语法解析 | 新增 `.zig` 适配器时使用。 |

### Development Tools

| Tool | Purpose | Notes |
|------|---------|-------|
| 现有 `wtcd-tests` 集成测试框架 | 回归验证 9 语种解析质量 | 每个语言至少覆盖：正常语法、部分损坏语法、导入导出、典型副作用提取。 |
| `cargo test -p wtcd-tests` | 跨命令链路验证 | 必测 `run/check/route` 对新增扩展名的适配器命中行为。 |

## Integration Points（必须改动点）

1. **workspace 依赖层**（`/Cargo.toml`）
   - 新增上述 9 个 grammar crate（建议统一放 `[workspace.dependencies]`）。
   - 将 `tree-sitter` 从 `0.26` 固定到 `0.26.7`（减少隐式升级不确定性）。
   - 建议显式加入 `tree-sitter-language = "0.1.7"`（即使主要为传递依赖）。

2. **适配器层**（`crates/wtcd-adapters`）
   - `src/lib.rs`：新增 `mod` 与 `register_all_adapters()` 注册顺序。
   - 新增文件：`rust.rs / dart.rs / java.rs / kotlin.rs / swift.rs / cpp.rs / csharp.rs / c.rs / zig.rs`。
   - 每个适配器沿用现有模式：`Mutex<Parser>` + `set_language(...)` + `parse(...)` + confidence 降级。

3. **扫描层**（`crates/wtcd-scope/src/scanner.rs`）
   - 扩展 `SUPPORTED_EXTENSIONS`，新增：
     - Rust: `rs`
     - Dart: `dart`
     - Java: `java`
     - Kotlin: `kt`, `kts`
     - Swift: `swift`
     - C++: `cpp`, `cc`, `cxx`, `hpp`, `hh`, `hxx`
     - C#: `cs`
     - C: `c`, `h`
     - Zig: `zig`
   - 注意：`.h` 在 C/C++ 双语义中不可避免，建议先以 C adapter 默认处理，后续再做上下文判定优化。

4. **命令链路层**（现有 CLI/MCP）
   - `run/parse` 命令无需架构变更；依赖 registry 命中扩展名即可。
   - 新增“启动时语言加载失败”报错归类（语法 ABI 不兼容时更可诊断）。

## Installation

```toml
# /Cargo.toml
[workspace.dependencies]
tree-sitter = "0.26.7"
tree-sitter-language = "0.1.7"

tree-sitter-rust = "0.24.1"
tree-sitter-dart = "0.1.0"
tree-sitter-java = "0.23.5"
tree-sitter-kotlin-ng = "1.1.0"
tree-sitter-swift = "0.7.1"
tree-sitter-cpp = "0.23.4"
tree-sitter-c-sharp = "0.23.1"
tree-sitter-c = "0.24.1"
tree-sitter-zig = "1.1.2"

# /crates/wtcd-adapters/Cargo.toml
[dependencies]
tree-sitter = { workspace = true }
tree-sitter-rust = { workspace = true }
tree-sitter-dart = { workspace = true }
tree-sitter-java = { workspace = true }
tree-sitter-kotlin-ng = { workspace = true }
tree-sitter-swift = { workspace = true }
tree-sitter-cpp = { workspace = true }
tree-sitter-c-sharp = { workspace = true }
tree-sitter-c = { workspace = true }
tree-sitter-zig = { workspace = true }
```

## Alternatives Considered

| Recommended | Alternative | When to Use Alternative |
|-------------|-------------|-------------------------|
| `tree-sitter-kotlin-ng@1.1.0` | `tree-sitter-kotlin@0.3.8` | 若你必须与旧查询/节点命名保持兼容，可短期维持旧包；否则优先 `-ng`。 |
| Tree-sitter 全语种统一解析 | 各语言官方编译器前端（rustc parser、JDT、Roslyn、SwiftSyntax 等） | 仅在你要做“编译级语义/类型推断/符号解析”时才值得；当前 WTCD 目标是结构化语义镜像，不需要。 |

## What NOT to Use

| Avoid | Why | Use Instead |
|-------|-----|-------------|
| 引入 LSP Server 作为解析核心（rust-analyzer/JDT/Roslyn/sourcekit 等） | 依赖重、启动慢、跨平台部署复杂；与“Rust 单二进制 CLI”约束冲突 | 继续用 tree-sitter grammar crate 直连解析 |
| 为每个语言拆独立 crate（`wtcd-adapter-*`） | 当前阶段会显著增加维护面和发布复杂度，收益不足 | 保持 `wtcd-adapters` 单 crate 多模块 |
| 在本里程碑加入语义索引数据库（SQLite/PG） | 需求外扩，拖慢交付，不属于适配器扩展最短路径 | 仅做解析与镜像输出兼容接入 |
| 使用“未验证来源”的 grammar fork 作为默认选型 | 长期维护风险高 | 首选 tree-sitter 官方/主流 grammar 仓库对应 crate |

## Stack Patterns by Variant

**如果目标是“最快完成 v0.2.0”**  
- 使用上述 9 个 grammar crate + 统一 adapter 模板（复制 `go.rs`/`py.rs` 模式）  
- 因为这条路径最少架构改动、最小回归面

**如果后续目标升级为“语义精确到类型系统级别”**  
- 在后续里程碑按语言引入编译器前端/LSP 结果融合层  
- 因为 tree-sitter 擅长语法结构，不直接提供类型绑定/解析

## Version Compatibility

| Package A | Compatible With | Notes |
|-----------|-----------------|-------|
| `tree-sitter@0.26.7` | `tree-sitter-language@0.1.7` | 官方 Rust binding 组合。 |
| `Parser::set_language` | 所有 grammar crate 的 `LANGUAGE` 常量 | 运行时会校验 language ABI 版本；不兼容会返回错误。 |
| `tree-sitter-kotlin-ng@1.1.0` | `tree-sitter@0.26.x` | **MEDIUM**：需在仓库内编译实测确认（建议加 CI smoke）。 |

## Sources

- Context7 `/tree-sitter/tree-sitter` — Rust binding 用法、`Parser::set_language` 行为、语言版本兼容说明（HIGH）
- 官方文档 `https://docs.rs/tree-sitter/latest/tree_sitter/struct.Parser.html` — `set_language` 兼容校验与错误语义（HIGH）
- 官方文档 `https://tree-sitter.github.io/tree-sitter/using-parsers/` — parser 使用基线与官方 bindings 入口（HIGH）
- crates 元数据（通过 Rust crate 检索）:
  - `tree-sitter@0.26.7`
  - `tree-sitter-language@0.1.7`
  - `tree-sitter-rust@0.24.1`
  - `tree-sitter-dart@0.1.0`
  - `tree-sitter-java@0.23.5`
  - `tree-sitter-kotlin-ng@1.1.0`
  - `tree-sitter-swift@0.7.1`
  - `tree-sitter-cpp@0.23.4`
  - `tree-sitter-c-sharp@0.23.1`
  - `tree-sitter-c@0.24.1`
  - `tree-sitter-zig@1.1.2`
  （版本与活跃度：MEDIUM-HIGH，建议落地前 `cargo update -p ...` + CI 编译二次确认）

---
*Stack research for: WTCD v0.2.0 Polyglot Adapters (9 languages)*
