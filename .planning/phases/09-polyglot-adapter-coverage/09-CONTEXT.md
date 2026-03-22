# Phase 9: Polyglot Adapter Coverage - Context

**Gathered:** 2026-03-22
**Status:** Ready for planning

<domain>
## Phase Boundary

用户在不改变 CLI 使用方式的情况下，可对 9 种新增语言（Rust/Dart/Java/Kotlin/Swift/C++/C#/C/Zig）完成稳定解析与结构化语义提取。适配器遵循现有 LanguageAdapter trait 模式，集成到 wtcd run 流水线，支持错误恢复和置信度分级。

</domain>

<decisions>
## Implementation Decisions

### Adapter Architecture
- 新适配器使用单 Mutex<Parser> 包装模式 — 与 PyAdapter/GoAdapter 一致
- 不扩展 ExportKind 枚举 — 现有类型足够覆盖通用导出
- 在 register_all_adapters() 中逐个注册 — 与现有模式一致
- 每种语言独立适配器，扩展名列表明确（如 Rust: ["rs"], Java: ["java"]）

### Extraction Mapping
- 使用现有 ExportKind + 名称前缀约定处理可见性 — 如 Go 用大写，Rust 用 pub 前缀
- 映射到现有 DependencyEdge 结构处理 import 语义 — source + symbols + kind
- 语言特定元数据存储在 SideEffect 中，kind=Log — 与 Python 装饰器模式一致
- 条件编译/预处理指令提取为 SideEffect，kind=Log — 标记但不展开

### Testing Strategy
- 每种语言一个目录 `tests/fixtures/{lang}/` — 与现有 ts/python/go 一致
- 每种语言至少 5 个 fixture：hello, imports, classes/types, syntax_error, complex
- 在 wtcd-tests crate 中添加跨语言集成测试 — 验证 wtcd run 输出
- 添加 parse_time_ms 断言 — 确保解析时间在合理范围内（< 100ms/文件）

### Integration Approach
- 实现顺序按语言复杂度递增：C → Zig → Dart → Kotlin → Swift → Java → C# → C++ → Rust
- 不需要新增 CLI 子命令 — 通过现有 `wtcd run` 自动检测扩展名
- 暂不处理语言间依赖关系（如 C/C++ 共享头文件）— 在 Phase 10 处理
- 使用代表性开源仓库验证适配器正确性 — 如 Rust 用 small 项目，Java 用 Spring Boot 示例

</decisions>

<code_context>
## Existing Code Insights

### Reusable Assets
- `LanguageAdapter` trait in `crates/wtcd-core/src/adapter.rs` — 3-method interface (language_name, file_extensions, parse)
- `AdapterRegistry` in same file — register() + find_adapter() by extension
- `FileResult` in `crates/wtcd-core/src/types.rs` — exports, imports, signatures, side_effects, confidence, parse_time_ms
- `ConfidenceBand` enum — High/Low/None
- `ExportedSymbol`, `DependencyEdge`, `FunctionSignature`, `SideEffect` types — all defined in types.rs

### Established Patterns
- 适配器结构体包装 `Mutex<Parser>` — 单解析器（PyAdapter/GoAdapter）或多解析器（TsAdapter）
- tree-sitter 0.26 作为工作区依赖 — 各语言语法库通过 workspace dependencies 引入
- `register_all_adapters()` in `crates/wtcd-adapters/src/lib.rs` — 注册入口点
- 测试 fixtures 目录结构：`tests/fixtures/{lang}/`

### Integration Points
- `crates/wtcd-adapters/Cargo.toml` — 添加 tree-sitter-* 依赖
- `Cargo.toml` (workspace root) — 添加 tree-sitter-* 到 workspace dependencies
- `crates/wtcd-adapters/src/lib.rs` — 添加 `mod {lang};` 并注册适配器
- `crates/wtcd-adapters/src/` — 新建 `{lang}.rs` 文件（如 rust.rs, java.rs 等）

</code_context>

<specifics>
## Specific Ideas

- 每种语言的 tree-sitter 语法库需要兼容 tree-sitter 0.26 — 检查 crates.io 版本
- Rust 适配器需要处理 pub 关键字、mod 声明、use 语句、impl 块、derive 宏
- Java 适配器需要处理 package 声明、import static、注解、泛型、内部类
- C/C++ 需要处理头文件包含、条件编译、宏定义、extern "C" 块
- 语言特定的 node types 需要查阅各 tree-sitter 语法库文档

</specifics>

<deferred>
## Deferred Ideas

- 跨语言依赖关系分析（如 C/C++ 共享头文件）— 属于 Phase 10
- 条件编译/预处理指令的完整展开 — 属于未来需求 POLY-02
- 编译器/LSP 辅助的高精度类型解析 — 属于未来需求 POLY-01
- JNI/PInvoke/extern 等 FFI 模式建模 — 属于未来需求 POLY-03

</deferred>
