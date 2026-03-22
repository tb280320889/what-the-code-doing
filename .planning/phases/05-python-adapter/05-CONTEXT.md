# Phase 5: Python Adapter - Context

**Gathered:** 2026-03-21
**Status:** Ready for planning

<domain>
## Phase Boundary

用户可以通过现有 LanguageAdapter 流水线解析 Python 文件，提取结构化语义事实（函数、类、import、装饰器、类型注解）。适配器遵循 TsAdapter 的三方法 trait 模式，使用 tree-sitter-python 进行容错解析。

</domain>

<decisions>
## Implementation Decisions

### Adapter Structure
- Follow TsAdapter pattern: `PyAdapter` struct wrapping `Mutex<Parser>` with `tree-sitter-python`
- Single parser — Python has no separate grammar variants (unlike TS/JS split)
- Same 3-method `LanguageAdapter` trait: `language_name()`, `file_extensions()`, `parse()`
- Register in `register_all_adapters()` alongside TsAdapter

### Extraction Mapping
- **Type annotations** map to `signatures` fields (param_types, return_type) — same convention as TS adapter
- **Decorators** represented as `decorators: Vec<String>` metadata on function/class exports (not side_effects)
- **Class inheritance** stored as `bases: Vec<String>` field in export metadata — flat name list

### Python Package Semantics
- **`__init__.py`** recognized as package marker; metadata stored but module boundary logic deferred to Phase 7
- **`__all__`** declarations parsed and stored in export metadata for Phase 7 module aggregation
- **Relative imports** (`from .X import Y`) normalized to structured import edges with `is_relative: bool` flag

### Error Recovery & Testing
- Confidence bands: reuse existing High/Low/None — no new levels
- Test fixtures in `tests/fixtures/python/` — follows existing `tests/fixtures/ts/` pattern
- Graceful degradation on syntax errors: `confidence: low` with partial extraction

</decisions>

<code_context>
## Existing Code Insights

### Reusable Assets
- `LanguageAdapter` trait in `crates/wtcd-core/src/adapter.rs` — 3-method interface (language_name, file_extensions, parse)
- `AdapterRegistry` in same file — register() + find_adapter() by extension
- `FileResult` in `crates/wtcd-core/src/types.rs` — exports, imports, signatures, side_effects, confidence, parse_time_ms
- `ConfidenceBand` enum — High/Low/None
- `Export`, `Import`, `Signature`, `SideEffect` types — all defined in types.rs

### Established Patterns
- TsAdapter pattern: `Mutex<Parser>` wrapping, extension-based parser selection, extract_* helper functions
- tree-sitter 0.26 as workspace dependency — need tree-sitter-python compatible version
- `register_all_adapters()` in `crates/wtcd-adapters/src/lib.rs` — registration entry point
- Test fixtures directory structure: `tests/fixtures/{lang}/`

### Integration Points
- `crates/wtcd-adapters/Cargo.toml` — add tree-sitter-python workspace dependency
- `Cargo.toml` (workspace root) — add tree-sitter-python to workspace dependencies
- `crates/wtcd-adapters/src/lib.rs` — add `mod py;` and register PyAdapter
- `crates/wtcd-adapters/src/` — new `py.rs` file (mirrors `ts.rs` layout)

</code_context>

<specifics>
## Specific Ideas

- tree-sitter-python needs version compatible with tree-sitter 0.26 — check crates.io
- Python has no JSX/TSX variant, so single parser approach is simpler than TS adapter
- Python-specific node types: function_definition, class_definition, import_statement, import_from_statement, decorated_definition, parameters, type, decorator
- `__all__` parsed from assignment statements where left side is `__all__`
- dataclass/@dataclass and Pydantic BaseModel patterns detected via decorator/base class matching

</specifics>

<deferred>
## Deferred Ideas

- Module boundary detection (Python `__init__.py` package hierarchy) — belongs to Phase 7
- Cross-language type reference — future requirement
- CGo analysis — out of scope
- Dynamic import analysis — cannot be statically resolved

</deferred>
