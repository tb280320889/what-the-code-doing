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

## v0.1.1 Requirements (Validated)

All 40 v0.1.1 requirements satisfied in Milestone v0.1.1. See `.planning/v0.1.1-MILESTONE-AUDIT.md`.

## v0.2.0 Requirements

v0.2.0 里程碑需求：新增 Rust、Dart、Java、Kotlin、Swift、C++、C#、C、Zig 九种语言完整适配，并确保聚合/路由/门禁链路一致可靠。

### Adapter Coverage

- [ ] **ADPT-01**: User can parse Rust files via `wtcd run` and extract exports/imports/signatures/side effects with confidence grading
- [ ] **ADPT-02**: User can parse Dart files via `wtcd run` and extract exports/imports/signatures/side effects with confidence grading
- [ ] **ADPT-03**: User can parse Java files via `wtcd run` and extract exports/imports/signatures/side effects with confidence grading
- [ ] **ADPT-04**: User can parse Kotlin files via `wtcd run` and extract exports/imports/signatures/side effects with confidence grading
- [ ] **ADPT-05**: User can parse Swift files via `wtcd run` and extract exports/imports/signatures/side effects with confidence grading
- [ ] **ADPT-06**: User can parse C++ files via `wtcd run` and extract exports/imports/signatures/side effects with confidence grading
- [ ] **ADPT-07**: User can parse C# files via `wtcd run` and extract exports/imports/signatures/side effects with confidence grading
- [ ] **ADPT-08**: User can parse C files via `wtcd run` and extract exports/imports/signatures/side effects with confidence grading
- [ ] **ADPT-09**: User can parse Zig files via `wtcd run` and extract exports/imports/signatures/side effects with confidence grading
- [ ] **ADPT-10**: User can see scanner coverage for all new language extensions so files are routed to the correct adapters
- [ ] **ADPT-11**: User can rely on adapter registry auto-selection so no new CLI command is needed for new languages
- [ ] **ADPT-12**: User can trust each new adapter via fixtures + unit tests + integration tests for normal/broken syntax and representative language constructs

### Aggregation Consistency

- [ ] **CONS-01**: User can get language-aware import normalization in depgraph so incremental impact expansion is accurate across all supported languages
- [ ] **CONS-02**: User can see module aggregation results stay semantically consistent across TS/JS/Python/Go and the 9 new languages
- [ ] **CONS-03**: User can get stable drift classification by using explicit analysis context for conditional compilation/preprocessor-sensitive languages
- [ ] **CONS-04**: User can see generated or uncertain symbols explicitly marked (with confidence impact) instead of silently treated as certain facts

### Consumer Quality (Route/Knowledge)

- [ ] **ROUT-01**: User can query routes with language and confidence signals to reduce false-positive candidate files
- [ ] **ROUT-02**: User can inspect a language capability matrix in knowledge output to understand per-language extraction coverage
- [ ] **ROUT-03**: User can see low-confidence and generated/uncertain areas explicitly surfaced in knowledge docs
- [ ] **ROUT-04**: User can trust route quality after adapter expansion via regression checks on representative multi-language repositories

### Stability Gates

- [ ] **STAB-01**: User can rely on CI matrix checks that validate `run/check/route` across all 9 new languages
- [ ] **STAB-02**: User can rely on performance budgets for parsing and incremental updates with measurable thresholds
- [ ] **STAB-03**: User can rely on parser/grammar version locks plus upgrade playbook to avoid uncontrolled ABI drift
- [ ] **STAB-04**: User can trust pipeline integrity because tests catch scanner-extension and adapter-registry mismatch before merge

## Future Requirements

Deferred to future release. Tracked but not in current roadmap.

### Cross-Language

- **CROSS-01**: Cross-language type reference (protobuf/OpenAPI bridge)
- **CROSS-02**: Unified module discovery across mixed-language repos

### Advanced Analysis

- **ADV-01**: CGo analysis (complexity too high for v0.1.1)
- **ADV-02**: Generic constraint deep analysis
- **ADV-03**: Complete UML diagram generation

### Polyglot Advanced

- **POLY-01**: Cross-language deep type resolution (compiler/LSP assisted) for high-confidence symbol binding
- **POLY-02**: Full macro/template/preprocessor expansion modeling for Rust/C/C++ level semantic depth
- **POLY-03**: High-precision FFI and interop edge modeling (JNI/PInvoke/extern/cgo-style patterns)

## Out of Scope

Explicitly excluded. Documented to prevent scope creep.

| Feature | Reason |
|---------|--------|
| LLM-generated free text | Violates A4 axiom — only verifiable facts, no "why" |
| Wiki-style editing | Dual truth source risk (R1) |
| Running Python/Go code | Security risk, violates "pure parsing" principle |
| Dynamic import analysis | Variables cannot be statically resolved |
| Type inference | Full type checker domain (mypy/pyright) |
| Compiler-level semantic binding in v0.2.0 | Too expensive for this milestone; defer to POLY-01 |
| Full macro/template/preprocessor expansion in v0.2.0 | High complexity and toolchain coupling; defer to POLY-02 |
| Web UI | v1 CLI-only, no web interface |
| Real-time background service | CLI on-demand execution only |

## Traceability

Which phases cover which requirements. Updated during roadmap creation.

| Requirement | Phase | Status |
|-------------|-------|--------|
| ADPT-01 | TBD | Pending |
| ADPT-02 | TBD | Pending |
| ADPT-03 | TBD | Pending |
| ADPT-04 | TBD | Pending |
| ADPT-05 | TBD | Pending |
| ADPT-06 | TBD | Pending |
| ADPT-07 | TBD | Pending |
| ADPT-08 | TBD | Pending |
| ADPT-09 | TBD | Pending |
| ADPT-10 | TBD | Pending |
| ADPT-11 | TBD | Pending |
| ADPT-12 | TBD | Pending |
| CONS-01 | TBD | Pending |
| CONS-02 | TBD | Pending |
| CONS-03 | TBD | Pending |
| CONS-04 | TBD | Pending |
| ROUT-01 | TBD | Pending |
| ROUT-02 | TBD | Pending |
| ROUT-03 | TBD | Pending |
| ROUT-04 | TBD | Pending |
| STAB-01 | TBD | Pending |
| STAB-02 | TBD | Pending |
| STAB-03 | TBD | Pending |
| STAB-04 | TBD | Pending |

**Coverage:**
- v0.2.0 requirements: 24 total
- Mapped to phases: 0
- Unmapped: 24 ⚠️

---
*Requirements defined: 2026-03-21*
*Last updated: 2026-03-22 after v0.2.0 requirement definition draft*
