# Phase 10: Aggregation Consistency Hardening - Context

**Gathered:** 2026-03-22
**Status:** Ready for planning

<domain>
## Phase Boundary

用户可获得跨 13 种语言（既有 + 新增）语义一致的依赖聚合与漂移判定结果。确保 import 归一、模块图一致性、置信度分级在所有语言中行为统一。

</domain>

<decisions>
## Implementation Decisions

### Import Path Resolution
- 在 `resolve_import_path()` 中添加 per-language 扩展名映射（Rust→.rs, Java→.java, C→.c/.h, 等）
- 保持 adapter 级别 import 解析 — 每个 adapter 将 import 标准化为 DependencyEdge
- C/C++ 头文件通过 `#include` 路径 + 文件存在性检查解析

### Module Graph Consistency
- 修复 `build_module_graph` 子串匹配 → 使用路径前缀比较（canonical module ID）
- 混合语言模块通过 SideEffect 存储 per-language 元数据，设置 `language="mixed"`
- 置信度上卷到模块级 — 取模块内所有文件的最低置信度

### Confidence & Generated Symbol Marking
- 在 ExportedSymbol 添加 `is_generated: bool` 和 `confidence: ConfidenceBand`
- 预处理指令（`#ifdef`/`#ifndef`）提取为 SideEffect(kind=Log)，不纳入语义指纹
- 在 ModuleResult 添加 `confidence` 字段，在 mirror header 中展示

</decisions>

<code_context>
## Existing Code Insights

### Reusable Assets
- `ReverseDepGraph` in `crates/wtcd-core/src/depgraph.rs` — 需要扩展 resolve_import_path()
- `aggregate_module()` in `crates/wtcd-mirror/src/module.rs` — 需要添加 confidence rollup
- `build_module_graph()` in same file — 需要修复子串匹配
- `ExportedSymbol` in `crates/wtcd-core/src/types.rs` — 需要添加 is_generated + confidence 字段
- `ModuleResult` in same file — 需要添加 confidence 字段

### Established Patterns
- 适配器输出统一 FileResult → DependencyEdge 标准化格式
- ConfidenceBand 已在 FileResult 级别使用，需要上卷到 symbol 和 module 级别
- SideEffect(kind=Log) 用于存储语言特定元数据

### Integration Points
- `crates/wtcd-core/src/types.rs` — ExportedSymbol, ModuleResult 结构变更
- `crates/wtcd-core/src/depgraph.rs` — resolve_import_path() 扩展
- `crates/wtcd-mirror/src/module.rs` — aggregate_module(), build_module_graph() 修复
- 所有 12 个适配器 — 需要设置 is_generated 和 confidence 在 ExportedSymbol 上

</code_context>

<specifics>
## Specific Ideas

- 扩展名映射使用 lazy_static 或 const HashMap，避免每次调用分配
- 路径规范化：统一使用 canonicalize 或相对路径，避免绝对/相对不一致
- 预处理指令在 C/C++/Swift/Zig 适配器中已经提取为 SideEffect，需要确认不纳入语义指纹

</specifics>

<deferred>
## Deferred Ideas

- 跨语言依赖关系分析（C/C++ 共享头文件的完整类型解析）— 未来需求
- 编译器/LSP 辅助的高精度类型解析 — 未来需求 POLY-01
- JNI/PInvoke/extern 等 FFI 模式建模 — 未来需求 POLY-03

</deferred>
