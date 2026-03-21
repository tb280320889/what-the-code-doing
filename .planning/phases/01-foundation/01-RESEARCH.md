# Phase 1 Research: Foundation

**Phase:** 01-foundation
**Date:** 2026-03-21
**Status:** RESEARCH COMPLETE

---

## 1. Technical Stack Validation

### tree-sitter Integration (Rust)

**Current versions (2026-03):**
- `tree-sitter` 0.26.x — MSRV 1.77, stable API
- `tree-sitter-typescript` 0.23.2 — MSRV not explicitly stated, depends on `tree-sitter-language ^0.1`
- `tree-sitter-javascript` — ships separately but in same `tree-sitter-typescript` crate via `LANGUAGE_TYPESCRIPT` and `LANGUAGE_TSX` constants

**API Pattern (tree-sitter 0.24+):**
```rust
use tree_sitter::{Parser, Language};
// Modern API:
let lang: Language = tree_sitter_typescript::LANGUAGE_TYPESCRIPT.into();
parser.set_language(&lang).expect("...");
let tree = parser.parse(source_code, None).unwrap();
```

**Key notes:**
- `tree-sitter-typescript` crate provides both TS and TSX grammars via separate constants
- For JS files, use `tree_sitter_javascript::LANGUAGE` (separate crate or bundled)
- Tree cursor API for efficient traversal — `PreorderCursor` preferred for large files
- Error recovery: parser always produces a tree (no panics), broken nodes marked with `is_error()` and `has_error()` flags
- Query API (`tree_sitter::Query`) supports S-expression patterns for targeted extraction

**Recommendation:** Use `tree-sitter 0.26` + `tree-sitter-typescript 0.23` + `tree-sitter-javascript 0.23`. Both grammars are in separate crates but have compatible APIs. For JS support, either use the TS parser (which handles JS fine) or add `tree-sitter-javascript` separately.

### ignore crate (File Scanning)

**Current version:** 0.4.18

**Key API:**
```rust
use ignore::WalkBuilder;
let walker = WalkBuilder::new(root_path)
    .git_ignore(true)
    .hidden(true)
    .build();
```

**Glob matching:** Use `globset` crate (dependency of `ignore`) for custom include/exclude patterns:
```rust
use globset::{GlobSetBuilder, Glob};
let mut builder = GlobSetBuilder::new();
builder.add(Glob::new("**/*.ts")?);
builder.add(Glob::new("!**/*.test.ts")?);
let set = builder.build()?;
```

**Key behavior:** WalkBuilder skips entire ignored directories (doesn't descend). For custom glob matching on top of gitignore, iterate `Walk` results and apply `GlobSet::is_match()`.

### Clap CLI

**Version:** 4.6 (derive API)
**MSRV:** 1.85 — note this may conflict if targeting older Rust
**Alternative:** clap 4.5 if MSRV < 1.85

**Subcommand pattern:**
```rust
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "wtcd", version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Init {},
    Run {},
}
```

### Configuration (yaml_serde)

**Version:** yaml_serde 0.10 (YAML org maintained, successor to deprecated serde_yaml)

**Deserialization pattern:**
```rust
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
struct Config {
    version: u32,
    scope: ScopeConfig,
    #[serde(default)]
    mirror: Option<MirrorConfig>,
}

#[derive(Deserialize, Serialize)]
struct ScopeConfig {
    source_roots: Vec<String>,
    exclude_patterns: Vec<String>,
}
```

---

## 2. Architecture Decisions

### Workspace Layout

```
wtcd/
├── Cargo.toml          (workspace root)
├── crates/
│   ├── wtcd-core/      (lib.rs, types.rs, adapter.rs, error.rs, config.rs)
│   ├── wtcd-scope/     (lib.rs, scanner.rs, scope.rs)
│   ├── wtcd-adapters/  (lib.rs, ts.rs)
│   └── wtcd-cli/       (main.rs, commands/init.rs, commands/run.rs, output.rs)
├── tests/              (integration tests)
│   ├── fixtures/
│   │   ├── ts/
│   │   └── js/
│   └── *.rs
└── schemas/            (symlink or copy from docs)
```

### Trait Architecture

```rust
// wtcd-core/src/adapter.rs
pub trait LanguageAdapter: Send + Sync {
    fn language_name(&self) -> &str;
    fn file_extensions(&self) -> &[&str];
    fn parse(&self, source: &str, file_path: &str) -> ParseResult;
    fn confidence(&self) -> ConfidenceBand;
}

// wtcd-core/src/types.rs
pub struct ParseResult {
    pub file_path: String,
    pub confidence: ConfidenceBand,
    pub exports: Vec<ExportedSymbol>,
    pub imports: Vec<DependencyEdge>,
    pub signatures: Vec<FunctionSignature>,
    pub side_effects: Vec<SideEffect>,
    pub parse_time_ms: u64,
    pub error_message: Option<String>,
}

pub enum ConfidenceBand { High, Low, None }
```

### Pipeline Architecture

Config → Scope → Scan → Parse → Output (JSON)

Each step is a function that takes the previous step's output. This is NOT a streaming pipeline — it's batch processing per `anrsm run` invocation.

---

## 3. Extraction Strategies

### Exports (tree-sitter queries for TS/JS)

```
; Named function export
(export_statement
  declaration: (function_declaration name: (identifier) @name))

; Named class export
(export_statement
  declaration: (class_declaration name: (type_identifier) @name))

; Named const/let/var export
(export_statement
  declaration: (lexical_declaration
    (variable_declarator name: (identifier) @name)))

; Type/interface/enum export
(export_statement
  declaration: (type_alias_declaration name: (type_identifier) @name))
(export_statement
  declaration: (interface_declaration name: (type_identifier) @name))
(export_statement
  declaration: (enum_declaration name: (identifier) @name))

; Re-export (export { ... })
(export_statement
  (export_clause
    (export_specifier
      name: (identifier) @name)))
```

### Imports

```
; ESM named import
(import_statement
  source: (string) @source
  (import_clause
    (named_imports
      (import_specifier name: (identifier) @name))))

; ESM default import
(import_statement
  source: (string) @source
  (import_clause
    (identifier) @name))

; ESM namespace import
(import_statement
  source: (string) @source
  (import_clause
    (namespace_import (identifier) @name)))

; CommonJS require
(call_expression
  function: (identifier) @fn (#eq? @fn "require")
  arguments: (arguments (string) @source))
```

### Side Effects (API-based detection)

Walk the tree looking for call expressions matching known side-effect APIs:
- `fs.*` — file I/O
- `fetch`, `axios.*` — network
- `localStorage.*`, `sessionStorage.*` — storage
- `console.*` — logging
- `process.exit` — process control

### Function Signatures

For each function/method declaration or arrow function:
- Extract: name, parameters (with type annotations if present), return type
- For parameters: `(required_parameter (identifier) @name type: (type_annotation) @type)?`
- For return type: `return_type: (type_annotation) @return_type`

---

## 4. Potential Pitfalls & Mitigations

| ID | Risk | Mitigation |
|----|------|------------|
| P1 | tree-sitter-typescript version mismatch with tree-sitter core | Pin exact versions in Cargo.toml; test compile early |
| P2 | MSRV conflict (clap 4.6 = 1.85, gix 0.80 = 1.82) | Target MSRV 1.85; if user needs lower, use clap 4.5 |
| P3 | Tree traversal performance on large files | Use cursor-based traversal, not recursive; set reasonable file size limit |
| P4 | Re-export detection complexity | Start with simple `export { X } from "Y"`, defer `export * from "Y"` |
| P5 | Dynamic import() false positives | Explicitly excluded per D-17; filter `import()` calls in query |
| P6 | CommonJS require() without string literal | Only match `require("...")` pattern; dynamic require → skip |

---

## 5. Test Strategy

### Golden Test Suite

Create test fixtures under `tests/fixtures/ts/` and `tests/fixtures/js/`:

| Fixture | Purpose | Expected |
|---------|---------|----------|
| `basic_exports.ts` | Named function/class/const exports | 3 exports, confidence=high |
| `type_exports.ts` | type/interface/enum exports | 3 exports, confidence=high |
| `reexports.ts` | `export { X } from "Y"` | 2 re-exports |
| `named_imports.ts` | ESM named + default imports | 3 imports |
| `commonjs.ts` | require() calls | 2 imports |
| `side_effects.ts` | fs, fetch, console calls | 4 side_effects |
| `broken_syntax.ts` | Intentional syntax error | confidence=low, partial extraction |
| `total_garbage.ts` | Non-parseable content | confidence=none, empty facts |
| `function_signatures.ts` | Typed params + return types | 2 signatures with full types |
| `mixed_module.ts` | ESM + CJS in same file | Both import kinds detected |

### Integration Test

`tests/integration_init.rs` — test `anrsm init` creates correct `anrsm.yaml` + directory structure.

`tests/integration_run.rs` — test `anrsm run` on a small fixture repo produces valid JSON with expected structure.

---

## 6. Dependency Versions (Recommended)

```toml
[workspace.dependencies]
tree-sitter = "0.26"
tree-sitter-typescript = "0.23"
tree-sitter-javascript = "0.23"
clap = { version = "4.6", features = ["derive"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
yaml-serde = "0.10"
ignore = "0.4"
anyhow = "1"
thiserror = "2"
chrono = { version = "0.4", features = ["serde"] }
```

**MSRV:** 1.85 (clap 4.6 requirement)

---

## 7. Validation Architecture

**Parser accuracy validation:**
- Run tree-sitter queries against golden fixtures
- Assert exact counts of exports, imports, signatures, side_effects
- Assert confidence band per fixture

**CLI validation:**
- `wtcd init` in empty dir → check anrsm.yaml exists with correct structure
- `wtcd init` in dir with existing anrsm.yaml → check error/warning
- `wtcd run` on fixture repo → check JSON schema compliance
- `wtcd run` with broken files → check confidence degradation

**Scope validation:**
- Include/exclude patterns → deterministic file list
- .gitignore interaction → correct filtering
- Empty scope → error message

---

*Research completed: 2026-03-21*
*Confidence: HIGH — all stack choices validated, API patterns confirmed*
