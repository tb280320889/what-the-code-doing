# Technical Implementation Research Supplement

**Project:** WTCD v0.1.1 — Multi-Language & Knowledge Layer
**Date:** 2026-03-21
**Purpose:** Reference Rust ecosystem best practices for implementation

---

## Key Reference Projects

### 1. NekoCode (moe-charm/nekocode-rust)

**What:** Ultra-fast multi-language code analyzer using tree-sitter
**Languages:** JavaScript, TypeScript, Python, C++, C#, Go, Rust, C
**Performance:** 16x faster than traditional parsers, 1000+ files in seconds
**Key Insight:** Incremental analysis (23-49ms vs 267ms full analysis = 918-1956x faster)

**Architecture Pattern:**
```
nekocode-rust/
├── src/
│   ├── analyzers/          # Language-specific analyzers
│   │   ├── javascript/
│   │   ├── python/
│   │   └── go/
│   └── tree-sitter/        # Unified tree-sitter integration
```

**Applicable to WTCD:**
- ✅ Single crate, multi-module pattern (matches our `wtcd-adapters` design)
- ✅ Incremental analysis via tree-sitter's incremental parsing
- ✅ JSON output format for Agent consumption
- ⚠️ Uses separate analyzer per language (we use trait-based adapter)

### 2. Dossier (doctave/dossier)

**What:** Multi-language source code and docstrings parser
**Languages:** TypeScript, Python (alpha)
**Key Insight:** `DocsParser` trait + `Entity` output structure

**Architecture Pattern:**
```rust
trait DocsParser {
    fn parse(&self, source: &str) -> Result<Vec<Entity>>;
}

struct Entity {
    title: String,
    description: String,
    kind: EntityKind,  // class, function, type_alias, etc.
    fqn: String,       // fully qualified name
    members: Vec<Entity>,
}
```

**Applicable to WTCD:**
- ✅ Trait-based parser pattern (matches our `LanguageAdapter` design)
- ✅ Structured JSON output with FQN (fully qualified names)
- ✅ Docstring extraction as metadata
- ⚠️ Only 2 languages (we need 3 + knowledge layer)

### 3. rust-code-analysis (Mozilla)

**What:** Library for code metrics across 10 languages
**Languages:** C, C++, C#, CSS, Go, HTML, Java, JavaScript, Python, Rust, etc.
**Key Insight:** Macro-based language registration

**Architecture Pattern:**
```rust
// enums/src/languages.rs
mk_langs!(
    (Rust, "The Rust language", "rust", RustCode, RustParser, tree_sitter_rust, "rs")
    (Python, "The Python language", "python", PythonCode, PythonParser, tree_sitter_python, "py")
    // ...
);
```

**Applicable to WTCD:**
- ✅ Macro-based registration reduces boilerplate
- ✅ Supports 10 languages with consistent API
- ✅ Academic paper validates approach (SoftwareX journal)
- ⚠️ Focused on metrics, not semantic extraction

### 4. ast-grep

**What:** CLI tool for structural search/replace using tree-sitter
**Key Insight:** YAML-based rule configuration + `SupportLang` enum

**Architecture Pattern:**
```rust
// ast-grep-language crate
pub enum SupportLang {
    Python, Go, Rust, TypeScript, // ...
}

impl SupportLang {
    pub fn from_extension(ext: &str) -> Option<Self> { ... }
    pub fn get_ts_language(&self) -> TSLanguage { ... }
}
```

**Applicable to WTCD:**
- ✅ Extension-based language detection (matches our `SUPPORTED_EXTENSIONS`)
- ✅ Production-proven (585K+ downloads)
- ✅ Uses `tree-sitter-language ^0.1` bridge (same as our plan)
- ⚠> Focused on search, not full semantic extraction

### 5. tree-sitter-utils

**What:** Composable handler combinator abstraction over tree-sitter
**Key Insight:** Zero-cost handler chain pattern

**Architecture Pattern:**
```rust
// Instead of ad-hoc loop { match node.kind() {... } }
fn make_labeller<'a>() -> impl Handler<&'a MyCtx<'a>, String> {
    handler_fn(|node: Node, ctx: &MyCtx| {
        match node.kind() {
            "function_definition" => Some("Function".to_string()),
            _ => None,
        }
    }).or(handler_fn(|node, ctx| { /* class handling */ }))
    .or(never())
}
```

**Applicable to WTCD:**
- ✅ Eliminates boilerplate in tree-sitter node dispatch
- ✅ Composable, zero-cost abstractions
- ✅ Language-agnostic design (consumer defines grammar)
- ⚠️ New crate (v0.1.4, low download count) — use as reference, not dependency

### 6. oxc_semantic

**What:** Comprehensive semantic analysis for JS/TS
**Key Insight:** Symbol tables, scope trees, control flow graphs

**Applicable to WTCD:**
- ✅ Production-grade semantic analysis (1.6M+ downloads)
- ✅ Shows what's possible with Rust AST analysis
- ⚠️ JS/TS only, not multi-language

---

## Implementation Recommendations

### Phase 5: Python Adapter

**Recommended Pattern:** Dossier-style `LanguageAdapter` trait implementation

```rust
// wtcd-adapters/src/py.rs
pub struct PyAdapter;

impl LanguageAdapter for PyAdapter {
    fn language_name(&self) -> &'static str { "python" }
    fn file_extensions(&self) -> &[&'static str] { &["py", "pyi"] }
    
    fn parse(&self, source: &str, path: &Path) -> Result<FileResult> {
        let mut parser = Parser::new();
        parser.set_language(&tree_sitter_python::LANGUAGE.into())?;
        let tree = parser.parse(source, None)
            .ok_or_else(|| anyhow!("Failed to parse Python"))?;
        
        let root = tree.root_node();
        let mut exports = Vec::new();
        let mut imports = Vec::new();
        let mut side_effects = Vec::new();
        
        // Use tree-sitter-utils style handler chain
        let mut cursor = root.walk();
        for child in root.children(&mut cursor) {
            match child.kind() {
                "function_definition" => {
                    exports.push(extract_function(child, source)?);
                }
                "class_definition" => {
                    exports.push(extract_class(child, source)?);
                }
                "import_statement" | "import_from_statement" => {
                    imports.push(extract_import(child, source)?);
                }
                _ => {}
            }
        }
        
        Ok(FileResult {
            file_path: path.to_path_buf(),
            exports,
            imports,
            side_effects,
            confidence: if tree.root_node().has_error() {
                ConfidenceBand::Low
            } else {
                ConfidenceBand::High
            },
            ..Default::default()
        })
    }
}
```

**Key Implementation Details:**
1. `decorated_definition` handling — check child for `function_definition` or `class_definition`
2. `__all__` parsing — find `assignment` with left side `identifier` = "__all__", extract list items
3. Relative imports — `import_from_statement` with `relative_import` child
4. Decorator detection — `decorator` child nodes of function/class

### Phase 6: Go Adapter

**Recommended Pattern:** Same trait, Go-specific visibility logic

```rust
// wtcd-adapters/src/go.rs
pub struct GoAdapter;

impl GoAdapter {
    fn is_exported(name: &str) -> bool {
        name.chars().next().map_or(false, |c| c.is_uppercase())
    }
}

impl LanguageAdapter for GoAdapter {
    fn language_name(&self) -> &'static str { "go" }
    fn file_extensions(&self) -> &[&'static str] { &["go"] }
    
    fn parse(&self, source: &str, path: &Path) -> Result<FileResult> {
        // Similar pattern to PyAdapter
        // Key difference: visibility = uppercase first letter
        // method_declaration: extract receiver from `parameter_list`
        // type_declaration: handle struct, interface, type_alias
    }
}
```

### Phase 7: Module Aggregation

**Recommended Pattern:** NekoCode-style incremental + Mozilla-style metrics

```rust
// New type in wtcd-core/src/types.rs
pub struct ModuleResult {
    pub module_id: String,
    pub source_paths: Vec<PathBuf>,
    pub exports: Vec<ExportedSymbol>,
    pub dependencies: Vec<DependencyEdge>,
    pub side_effects: Vec<SideEffect>,
    pub fingerprint: String,  // hash(sorted(child_fingerprints))
    pub fan_in: usize,
    pub fan_out: usize,
    pub intra_module_graph: IntraModuleGraph,
}

// Aggregation logic
pub fn aggregate_module(files: &[FileResult]) -> ModuleResult {
    let mut exports = Vec::new();
    let mut deps = HashSet::new();
    let mut side_effects = Vec::new();
    
    for file in files {
        exports.extend(file.exports.iter().cloned());
        deps.extend(file.imports.iter().map(|i| i.source.clone()));
        side_effects.extend(file.side_effects.iter().cloned());
    }
    
    ModuleResult {
        module_id: derive_module_id(files),
        source_paths: files.iter().map(|f| f.file_path.clone()).collect(),
        exports,
        dependencies: deps.into_iter().collect(),
        side_effects,
        fingerprint: calculate_module_fingerprint(files),
        ..Default::default()
    }
}
```

### Phase 8: Knowledge Layer

**Recommended Pattern:** Template-based generation (no LLM)

```rust
// wtcd-mirror/src/knowledge.rs
pub struct KnowledgeGenerator;

impl KnowledgeGenerator {
    pub fn generate_overview(modules: &[ModuleResult]) -> String {
        // Template: language distribution, module count, entry points
        // Use pulldown-cmark for Markdown rendering
        // Use yaml_serde for Front Matter
    }
    
    pub fn generate_dependency_graph(modules: &[ModuleResult]) -> String {
        // Generate Mermaid graph from module dependencies
        // graph TD\n  auth --> core\n  api --> auth\n...
    }
    
    pub fn generate_export_index(modules: &[ModuleResult]) -> String {
        // Global export index: module | symbol | kind | file
    }
}
```

---

## Dependency Matrix

| Crate | Version | Purpose | Phase |
|-------|---------|---------|-------|
| tree-sitter | 0.26.7 | Core parser | 5, 6 |
| tree-sitter-python | 0.25.0 | Python grammar | 5 |
| tree-sitter-go | 0.25.0 | Go grammar | 6 |
| tree-sitter-language | 0.1.x | Version bridge | 5, 6 |
| pulldown-cmark | 0.13 | Markdown parsing | 8 |
| yaml-serde | 0.10 | YAML Front Matter | 8 |
| serde_json | 1.0 | JSON serialization | All |

**No new dependencies needed for Phase 7 (Module Aggregation)** — pure business logic.

---

## Risk Mitigations

| Risk | Mitigation | Reference |
|------|-----------|-----------|
| tree-sitter version lock | Use `tree-sitter-language ^0.1` bridge | ast-grep, NekoCode |
| Python indentation edge cases | Test with Django/Flask/FastAPI repos | NekoCode |
| Go package semantics | Parse package declaration, not directory | Dossier |
| Knowledge layer useless content | Only verifiable facts, no "why" | Mozilla metrics |
| Performance regression | Incremental analysis (tree-sitter native) | NekoCode |

---

*Research supplemented: 2026-03-21*
