# Requirements: WTCD (What The Code Doing)

**Defined:** 2026-03-21
**Core Value:** 让 AI Agent 的默认读取行为从"吞源码"变为"先读镜像"，用最小充分上下文完成工程决策。

## v1 Requirements (Validated)

All 38 v1 requirements satisfied in Milestone v1.0. See v1.0-MILESTONE-AUDIT.md.

### Core Engine

- [x] **CORE-01**: CLI 可通过 `cargo install wtcd` 或 GitHub Releases 安装为单二进制
- [x] **CORE-02**: CLI 支持 `wtcd init` 命令，在目标仓库初始化配置和目录结构 — Phase 1
- [x] **CORE-03**: CLI 支持 `wtcd run` 命令，对目标仓库执行全量扫描和镜像生成 — Phase 1+2
- [x] **CORE-04**: CLI 支持 `wtcd check` 命令，检测当前变更集的镜像漂移 — Phase 3
- [x] **CORE-05**: CLI 支持 `wtcd route <task>` 命令，将任务路由到候选模块和文件 — Phase 4
- [x] **CORE-06**: CLI 输出 JSON 格式，可被 Agent 直接解析

### Scope / Language / Mirror / Drift / Gate / Index / MCP

- [x] All 32 remaining v1 requirements (SCOP-01~03, LANG-01~06, MIRR-01~05, FP-01~03, DRFT-01~04, GATE-01~05, RTIX-01~03, INCR-01~03)

## v0.1.1 Requirements

v0.1.1 里程碑需求：扩展多语言支持（Python, Go）并新增模块级聚合和知识层。

### Python Adapter

- [ ] **PY-01**: User can parse Python files using tree-sitter-python with error tolerance (confidence: low on parse failure)
- [ ] **PY-02**: User can extract function definitions (def name(params) -> return_type) with type annotations
- [ ] **PY-03**: User can extract class definitions (class Name(Base):) with base class info
- [ ] **PY-04**: User can extract import statements: import X, from X import Y, relative imports (from .X import Y)
- [ ] **PY-05**: User can detect decorators (@decorator) on functions and classes as metadata
- [ ] **PY-06**: User can handle __init__.py files as package markers
- [ ] **PY-07**: User can parse __all__ declarations for explicit export filtering
- [ ] **PY-08**: User can identify @property, @staticmethod, @classmethod method types
- [ ] **PY-09**: User can detect dataclass and Pydantic BaseModel patterns

### Go Adapter

- [ ] **GO-01**: User can parse Go files using tree-sitter-go with error tolerance (confidence: low on parse failure)
- [ ] **GO-02**: User can extract function declarations (func Name(params) returns)
- [ ] **GO-03**: User can extract method declarations with receivers (func (T) Method())
- [ ] **GO-04**: User can extract type declarations: struct, interface, type alias
- [ ] **GO-05**: User can extract import statements (import "pkg", import alias "path")
- [ ] **GO-06**: User can extract constant and variable declarations
- [ ] **GO-07**: User can determine export visibility (uppercase = exported, lowercase = private)
- [ ] **GO-08**: User can extract struct fields with types and tags
- [ ] **GO-09**: User can extract interface method signatures
- [ ] **GO-10**: User can identify embedded structs (type T struct { Base })
- [ ] **GO-11**: User can detect goroutine and channel patterns as side effects
- [ ] **GO-12**: User can detect compiler directives (//go:embed, //go:generate)

### Module Aggregation

- [ ] **MOD-01**: User can aggregate exports from all files in a module into module-level summary
- [ ] **MOD-02**: User can aggregate dependencies from all files in a module
- [ ] **MOD-03**: User can generate module-level responsibility description from file mirrors
- [ ] **MOD-04**: User can list all files belonging to a module
- [ ] **MOD-05**: User can aggregate side effects from all files in a module
- [ ] **MOD-06**: User can build intra-module dependency graph from import resolution
- [ ] **MOD-07**: User can auto-detect module boundaries per language (Python: __init__.py, Go: package, TS: directory)
- [ ] **MOD-08**: User can calculate module-level semantic fingerprint (hash of sorted child fingerprints)
- [ ] **MOD-09**: User can detect module-level drift via C0-C3 rollup from file-level drift
- [ ] **MOD-10**: User can calculate fan-in/fan-out statistics per module

### Knowledge Layer

- [ ] **KNOW-01**: User can generate repository overview document (language distribution, module list, entry points)
- [ ] **KNOW-02**: User can generate module dependency graph in Mermaid format
- [ ] **KNOW-03**: User can generate global export index across all modules
- [ ] **KNOW-04**: User can generate language and file statistics
- [ ] **KNOW-05**: User can perform semantic clustering via community detection algorithm
- [ ] **KNOW-06**: User can generate change hotspot map from drift history
- [ ] **KNOW-07**: User can calculate token compression ratio (mirror_tokens / source_tokens)
- [ ] **KNOW-08**: User can suggest Agent read paths based on dependency graph + routing index
- [ ] **KNOW-09**: User can generate ADR skeleton from C2/C3 drift reports

## Future Requirements

Deferred to future release. Tracked but not in current roadmap.

### Cross-Language

- **CROSS-01**: Cross-language type reference (protobuf/OpenAPI bridge)
- **CROSS-02**: Unified module discovery across mixed-language repos

### Advanced Analysis

- **ADV-01**: CGo analysis (complexity too high for v0.1.1)
- **ADV-02**: Generic constraint deep analysis
- **ADV-03**: Complete UML diagram generation

## Out of Scope

Explicitly excluded. Documented to prevent scope creep.

| Feature | Reason |
|---------|--------|
| LLM-generated free text | Violates A4 axiom — only verifiable facts, no "why" |
| Wiki-style editing | Dual truth source risk (R1) |
| Running Python/Go code | Security risk, violates "pure parsing" principle |
| Dynamic import analysis | Variables cannot be statically resolved |
| Type inference | Full type checker domain (mypy/pyright) |
| Web UI | v1 CLI-only, no web interface |
| Real-time background service | CLI on-demand execution only |

## Traceability

Which phases cover which requirements. Updated during roadmap creation.

| Requirement | Phase | Status |
|-------------|-------|--------|
| PY-01 ~ PY-09 | Phase 5 | Pending |
| GO-01 ~ GO-12 | Phase 6 | Pending |
| MOD-01 ~ MOD-10 | Phase 7 | Pending |
| KNOW-01 ~ KNOW-09 | Phase 8 | Pending |

**Coverage:**
- v0.1.1 requirements: 40 total
- Mapped to phases: 40
- Unmapped: 0 ✓

---
*Requirements defined: 2026-03-21*
*Last updated: 2026-03-21 after v0.1.1 milestone scoping*
