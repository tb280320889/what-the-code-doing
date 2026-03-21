# ANRSM — Technology Stack (v0.1.1 Extensions)

**Domain:** Rust CLI tool for code repository semantic mirror generation
**Researched:** 2026-03-21
**Confidence:** HIGH
**Scope:** ONLY new additions for Python adapter, Go adapter, module-level aggregation, knowledge layer

## New Dependencies

### Required Additions to Cargo.toml

| Crate | Version | Purpose | Why This Version |
|-------|---------|---------|------------------|
| **tree-sitter-python** | 0.25.0 | Python 语法解析 | 最新稳定版，2025-09 发布，5.2M+ 下载。依赖 `tree-sitter-language ^0.1`，与 tree-sitter 0.26 ABI 兼容（ABI v15） |
| **tree-sitter-go** | 0.25.0 | Go 语法解析 | 最新稳定版，2025-08 发布，3.8M+ 下载。同样依赖 `tree-sitter-language ^0.1`，ABI 兼容 |
| **tree-sitter-md** | 0.5.3 | Markdown 结构化解析（可选） | 用于知识层 ADR/架构文档提取。2026-02 发布，499K+ 下载，tree-sitter-grammars 官方维护。可选 feature `parser` 按需引入 |

### NOT Added — Existing Crates Suffice

| 需求 | 为什么不需要新依赖 |
|------|-------------------|
| 模块级聚合 | 纯业务逻辑，使用 `std::collections::HashMap` + 已有的 `FileResult` 类型 |
| YAML Front Matter 解析 | 已有 `yaml_serde` 0.10 |
| ADR 内容提取 | 方案 A: `pulldown-cmark` 0.13（已有）足够提取标题/段落。方案 B: 若需精确 AST 则加 `tree-sitter-md` |
| go.mod 解析 | 简单文本解析（`module` 行 + `require` 块），不需要专门的 Go mod parser |
| `__all__` 列表解析 | tree-sitter-python AST 节点已覆盖（`expression_statement > assignment`） |
| 并行处理 | `std::thread` + `rayon`（若未来需要）；当前文件级解析已足够快 |

## Version Compatibility Analysis

### tree-sitter 生态兼容矩阵

```
tree-sitter core:     0.26.7 (项目当前版本)
├── tree-sitter-language: ^0.1 (桥梁 crate)
├── tree-sitter-typescript: 0.23 ✅ 已使用
├── tree-sitter-javascript: 0.23 ✅ 已使用
├── tree-sitter-python: 0.25.0 ✅ 新增（同样依赖 tree-sitter-language ^0.1）
├── tree-sitter-go: 0.25.0 ✅ 新增（同样依赖 tree-sitter-language ^0.1）
└── tree-sitter-md: 0.5.3 ✅ 可选（同样依赖 tree-sitter-language ^0.1）
```

**关键架构决策（PR #3069）：** 从 tree-sitter 0.23 起，语法库通过 `tree-sitter-language ^0.1` 桥接，不再直接依赖特定 tree-sitter 版本。这意味着：
- `tree-sitter-python 0.25.0` 编译时不依赖 tree-sitter 0.25.x 运行时
- 只要运行时 tree-sitter 0.26 支持 ABI v15（语法库编译的 ABI 版本），就能正常加载
- 项目已验证此模式：`tree-sitter-typescript 0.23` 与 `tree-sitter 0.26` 配合工作

**信心等级：HIGH** — 基于 tree-sitter-language 架构 + 项目已有相同模式的类型脚本适配器验证

### 需要验证的点

| 验证项 | 风险 | 缓解措施 |
|--------|------|----------|
| tree-sitter-python 0.25.0 与 tree-sitter 0.26 编译 | LOW | 同一 `tree-sitter-language ^0.1` 桥接层 |
| tree-sitter-go 0.25.0 与 tree-sitter 0.26 编译 | LOW | 同上 |
| tree-sitter-md 0.5.3 与 tree-sitter 0.26 编译 | LOW | 同上，且 dev dep 指向 `^0.26.3` |
| Python `decorated_definition` 节点提取 | MEDIUM | 需要额外处理装饰器包裹的函数/类定义 |
| Go `method_declaration` 与 `function_declaration` 区分 | MEDIUM | 不同的 AST 节点类型，需要各自的提取逻辑 |

## Core Type Extensions

### ExportKind — 新增变体

当前 `ExportKind` 只有 TS/JS 概念的变体。Python 和 Go 需要补充：

```rust
pub enum ExportKind {
    // 现有（TS/JS）
    Function, Class, Const, Let, Var, Type, Interface, Enum,
    // 新增 — Python
    Module,       // 模块级定义（Python 的顶级命名空间）
    Decorator,    // @decorator 标注的导出
    // 新增 — Go
    Method,       // 带 receiver 的方法（与 Function 区分）
    Struct,       // type X struct {}
    InterfaceGo,  // type X interface {}（避免与 TS Interface 冲突命名）
}
```

**替代方案考虑：** 不扩展 ExportKind，而是用 `Function` + 元数据字段区分。但显式变体对下游消费者（漂移检测、路由索引）更友好。

### ImportKind — 新增变体

```rust
pub enum ImportKind {
    // 现有（TS/JS）
    Named, Default, Namespace,
    // 新增 — Python
    Wildcard,     // from x import *
    Relative,     // from . import x / from .. import y
    // 新增 — Go
    Dot,          // import . "fmt"
    Blank,        // import _ "pkg"
}
```

### SideEffectKind — 新增变体

```rust
pub enum SideEffectKind {
    // 现有
    Io, Network, Storage, Log,
    // 新增 — 通用
    Subprocess,   // Python: subprocess.run(), Go: os/exec
    Environment,  // Python: os.environ, Go: os.Getenv
}
```

## Scanner Extension

`wtcd-scope/src/scanner.rs` 的 `SUPPORTED_EXTENSIONS` 需扩展：

```rust
// 当前
const SUPPORTED_EXTENSIONS: &[&str] = &["ts", "tsx", "js", "jsx"];

// 扩展后
const SUPPORTED_EXTENSIONS: &[&str] = &[
    "ts", "tsx", "js", "jsx",  // TypeScript/JavaScript
    "py", "pyi",                 // Python（.pyi = type stub）
    "go",                        // Go
];
```

## Python Adapter — AST Node Types

tree-sitter-python 关键节点类型（从 node-types.json 验证）：

| 提取目标 | tree-sitter 节点类型 | 字段 |
|----------|---------------------|------|
| 函数定义 | `function_definition` | `name`, `parameters`, `return_type` |
| 类定义 | `class_definition` | `name`, `superclasses` |
| 装饰器包裹的定义 | `decorated_definition` | `definition`, `decorator` |
| 普通 import | `import_statement` | `name` (dotted_name) |
| from import | `import_from_statement` | `module_name`, `dotted_name` |
| 相对 import | `relative_import` | 父节点 `import_from_statement` |
| 赋值（`__all__`, 类型别名） | `assignment` | `left`, `type`, `right` |
| 类型别名 (3.12+) | `type_alias_statement` | `type` |
| async 函数 | `function_definition` | 有 `async` 关键字 |

### Python 特殊处理

1. **`__all__` 列表：** 识别 `__all__ = ["foo", "bar"]` 赋值，提取列表内容作为导出声明
2. **装饰器：** `@app.route("/")` 包裹的函数仍导出为 Function，但标记装饰器
3. **Type hints：** `def foo(x: int) -> str:` 的 `return_type` 字段直接可用
4. **async/await：** `async def` 的函数定义节点包含 `async` 关键字子节点

## Go Adapter — AST Node Types

tree-sitter-go 关键节点类型（从 node-types.json 验证）：

| 提取目标 | tree-sitter 节点类型 | 字段 |
|----------|---------------------|------|
| 包声明 | `package_clause` | `package_identifier` |
| 函数声明 | `function_declaration` | `name`, `parameters`, `result` |
| 方法声明 | `method_declaration` | `receiver`, `name`, `parameters`, `result` |
| 类型声明 | `type_declaration` | `type_spec` (name + type) |
| 结构体类型 | `struct_type` | `field_declaration_list` |
| 接口类型 | `interface_type` | `method_spec_list` |
| 单个 import | `import_spec` | `path` (interpreted_string_literal) |
| 分组 import | `import_declaration` | 多个 `import_spec` |
| 常量声明 | `const_declaration` | `const_spec` |
| 变量声明 | `var_declaration` | `var_spec` |

### Go 特殊处理

1. **导出规则：** 首字母大写的标识符是导出的（Go 语言规范）
2. **init() 函数：** 特殊函数，需要标记为 side effect（包初始化逻辑）
3. **method vs function：** `method_declaration` 带 `receiver`，是类型方法而非独立函数
4. **go.mod：** 文本格式，提取 `module` 行获取包路径，解析 `require` 块获取依赖
5. **blank/dot imports：** `import _ "pkg"` 和 `import . "pkg"` 是特殊导入模式

## Module-Level Aggregation — 无需新依赖

模块级镜像聚合是纯业务逻辑，使用已有类型和标准库：

```
输入: Vec<FileResult>（同一模块下的所有文件解析结果）
输出: ModuleResult

聚合逻辑:
1. 按目录分组文件 → 模块边界
2. 构建模块内部依赖图（文件间 import 关系）
3. 汇总统计：总导出数、内部/外部依赖比
4. 计算模块级语义指纹（聚合所有文件指纹的 SHA-256）
5. 识别模块公共 API（被外部模块 import 的符号）
```

**数据结构（在 wtcd-core/src/types.rs 中新增）：**
```rust
pub struct ModuleResult {
    pub module_id: String,
    pub module_path: String,
    pub files: Vec<FileResult>,
    pub public_exports: Vec<ExportedSymbol>,  // 被外部引用的
    pub internal_edges: Vec<DependencyEdge>,  // 模块内文件间依赖
    pub external_edges: Vec<DependencyEdge>,  // 模块外依赖
    pub aggregate_fingerprint: String,        // 模块级 SHA-256
}
```

## Knowledge Layer — 最小依赖方案

### 方案 A: pulldown-cmark（推荐，零新依赖）

使用已有的 `pulldown-cmark 0.13` 解析 ADR/架构文档：
- 提取 YAML Front Matter（`---` 分隔的头部）→ `yaml_serde`
- 提取标题结构（`#`, `##`）→ pulldown-cmark Event::Start(Tag::Heading)
- 提取段落文本 → pulldown-cmark Event::Text
- **局限：** 不能精确解析表格、嵌套列表等复杂 Markdown 结构

### 方案 B: tree-sitter-md（精确解析，新增一个可选依赖）

使用 `tree-sitter-md 0.5.3` 获得完整的 Markdown AST：
- 精确识别标题级别、表格、代码块、链接引用
- 与项目其他适配器统一使用 tree-sitter 框架
- **代价：** 新增一个依赖，增加编译时间 ~10s
- **注意：** tree-sitter-md 文档明确说明"不推荐用于需要正确性的场景"，但对文档提取足够

**推荐：** 先用方案 A（pulldown-cmark），如果提取精度不足再升级到方案 B。知识层的增量价值不在于精确的 Markdown 解析，而在于交叉引用和结构化组织。

### 知识层数据结构

```rust
pub struct KnowledgeDoc {
    pub doc_type: KnowledgeDocType,      // Adr, Arch, Decision, Rfc
    pub source_path: String,
    pub title: String,
    pub status: String,                   // "Accepted", "Proposed", "Deprecated"
    pub date: Option<String>,
    pub related_modules: Vec<String>,     // 交叉引用的模块 ID
    pub related_files: Vec<String>,       // 交叉引用的文件路径
    pub fingerprint: String,
}

pub enum KnowledgeDocType {
    Adr,       // Architecture Decision Record
    Arch,      // Architecture overview
    Decision,  // Technical decision
    Rfc,       // Request for Comments
}
```

## What NOT to Add

| 避免 | 原因 | 替代 |
|------|------|------|
| **rustpython-parser** | 完整的 Python AST 解析器，但 ~200K SLoC，编译慢，与 tree-sitter 重复 | tree-sitter-python（轻量、统一 API） |
| **syn** | Rust 的 AST 解析器，不是 Python/Go 的 | 不适用 |
| **tree-sitter-rust** | Rust 适配器不在 v0.1.1 里程碑内 | 后续版本按需添加 |
| **pest** / **nom** | 通用解析器组合子，但 tree-sitter 已提供精确的语法解析 | tree-sitter |
| **serde_yml** | 被怀疑 AI 生成代码，社区已弃用 | yaml_serde 0.10（已有） |
| **rayon** | 当前是同步 CLI，文件解析已足够快 | std::thread（如需并行） |
| **walkdir** | 已有 `ignore` crate，功能超集 | ignore 0.4（已有） |

## Installation

```toml
# 新增到 workspace Cargo.toml
[workspace.dependencies]
tree-sitter-python = "0.25.0"
tree-sitter-go = "0.25.0"
tree-sitter-md = { version = "0.5.3", features = ["parser"], optional = true }

# crates/wtcd-adapters/Cargo.toml
[dependencies]
tree-sitter-python = { workspace = true }
tree-sitter-go = { workspace = true }

# crates/wtcd-core/Cargo.toml (如需知识层)
[dependencies]
tree-sitter-md = { workspace = true, optional = true }
```

## Architecture Decisions

### 1. 适配器 crate 结构 — 单 crate 多模块

**决策：** 保持 `wtcd-adapters` 为单个 crate，每种语言一个模块（`ts.rs`, `python.rs`, `go.rs`）。

**理由：**
- 避免每个语言一个 crate 的编译和维护开销
- 适配器逻辑简短（<500 行/语言），不值得独立 crate
- `register_all_adapters()` 函数集中注册，简单清晰

**替代方案考虑：** 每种语言一个 crate（`wtcd-adapter-python`）。若未来语言超过 5 种或适配器逻辑变得复杂（>1000 行），重新评估拆分。

### 2. Knowledge Layer 归属 — 新 crate vs 现有 crate

**决策：** 在 `wtcd-mirror` crate 中新增 `knowledge` 模块，不创建新 crate。

**理由：**
- 知识层文档本质上是一种特殊类型的镜像（模块镜像 → 知识镜像的扩展）
- 复用 `wtcd-mirror` 已有的 YAML Front Matter 处理、指纹计算、文件 I/O
- 保持 workspace 的 crate 数量可控

**替代方案考虑：** 创建 `wtcd-knowledge` crate。若知识层需要独立发布或逻辑超过 1000 行，再拆分。

### 3. tree-sitter-md 作为可选依赖

**决策：** `tree-sitter-md` 设为 optional feature，默认不启用。

**理由：**
- 知识层提取的大部分场景，pulldown-cmark 足够
- tree-sitter-md 增加编译时间和二进制体积
- 用户可通过 feature flag 按需启用

---

## Sources

- crates.io: `tree-sitter-python` 0.25.0 — version, deps, downloads (HIGH)
- crates.io: `tree-sitter-go` 0.25.0 — version, deps, downloads (HIGH)
- crates.io: `tree-sitter-md` 0.5.3 — version, deps, downloads (HIGH)
- crates.io: `tree-sitter` 0.26.7 — current core version (HIGH)
- Context7: `/tree-sitter/tree-sitter-python` — Python AST node types, grammar structure (HIGH)
- Context7: `/tree-sitter/tree-sitter-go` — Go AST node types, grammar structure (HIGH)
- GitHub: tree-sitter/tree-sitter#3069 — tree-sitter-language crate architecture (HIGH)
- GitHub: tree-sitter/tree-sitter-python/node-types.json — complete Python node types (HIGH)
- GitHub: tree-sitter/tree-sitter-go/test/corpus — Go AST structure examples (HIGH)
- WebSearch: "tree-sitter 0.26 compatible tree-sitter-python 0.25" — ABI compatibility (MEDIUM)

---
*Stack research for: ANRSM v0.1.1 — Multi-Language & Knowledge Layer*
*Researched: 2026-03-21*
*Next review: After Python adapter MVP, validate tree-sitter-python extraction quality*
