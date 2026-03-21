# Architecture Research: Multi-Language & Knowledge Layer Integration

**Domain:** Language adapter integration + module aggregation + knowledge generation
**Researched:** 2026-03-21
**Confidence:** HIGH

## Executive Summary

现有 WTCD 架构是一条设计良好的流水线：**scan → parse → mirror → index → diff/gate**。四个新特性（Python 适配器、Go 适配器、模块级聚合、知识层）均能干净地融入这条流水线，遵循已建立的 trait-based 模式。关键架构变更：

1. **Python & Go 适配器** — 纯增量，扩展 `wtcd-adapters`，实现现有 `LanguageAdapter` trait
2. **模块级聚合** — 文件镜像与路由索引之间的新中间层，需要新类型和新输出目录
3. **知识层** — 路由索引 + 模块镜像的下游消费者，生成仓库级语义文档

构建顺序受约束：适配器必须先行（产出 `FileResult` 供所有后续消费），然后模块聚合，最后知识层。

---

## Current Architecture (Baseline)

### System Overview

```
┌─────────────────────────────────────────────────────────────┐
│                     CLI (wtcd-cli)                           │
│  init │ run │ check │ route │ mcp                            │
└──────┬──────────────┬──────────────┬─────────────────────────┘
       │              │              │
┌──────▼──────┐ ┌─────▼──────┐ ┌────▼───────┐
│  wtcd-scope │ │ wtcd-adapt │ │ wtcd-diff  │
│  scan files │ │ TS parser  │ │ git diff   │
│  (ext list) │ │ tree-sitter│ │ classify   │
└──────┬──────┘ └─────┬──────┘ └────┬───────┘
       │              │              │
       └──────────────┼──────────────┘
                      │
              ┌───────▼────────┐
              │   FileResult   │  ← core types (wtcd-core)
              │  exports       │
              │  imports       │
              │  signatures    │
              │  side_effects  │
              └───────┬────────┘
                      │
         ┌────────────┼────────────┐
         │            │            │
   ┌─────▼─────┐ ┌───▼────┐ ┌────▼──────┐
   │ wtcd-mirror│ │ index  │ │ depgraph  │
   │ file .md   │ │ route  │ │ reverse   │
   │ fingerprint│ │ query  │ │ edges     │
   └────────────┘ └────────┘ └───────────┘
```

### Component Responsibilities

| Component | Responsibility | Interface |
|-----------|---------------|-----------|
| `wtcd-core` | Shared types, adapter trait, index builder, depgraph | `LanguageAdapter` trait, `FileResult`, `RoutingIndex` |
| `wtcd-scope` | File discovery, config loading | `scan()`, `load_config()` |
| `wtcd-adapters` | Language-specific parsers | `TsAdapter` → `FileResult` |
| `wtcd-mirror` | File-level mirror generation (Markdown + YAML) | `generate_and_write_mirror()` |
| `wtcd-diff` | Git diff analysis, drift detection, CI gate | `diff_working_tree_vs_head()`, `classify_changes()` |
| `wtcd-core::index` | Routing index build and query | `build_routing_index()`, `route_query()` |
| `wtcd-cli` | Orchestration, JSON output | `run_analysis()` pipeline |

### Current Pipeline (run command)

```
1. load_config (wtcd-scope)
2. register_all_adapters (wtcd-adapters) → AdapterRegistry
3. scan (wtcd-scope) → Vec<PathBuf>
4. for each file: registry.find_adapter → adapter.parse() → FileResult
5. generate_and_write_mirror (wtcd-mirror) → mirror/file/*.md
6. build_routing_index (wtcd-core) → routing_index.json
7. JSON output
```

### Existing Trait Contract

```rust
pub trait LanguageAdapter: Send + Sync {
    fn language_name(&self) -> &str;
    fn file_extensions(&self) -> &[&str];
    fn parse(&self, source: &str, file_path: &str) -> FileResult;
}
```

`AdapterRegistry::find_adapter()` matches by file extension. `register_all_adapters()` is the single registration point.

---

## Integration Points for New Features

### 1. Python Adapter — `crates/wtcd-adapters/src/py.rs`

**Integration type:** Pure addition. No existing code changes to trait or registry logic.

**What exists for this:**
- `LanguageAdapter` trait — already defined, already used by `TsAdapter`
- `AdapterRegistry::find_adapter()` — matches by extension, extensible by design
- `tree-sitter-python` crate — available on crates.io, same API as `tree-sitter-typescript`
- `register_all_adapters()` — single function to add to

**What needs to change:**

| Location | Change | Type |
|----------|--------|------|
| `wtcd-adapters/src/py.rs` | `PyAdapter` struct implementing `LanguageAdapter` | **NEW** |
| `wtcd-adapters/src/lib.rs` | `pub mod py;` + register in `register_all_adapters()` | **MODIFY** |
| `wtcd-adapters/Cargo.toml` | `tree-sitter-python` dependency | **MODIFY** |
| `Cargo.toml` (workspace) | `tree-sitter-python` workspace dependency | **MODIFY** |
| `wtcd-scope/src/scanner.rs` | Add `"py"` to `SUPPORTED_EXTENSIONS` | **MODIFY** |
| `wtcd-core::types::ExportKind` | Add `Module`, `Function` variants for Python top-level defs | **MODIFY** |

**Python extraction mapping:**

| TS/JS Concept | Python Equivalent | tree-sitter node |
|--------------|-------------------|-----------------|
| `export function` | Top-level `def` / `async def` | `function_definition` |
| `export class` | Top-level `class` | `class_definition` |
| `export const` | Top-level assignment | `assignment` |
| `import { x } from` | `from x import y` | `import_from_statement` |
| `import x` | `import x` | `import_statement` |
| `console.log()` | `print()` | `call` → `identifier` = `print` |
| `fs.open()` | `open()` | `call` → `identifier` = `open` |
| `fetch()` | `requests.get()` | `call` → `attribute` starts with `requests.` |

**Side effects table (Python-specific):**

| API Pattern | Kind |
|------------|------|
| `print`, `pprint` | Log |
| `open` | Io |
| `requests.*`, `httpx.*`, `urllib.*` | Network |
| `logging.*` | Log |
| `sqlite3`, `psycopg`, `redis.*` | Storage |
| `os.remove`, `shutil.*`, `subprocess.*` | Io |

**Confidence:** Same model as TS adapter — tree-sitter error recovery → `Low`, complete failure → `None`.

---

### 2. Go Adapter — `crates/wtcd-adapters/src/go.rs`

**Integration type:** Pure addition. Same pattern as Python adapter.

**What needs to change:**

| Location | Change | Type |
|----------|--------|------|
| `wtcd-adapters/src/go.rs` | `GoAdapter` struct implementing `LanguageAdapter` | **NEW** |
| `wtcd-adapters/src/lib.rs` | `pub mod go;` + register | **MODIFY** |
| `wtcd-adapters/Cargo.toml` | `tree-sitter-go` dependency | **MODIFY** |
| `Cargo.toml` (workspace) | `tree-sitter-go` workspace dependency | **MODIFY** |
| `wtcd-scope/src/scanner.rs` | Add `"go"` to `SUPPORTED_EXTENSIONS` | **MODIFY** |

**Go extraction mapping:**

| TS/JS Concept | Go Equivalent | tree-sitter node |
|--------------|---------------|-----------------|
| `export function` | Uppercase `func Name()` | `function_declaration` |
| `export class` | `type Name struct` (uppercase) | `type_declaration` + `struct_type` |
| `export interface` | `type Name interface` (uppercase) | `type_declaration` + `interface_type` |
| `export const` | `const Name` (uppercase) | `const_declaration` |
| `import { x } from` | `import "pkg/path"` | `import_spec` |
| `console.log()` | `fmt.Println()` | `call_expression` with `selector_expression` |

**Key difference:** Go uses **capitalization** to determine export visibility, not keywords. The adapter must check `name.starts_with(|c: char| c.is_uppercase())` for export detection.

**Side effects table (Go-specific):**

| API Pattern | Kind |
|------------|------|
| `fmt.Print*`, `log.*` | Log |
| `os.Open`, `os.Create`, `os.Remove` | Io |
| `io/ioutil.ReadFile` | Io |
| `http.*` (Get, Post, etc.) | Network |
| `database/sql.*` | Storage |
| `redis.*` | Storage |

**NOTE:** `ExportKind` in `wtcd-core::types` already has `Function`, `Class`, `Interface`, `Enum` variants that map well. May need `Struct` variant for Go `type X struct`.

---

### 3. Module-Level Mirror Aggregation

**Integration type:** New layer between file mirrors and routing index. Most architecturally significant addition.

Current pipeline:
```
file parse → file mirror → routing index
```

New pipeline:
```
file parse → file mirror → module aggregation → module mirror → routing index (enhanced)
```

#### New Types Required

**In `wtcd-core::types`:**

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModuleResult {
    pub module_id: String,
    pub files: Vec<String>,                    // source paths in this module
    pub aggregated_exports: Vec<ExportedSymbol>,
    pub aggregated_imports: Vec<DependencyEdge>,  // cross-module only, deduped
    pub aggregated_side_effects: Vec<SideEffect>,
    pub file_count: usize,
    pub confidence: ConfidenceBand,            // worst-case across files
}
```

**In `wtcd-mirror::types`:**

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModuleMirrorHeader {
    pub anrsm_version: u32,
    pub artifact_type: String,           // "module_mirror"
    pub artifact_id: String,             // "module_mirror:auth"
    pub module_id: String,
    pub file_count: usize,
    pub source_commit: String,
    pub semantic_fingerprint: String,    // hash of aggregated semantic
    pub freshness_state: FreshnessState,
    pub confidence_band: MirrorConfidenceBand,
    pub generator_name: String,
    pub generator_version: String,
    pub generated_at: String,
    pub member_files: Vec<String>,
    pub exports: Option<Vec<String>>,
    pub dependencies: Option<Vec<String>>,
    pub risk_tags: Option<Vec<String>>,
}
```

#### Data Flow

```
Vec<FileResult>  (from parse step)
       │
       ▼
┌──────────────────────┐
│ group_by_module_id() │  ← NEW in wtcd-core
│                      │
│ HashMap<module_id,   │
│   Vec<FileResult>>   │
└──────────┬───────────┘
           │
           ▼
┌──────────────────────────┐
│ aggregate_module_result()│  ← NEW in wtcd-core
│                          │
│ - union exports          │
│ - dedup imports          │
│ - merge side effects     │
│ - worst-case confidence  │
└──────────┬───────────────┘
           │
           ▼
┌──────────────────────────┐
│ generate_module_mirror() │  ← NEW in wtcd-mirror
│                          │
│ output: mirror/module/   │
│   <module_id>.md         │
└──────────┬───────────────┘
           │
           ▼
┌──────────────────────────┐
│ enhance_routing_index()  │  ← NEW in wtcd-core::index
│                          │
│ artifact_id:             │
│   "module_mirror:<id>"   │
└──────────────────────────┘
```

#### Path Convention

```
mirror/
├── file/                    # existing — per-file mirrors
│   ├── src/auth/login.ts.md
│   └── src/utils/format.ts.md
├── module/                  # NEW — per-module mirrors
│   ├── auth.md
│   └── utils.md
├── knowledge/               # NEW — knowledge layer
│   ├── architecture.md
│   └── api-surface.md
└── routing_index.json       # enhanced with module entries
```

#### What Needs to Change

| Location | Change | Type |
|----------|--------|------|
| `wtcd-core::types` | Add `ModuleResult` struct | **NEW** |
| `wtcd-core::index` | Add `build_enhanced_routing_index()` with module entries | **MODIFY** |
| `wtcd-mirror::types` | Add `ModuleMirrorHeader`, `ModuleMirrorBody` | **NEW** |
| `wtcd-mirror::template` | Add `generate_module_mirror()`, body template, YAML serializer | **NEW** |
| `wtcd-mirror::io` | Add `generate_and_write_module_mirror()` | **NEW** |
| `wtcd-core::config::MirrorConfig` | Add `module_output_dir` field (default: `"mirror/module"`) | **MODIFY** |
| `wtcd-cli::commands::run.rs` | Insert module aggregation + module mirror generation after file mirrors | **MODIFY** |

#### Module Body Sections (Adapted from File Mirror)

| File Mirror Section | Module Mirror Adaptation |
|--------------------|--------------------------|
| 职责 (Responsibilities) | Module's collective purpose: all exported symbols across files |
| 对外契约 (External Contracts) | Combined exported function signatures from all files |
| 输入与前置条件 (Preconditions) | Cross-module dependencies (external packages + other modules) |
| 状态与控制流 (State & Control Flow) | Summary of side effects across all files |
| 副作用与外部依赖 (Side Effects) | Aggregated + deduplicated side effects |
| 关键不变量与风险 (Invariants & Risks) | Risk tags, confidence summary, low-confidence files flagged |
| 变更影响 (Change Impact) | Other modules depending on this module (from depgraph) |
| 何时必须展开源码 (Expand Conditions) | When to drill into specific files vs reading module summary |

#### Aggregation Logic

```rust
pub fn group_by_module_id(
    results: &[FileResult],
    module_ids: &HashMap<String, String>,
) -> HashMap<String, Vec<&FileResult>> {
    let mut groups = HashMap::new();
    for result in results {
        let module_id = module_ids.get(&result.file_path)
            .cloned()
            .unwrap_or_else(|| "global".to_string());
        groups.entry(module_id).or_default().push(result);
    }
    groups
}

pub fn aggregate_module(module_id: &str, files: &[&FileResult]) -> ModuleResult {
    // Union all exports
    // Dedup imports by source path (keep cross-module only)
    // Merge side effects, dedup by kind+target
    // Confidence = worst case across all files
}
```

---

### 4. Knowledge Layer Document Generation

**Integration type:** New downstream consumer. Depends on routing index + module mirrors.

The knowledge layer produces repository-level semantic documents. Unlike file/module mirrors (mechanical extraction), knowledge documents synthesize patterns across the entire codebase.

#### Knowledge Document Types

| Document | Content | Sources |
|----------|---------|---------|
| `architecture.md` | Module relationships, dependency graph, data flow | Module mirrors, depgraph |
| `api-surface.md` | Public API surface, exported symbols, contract summary | Routing index, export aggregation |
| `risk-map.md` | Risk tag aggregation, high-risk modules, side effect chains | Module mirror risk_tags |

#### New Components

Knowledge generation lives in `wtcd-mirror::knowledge` (keeps mirror generation together):

```
wtcd-mirror/src/
├── lib.rs
├── types.rs         # existing + add KnowledgeHeader
├── template.rs      # existing file/module mirror templates
├── fingerprint.rs   # existing
├── io.rs            # existing + add knowledge I/O
└── knowledge/       # NEW module
    ├── mod.rs       # orchestrator
    ├── analysis.rs  # module depgraph, risk aggregation, entry point detection
    └── templates.rs # knowledge document body templates
```

#### Data Flow

```
routing_index.json  +  mirror/module/*.md  +  depgraph
                          │
                          ▼
               ┌─────────────────────┐
               │ knowledge_generator │  ← NEW
               │                     │
               │ Pass 1: Collect     │
               │   read all module   │
               │   mirrors + index   │
               │                     │
               │ Pass 2: Analyze     │
               │   module depgraph   │
               │   risk aggregation  │
               │   entry points      │
               │   data flow chains  │
               │                     │
               │ Pass 3: Generate    │
               │   architecture.md   │
               │   api-surface.md    │
               │   risk-map.md       │
               └─────────────────────┘
```

#### Knowledge Document Structure

```yaml
---
anrsm_version: 1
artifact_type: knowledge
artifact_id: knowledge:architecture
source_commit: abc1234
generated_at: 2026-03-21T...
generator_name: anrsm
generator_version: 0.1.1
scope: repository
module_count: 5
file_count: 42
---

## Module Map
[description of each module and its role]

## Dependency Graph
[cross-module dependency relationships]

## Data Flow
[how data enters, transforms, and exits the system]

## Risk Summary
[aggregated risk tags, high-risk modules]
```

#### What Needs to Change

| Location | Change | Type |
|----------|--------|------|
| `wtcd-mirror::knowledge` | New module: `mod.rs`, `analysis.rs`, `templates.rs` | **NEW** |
| `wtcd-mirror::types` | Add `KnowledgeHeader` struct | **NEW** |
| `wtcd-mirror/src/lib.rs` | `pub mod knowledge;` | **MODIFY** |
| `wtcd-core::config::MirrorConfig` | Add `knowledge_output_dir` (default: `"mirror/knowledge"`) | **MODIFY** |
| `wtcd-cli::commands::run.rs` | Add knowledge generation step after module aggregation | **MODIFY** |
| `wtcd-mcp::tools` | Add `knowledge` MCP tool (optional) | **NEW** |

---

## Anti-Patterns to Avoid

### Anti-Pattern 1: Duplicating Adapter Logic

**What people do:** Copy the entire `TsAdapter` and change syntax patterns.
**Why it's wrong:** Maintenance burden doubles. Shared traversal logic diverges.
**Do this instead:** Extract common tree traversal into shared helper. Each adapter only implements node matching.

### Anti-Pattern 2: Knowledge Layer as Replacement for File Mirrors

**What people do:** Skip file mirrors, generate knowledge docs directly from source.
**Why it's wrong:** Knowledge docs are high-level summaries. Agents still need file mirrors for precise navigation. Skipping mirrors loses drift detection.
**Do this instead:** Knowledge layer is additive. File mirrors → Module mirrors → Knowledge docs. Each layer adds abstraction, never replaces.

### Anti-Pattern 3: Hard-Coding Module Boundaries

**What people do:** Assume modules = directories.
**Why it's wrong:** Some repos use flat structures, others deeply nested.
**Do this instead:** Keep `module_id` derivation flexible. Add optional `module_mapping` config in `anrsm.yaml` for explicit overrides.

### Anti-Pattern 4: Adding Re-Parse to the Pipeline

**What people do:** Make module aggregation require a second parse pass.
**Why it's wrong:** Parsing is the most expensive step. Doubles execution time.
**Do this instead:** Module aggregation operates on `Vec<FileResult>` (already parsed). No re-parsing needed.

---

## Build Order & Dependencies

```
Phase 1: Language Adapters (Python + Go in parallel)
  ├── Depends on: tree-sitter-python, tree-sitter-go crates
  ├── Produces: FileResult (same as TS adapter)
  ├── Blocks: nothing (additive)
  └── Touches: wtcd-adapters, wtcd-scope, wtcd-core::types

Phase 2: Module-Level Aggregation
  ├── Depends on: Phase 1 (needs multi-language FileResults to validate)
  ├── Produces: mirror/module/*.md, enhanced routing_index.json
  ├── Blocks: Phase 3
  └── Touches: wtcd-core::types, wtcd-mirror, wtcd-core::index, wtcd-cli

Phase 3: Knowledge Layer
  ├── Depends on: Phase 2 (needs module mirrors + enhanced index)
  ├── Produces: mirror/knowledge/*.md
  ├── Blocks: nothing
  └── Touches: wtcd-mirror::knowledge, wtcd-cli, wtcd-mcp
```

### Phase 1 Detail: Language Adapters

**Sub-phase 1a: Python adapter** (parallel with 1b)
1. Add `tree-sitter-python` to workspace Cargo.toml
2. Create `crates/wtcd-adapters/src/py.rs` with `PyAdapter`
3. Extend `ExportKind` in `wtcd-core::types` with `Module` variant
4. Register in `wtcd-adapters/src/lib.rs`
5. Add `"py"` to `SUPPORTED_EXTENSIONS` in scanner
6. Integration tests with Python fixtures

**Sub-phase 1b: Go adapter** (parallel with 1a)
1. Add `tree-sitter-go` to workspace Cargo.toml
2. Create `crates/wtcd-adapters/src/go.rs` with `GoAdapter`
3. Register in `wtcd-adapters/src/lib.rs`
4. Add `"go"` to `SUPPORTED_EXTENSIONS` in scanner
5. Integration tests with Go fixtures

### Phase 2 Detail: Module Aggregation

1. Add `ModuleResult` type to `wtcd-core::types`
2. Add `group_by_module_id()` and `aggregate_module()` to `wtcd-core`
3. Add `ModuleMirrorHeader`, `ModuleMirrorBody` to `wtcd-mirror::types`
4. Add `generate_module_mirror()`, body template, YAML serializer to `wtcd-mirror::template`
5. Add `generate_and_write_module_mirror()` to `wtcd-mirror::io`
6. Add `module_output_dir` to `MirrorConfig`
7. Extend `wtcd-cli::commands::run.rs` pipeline:
   - After file mirrors: group → aggregate → generate module mirrors
8. Enhance routing index with module-level entries
9. Integration tests

### Phase 3 Detail: Knowledge Layer

1. Add `KnowledgeHeader` type to `wtcd-mirror::types`
2. Create `wtcd-mirror::knowledge` module:
   - `mod.rs` — orchestrator: collect → analyze → generate
   - `analysis.rs` — module depgraph, risk aggregation, entry point detection
   - `templates.rs` — knowledge document body templates
3. Add `knowledge_output_dir` to `MirrorConfig`
4. Extend `wtcd-cli::commands::run.rs` pipeline:
   - After module mirrors: generate knowledge documents
5. Add `knowledge` MCP tool to `wtcd-mcp`
6. Integration tests with multi-module fixture repos

---

## Sources

- Existing codebase: `wtcd-core::adapter`, `wtcd-adapters::ts`, `wtcd-mirror::template`, `wtcd-cli::commands::run`
- tree-sitter-python: https://crates.io/crates/tree-sitter-python
- tree-sitter-go: https://crates.io/crates/tree-sitter-go
- tree-sitter API: https://docs.rs/tree-sitter

---

*Architecture research for: WTCD multi-language & knowledge layer milestone*
*Researched: 2026-03-21*
