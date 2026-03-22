# Architecture Research

**Domain:** WTCD 多语言适配器集成（Rust/Dart/Java/Kotlin/Swift/C++/C#/C/Zig）
**Researched:** 2026-03-22
**Confidence:** HIGH

## Standard Architecture

### System Overview

```text
┌──────────────────────────────────────────────────────────────────────┐
│                          wtcd-cli pipeline                           │
│                       run / check / route                            │
└───────────────┬──────────────────────────────┬───────────────────────┘
                │                              │
        ┌───────▼────────┐             ┌───────▼─────────────────┐
        │ wtcd-scope     │             │ wtcd-adapters           │
        │ scan()         │             │ AdapterRegistry         │
        │ (按扩展名筛选) │             │ + LanguageAdapter impls │
        └───────┬────────┘             └───────┬─────────────────┘
                │                              │
                │ files_to_parse               │ parse(source,path)
                └──────────────┬───────────────┘
                               ▼
                      ┌───────────────────┐
                      │ FileResult[]      │
                      │ exports/imports/  │
                      │ signatures/effects│
                      └───────┬───────────┘
                              │
          ┌───────────────────┼─────────────────────┐
          ▼                   ▼                     ▼
  ┌───────────────┐   ┌──────────────┐     ┌────────────────────┐
  │ wtcd-mirror   │   │ wtcd-core    │     │ wtcd-diff/depgraph │
  │ file/module/  │   │ routing_index│     │ affected expansion  │
  │ knowledge docs│   │ route query  │     │ drift classification│
  └───────────────┘   └──────────────┘     └────────────────────┘
```

### Component Responsibilities

| Component | Responsibility | Typical Implementation |
|-----------|----------------|------------------------|
| `LanguageAdapter` trait | 统一语言解析接口 | `language_name/file_extensions/parse` |
| `AdapterRegistry` | 扩展名→适配器路由 | `Vec<Box<dyn LanguageAdapter>>` + `find_adapter` |
| `wtcd-adapters` | 新语言 AST 解析与语义抽取 | 每种语言一个 `*.rs` 适配器文件 + `register_all_adapters()` |
| `wtcd-scope::scanner` | 扫描进入管线的文件 | `SUPPORTED_EXTENSIONS` 白名单 |
| `wtcd-core::depgraph` | 增量模式影响面扩展 | 基于 `imports` 构建 reverse graph |
| `wtcd-core::index` | route 检索索引 | 从 `FileResult[]` 聚合关键词/导出/风险 |

## Recommended Project Structure

```text
crates/
├── wtcd-adapters/
│   ├── src/
│   │   ├── lib.rs            # 统一注册入口（MODIFY）
│   │   ├── ts.rs             # 已有
│   │   ├── py.rs             # 已有
│   │   ├── go.rs             # 已有
│   │   ├── rust.rs           # NEW
│   │   ├── dart.rs           # NEW
│   │   ├── java.rs           # NEW
│   │   ├── kotlin.rs         # NEW
│   │   ├── swift.rs          # NEW
│   │   ├── cpp.rs            # NEW
│   │   ├── csharp.rs         # NEW
│   │   ├── c.rs              # NEW
│   │   └── zig.rs            # NEW
│   └── Cargo.toml            # 新增 tree-sitter grammar 依赖（MODIFY）
├── wtcd-scope/src/scanner.rs # 扩展扫描白名单（MODIFY）
├── wtcd-core/
│   ├── src/types.rs          # （建议）增加 file.language 字段（MODIFY）
│   └── src/depgraph.rs       # 语言感知 import 解析（MODIFY）
└── wtcd-tests/
    ├── tests/integration_*.rs    # 每语言集成回归（NEW/MODIFY）
    └── ../../tests/fixtures/     # 每语言 fixture（NEW）
```

### Structure Rationale

- **`wtcd-adapters/src/<lang>.rs`：** 每语言单文件，职责收敛，便于并行开发与回归。
- **`lib.rs` 单注册入口：** 保持现有“集中装配”架构，不引入动态插件系统，避免里程碑范围膨胀。
- **`wtcd-core` 最小改动：** 仅修正跨语言增量依赖/路径解析问题，不重写 run/check/route 主流程。

## Architectural Patterns

### Pattern 1: Adapter-per-language（保持现有主模式）

**What:** 每种语言实现同一 `LanguageAdapter` trait，接入同一注册表。  
**When to use:** 所有新增语言都应走该模式。  
**Trade-offs:** 开发速度快、改动面小；但共享抽取逻辑少，重复代码会增加。

**Example:**
```rust
pub struct RustAdapter { parser: Mutex<Parser> }

impl LanguageAdapter for RustAdapter {
    fn language_name(&self) -> &str { "rust" }
    fn file_extensions(&self) -> &[&str] { &["rs"] }
    fn parse(&self, source: &str, file_path: &str) -> FileResult { /* ... */ }
}
```

### Pattern 2: Language-aware import normalization（建议新增）

**What:** 在 depgraph 侧把 `imports.source` 规范化为“可比较键”，按语言规则解析。  
**When to use:** `run` 增量模式与 `check` 漂移分析要跨语言稳定工作时。  
**Trade-offs:** 需要额外规则表；但能避免当前 TS 专用解析逻辑污染其他语言。

**Example:**
```rust
fn resolve_import_path(importer: &str, raw: &str, lang: &str) -> String {
    match lang {
        "typescript" | "javascript" => resolve_ts_like(importer, raw),
        "python" => resolve_python_like(importer, raw),
        "rust" => resolve_rust_use(raw),
        _ => raw.to_string(),
    }
}
```

### Pattern 3: Progressive extraction contract（统一输出，不追求首版全语义）

**What:** 先保证 `exports/imports/signatures/side_effects` 四元组稳定输出，再逐语言提高覆盖率。  
**When to use:** 9 语言同里程碑交付、需要控制风险时。  
**Trade-offs:** 初期精度可能不均；但能确保 pipeline 先打通。

## Data Flow

### Request Flow（run/check）

```text
scan(source_roots)
  ↓
按扩展名筛选文件
  ↓
AdapterRegistry.find_adapter(path)
  ↓
adapter.parse(source, path) -> FileResult
  ↓
mirror/index/depgraph/diff 复用既有下游
```

### State Management

```text
核心状态载体: Vec<FileResult>
  ├── 供镜像层生成 file/module/knowledge
  ├── 供 index 层生成 routing_index
  └── 供 depgraph/diff 做增量与漂移判断
```

### Key Data Flows（新增语言后的关键变化）

1. **扫描流变化：** `wtcd-scope::SUPPORTED_EXTENSIONS` 从当前 `ts/tsx/js/jsx` 扩展到 13 种扩展（含已有 py/go）。
2. **解析流变化：** `register_all_adapters()` 增加 9 个 adapter 注册，`find_adapter` 命中率显著提升。
3. **增量流变化（必须修正）：** `wtcd-core::depgraph::resolve_import_path()` 目前是 TS/JS 规则，需改成 language-aware，否则新增语言在 `run --incremental` / `check` 的影响面扩展会失真。

## Scaling Considerations

| Scale | Architecture Adjustments |
|-------|--------------------------|
| 0-1k files | 维持当前串行 parse 即可，重点保证正确性 |
| 1k-20k files | 按语言/文件分片并行 parse（线程池），但输出顺序保持稳定排序 |
| 20k+ files | 引入 parser 池与增量缓存（按文件 hash），避免每次全量重建 depgraph |

### Scaling Priorities

1. **第一瓶颈：** parse CPU 时间（9 语言后更明显）→ 先做并行 parse。  
2. **第二瓶颈：** 增量依赖误判导致“伪全量”→ 优先做 language-aware depgraph 解析。

## Anti-Patterns

### Anti-Pattern 1: “只加 adapter，不改 scanner”

**What people do:** 新增了适配器和注册，但没把扩展名加入 `wtcd-scope::SUPPORTED_EXTENSIONS`。  
**Why it's wrong:** 文件根本进不了 parse 管线，表现为“adapter 不生效”。  
**Do this instead:** 扫描白名单与 adapter 注册必须同一 PR 原子提交。

### Anti-Pattern 2: 继续用 TS-only depgraph 解析所有语言

**What people do:** 复用 `resolve_import_path` 里 `.ts/.tsx/.js/.jsx` 逻辑处理 Java/Rust/C# 等。  
**Why it's wrong:** `check` 漂移级别和 `run --incremental` 受影响文件会系统性偏差。  
**Do this instead:** 引入 language-aware import normalization（至少按语言族分流）。

## Integration Points

### External Services / Dependencies

| Service | Integration Pattern | Notes |
|---------|---------------------|-------|
| crates.io tree-sitter grammars | 在 workspace + `wtcd-adapters` 增加依赖 | 当前可确认存在：`tree-sitter-rust/java/kotlin/swift/cpp/c-sharp/c/zig/dart` |
| tree-sitter runtime | 统一 `Parser::new() + set_language() + parse()` | 与现有 ts/py/go 适配器一致 |

### Internal Boundaries

| Boundary | Communication | Notes |
|----------|---------------|-------|
| `wtcd-scope` ↔ `wtcd-adapters` | 文件路径 + 扩展名约定 | 扫描白名单必须覆盖全部新语言扩展 |
| `wtcd-adapters` ↔ `wtcd-core::types` | `FileResult` 结构体 | （建议）增加 `language` 字段，降低下游猜测成本 |
| `wtcd-adapters` ↔ `wtcd-core::depgraph` | `imports.source` 语义 | 需要语言感知规范化，避免误扩散/漏扩散 |
| `wtcd-cli run/check/route` ↔ registry | `register_all_adapters()` | 入口已统一，无需新增命令 |

## New vs Modified（显式清单）

| Area | NEW | MODIFY |
|------|-----|--------|
| 语言适配器 | `rust.rs/dart.rs/java.rs/kotlin.rs/swift.rs/cpp.rs/csharp.rs/c.rs/zig.rs` | `wtcd-adapters/src/lib.rs` 注册 9 语言 |
| 依赖 | 各 grammar crate 依赖项 | 根 `Cargo.toml` + `wtcd-adapters/Cargo.toml` |
| 扫描入口 | - | `wtcd-scope/src/scanner.rs` 扩展白名单 |
| 核心数据流 | （建议）`FileResult.language` | `wtcd-core/src/types.rs` |
| 增量影响分析 | （建议）`import normalizer` 子模块 | `wtcd-core/src/depgraph.rs` |
| 测试资产 | 9 语言 fixtures + integration tests | 现有 `wtcd-tests` 聚合测试入口 |

## Suggested Build Order（按依赖排序）

1. **基础接线层（必须先做）**
   - 更新 Cargo 依赖（workspace + wtcd-adapters）
   - 扩展 `scanner` 白名单
   - 在 `register_all_adapters()` 预留注册位（可先 `todo!`）

2. **适配器实现层（可并行）**
   - 并行批次 A：Rust / C / C++ / Zig（语法族接近，风险较低）
   - 并行批次 B：Java / Kotlin / C# / Swift / Dart（面向对象语义多，抽取规则更复杂）
   - 每个适配器完成即补对应 fixture + integration test

3. **核心一致性层（串行，收口）**
   - 修正 `depgraph` 为 language-aware（至少去掉 TS-only 假设）
   - （可选但推荐）给 `FileResult` 加 `language` 字段，减少下游推断

4. **端到端验证层**
   - `wtcd run`：验证多语言文件可产出 mirror/index
   - `wtcd check`：验证新增语言改动可正确进 drift/gate
   - `wtcd route`：验证关键词和导出检索仍可工作

> 依赖关系：2 依赖 1；3 依赖 2（至少有最小可运行适配器集）；4 依赖 2+3。

## Sources

- Repo code (HIGH)
  - `crates/wtcd-core/src/adapter.rs`
  - `crates/wtcd-cli/src/commands/run.rs`
  - `crates/wtcd-cli/src/commands/check.rs`
  - `crates/wtcd-core/src/depgraph.rs`
  - `crates/wtcd-scope/src/scanner.rs`
  - `crates/wtcd-adapters/src/{ts,py,go}.rs`
  - `crates/wtcd-core/src/config.rs`
- crates.io/docs.rs via cargo docs RAG (MEDIUM-HIGH)
  - `tree-sitter` 0.26.7
  - `tree-sitter-rust` 0.24.1
  - `tree-sitter-java` 0.23.5
  - `tree-sitter-kotlin` 0.3.8
  - `tree-sitter-swift` 0.7.1
  - `tree-sitter-cpp` 0.23.4
  - `tree-sitter-c-sharp` 0.23.1
  - `tree-sitter-c` 0.24.1
  - `tree-sitter-zig` 1.1.2
  - `tree-sitter-dart` 0.1.0

---
*Architecture research for: WTCD v0.2.0 polyglot adapter integration*
*Researched: 2026-03-22*
