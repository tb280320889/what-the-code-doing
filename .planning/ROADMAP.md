# Roadmap: WTCD — What The Code Doing

**Milestone:** v0.1.1 — Multi-Language & Knowledge Layer
**Created:** 2026-03-21
**Granularity:** Coarse
**Total v0.1.1 Requirements:** 40
**Phases:** 4 (Phase 5–8)
**Status:** 🚧 IN PROGRESS

---

## Milestones

- ✅ **v1.0 MVP** — Phases 1–4 + 999.1 (shipped 2026-03-21, 38/38 requirements)
- 🚧 **v0.1.1 Multi-Language & Knowledge** — Phases 5–8 (in progress)

---

## Phases

- [ ] **Phase 5: Python Adapter** — Python tree-sitter 解析适配器，提取函数/类/import/装饰器/类型注解
- [ ] **Phase 6: Go Adapter** — Go tree-sitter 解析适配器，提取函数/方法/struct/interface/import/可见性
- [ ] **Phase 7: Module Aggregation** — 模块级镜像聚合：导出汇总、依赖图、指纹、漂移检测
- [ ] **Phase 8: Knowledge Layer** — 知识层文档生成：仓库总览、模块关系图、导出索引、统计

---

<details>
<summary>✅ v1.0 MVP (Phases 1–4 + 999.1) — SHIPPED 2026-03-21</summary>

### Phase 1: Foundation ✅
**Goal**: Core types, CLI framework, TS/JS parsing, config & scope management
**Plans**: 5 plans — Complete

### Phase 2: Mirror + Fingerprint ✅
**Goal**: File mirror generation, semantic fingerprint calculation
**Plans**: 5 plans — Complete

### Phase 3: Drift + Gate ✅
**Goal**: Drift detection, CI gate, incremental updates
**Plans**: 6 plans — Complete

### Phase 4: Index + Route ✅
**Goal**: Routing index construction, Agent query interface
**Plans**: 3 plans — Complete

### Phase 999.1: MCP Server ✅
**Goal**: Expose WTCD core capabilities via MCP protocol
**Plans**: 3 plans — Complete

</details>

---

## Phase Details

### Phase 5: Python Adapter
**Goal**: Users can parse Python files and extract structured semantic facts (functions, classes, imports, decorators, type annotations) via the existing LanguageAdapter pipeline
**Depends on**: Phase 1 (existing adapter infrastructure)
**Requirements**: PY-01, PY-02, PY-03, PY-04, PY-05, PY-06, PY-07, PY-08, PY-09
**Success Criteria** (what must be TRUE):
  1. User can parse `.py` files via `wtcd run` and see extracted function definitions with parameter names and type annotations
  2. User can see class definitions with base class information in the parsed output
  3. User can see all import statements including relative imports (`from .X import Y`) in dependency edges
  4. Decorators on functions and classes appear as metadata in the output
  5. `__init__.py` files are recognized as package markers; `__all__` declarations filter exported symbols
  6. Parser gracefully degrades on Python syntax errors — output includes `confidence: low` instead of crashing
**Plans**: TBD

Plans:
- [ ] 05-01: [To be planned]
- [ ] 05-02: [To be planned]

### Phase 6: Go Adapter
**Goal**: Users can parse Go files and extract structured semantic facts (functions, methods, types, imports, visibility) via the existing LanguageAdapter pipeline
**Depends on**: Phase 1 (existing adapter infrastructure)
**Requirements**: GO-01, GO-02, GO-03, GO-04, GO-05, GO-06, GO-07, GO-08, GO-09, GO-10, GO-11, GO-12
**Success Criteria** (what must be TRUE):
  1. User can parse `.go` files via `wtcd run` and see extracted function declarations with parameter and return types
  2. User can see method declarations with receiver types and struct fields with types and tags
  3. User can see interface definitions with method signatures and embedded struct relationships
  4. Export visibility is correctly determined by uppercase/lowercase first letter of identifiers
  5. Goroutine/channel usage and compiler directives (`//go:embed`, `//go:generate`) appear as side effect metadata
  6. Parser gracefully degrades on Go syntax errors — output includes `confidence: low` instead of crashing
**Plans**: TBD

Plans:
- [ ] 06-01: [To be planned]
- [ ] 06-02: [To be planned]

### Phase 7: Module Aggregation
**Goal**: File-level mirror results are aggregated into module-level summaries with dependency graphs, semantic fingerprints, and drift rollup
**Depends on**: Phase 5, Phase 6 (multi-language FileResults needed for aggregation)
**Requirements**: MOD-01, MOD-02, MOD-03, MOD-04, MOD-05, MOD-06, MOD-07, MOD-08, MOD-09, MOD-10
**Success Criteria** (what must be TRUE):
  1. User can see module-level mirror files under `mirror/module/` containing aggregated exports, dependencies, and responsibility descriptions from all files in the module
  2. Module boundaries are auto-detected per language (Python: `__init__.py`, Go: `package`, TS: directory) and configurable via `anrsm.yaml`
  3. Each module has a semantic fingerprint calculated from sorted child fingerprints and a fan-in/fan-out statistics summary
  4. Module-level drift detection rolls up C0–C3 classifications from file-level drift to the module level
  5. Intra-module dependency graph is generated showing file-to-file relationships within each module
**Plans**: TBD

Plans:
- [ ] 07-01: [To be planned]
- [ ] 07-02: [To be planned]

### Phase 8: Knowledge Layer
**Goal**: Users can generate repository-level knowledge documents (overview, dependency graph, export index, statistics) compiled from module mirrors
**Depends on**: Phase 7 (module mirrors are the input)
**Requirements**: KNOW-01, KNOW-02, KNOW-03, KNOW-04, KNOW-05, KNOW-06, KNOW-07, KNOW-08, KNOW-09
**Success Criteria** (what must be TRUE):
  1. User can find a repository overview document under `mirror/knowledge/` listing language distribution, module count, and entry points
  2. User can see a module dependency graph in Mermaid format and a global export index across all modules
  3. User can see language/file statistics and token compression ratio (mirror_tokens / source_tokens) in the knowledge output
  4. User can see Agent read path suggestions derived from the dependency graph and routing index
  5. C2/C3 drift reports generate ADR skeleton documents for architectural decision tracking
**Plans**: TBD

Plans:
- [ ] 08-01: [To be planned]
- [ ] 08-02: [To be planned]

---

## Progress Table

| Phase | Plans Complete | Status | Completed |
|-------|----------------|--------|-----------|
| 5. Python Adapter | 0/TBD | Not started | - |
| 6. Go Adapter | 0/TBD | Not started | - |
| 7. Module Aggregation | 0/TBD | Not started | - |
| 8. Knowledge Layer | 0/TBD | Not started | - |

---

## Coverage Validation (v0.1.1)

| Requirement | Phase | Status |
|-------------|-------|--------|
| PY-01 | Phase 5 | Pending |
| PY-02 | Phase 5 | Pending |
| PY-03 | Phase 5 | Pending |
| PY-04 | Phase 5 | Pending |
| PY-05 | Phase 5 | Pending |
| PY-06 | Phase 5 | Pending |
| PY-07 | Phase 5 | Pending |
| PY-08 | Phase 5 | Pending |
| PY-09 | Phase 5 | Pending |
| GO-01 | Phase 6 | Pending |
| GO-02 | Phase 6 | Pending |
| GO-03 | Phase 6 | Pending |
| GO-04 | Phase 6 | Pending |
| GO-05 | Phase 6 | Pending |
| GO-06 | Phase 6 | Pending |
| GO-07 | Phase 6 | Pending |
| GO-08 | Phase 6 | Pending |
| GO-09 | Phase 6 | Pending |
| GO-10 | Phase 6 | Pending |
| GO-11 | Phase 6 | Pending |
| GO-12 | Phase 6 | Pending |
| MOD-01 | Phase 7 | Pending |
| MOD-02 | Phase 7 | Pending |
| MOD-03 | Phase 7 | Pending |
| MOD-04 | Phase 7 | Pending |
| MOD-05 | Phase 7 | Pending |
| MOD-06 | Phase 7 | Pending |
| MOD-07 | Phase 7 | Pending |
| MOD-08 | Phase 7 | Pending |
| MOD-09 | Phase 7 | Pending |
| MOD-10 | Phase 7 | Pending |
| KNOW-01 | Phase 8 | Pending |
| KNOW-02 | Phase 8 | Pending |
| KNOW-03 | Phase 8 | Pending |
| KNOW-04 | Phase 8 | Pending |
| KNOW-05 | Phase 8 | Pending |
| KNOW-06 | Phase 8 | Pending |
| KNOW-07 | Phase 8 | Pending |
| KNOW-08 | Phase 8 | Pending |
| KNOW-09 | Phase 8 | Pending |

**Coverage: 40/40 v0.1.1 requirements mapped ✅**

---

## Pitfall Mitigations (v0.1.1)

| Pitfall | Mitigation |
|---------|------------|
| EXT-C1: tree-sitter 版本锁冲突 | All tree-sitter crates lock same minor version; CI compile integration test |
| EXT-C2: Python 缩进解析边缘情况 | Test with real Django/Flask/FastAPI repos; ERROR nodes degrade to `confidence: low` |
| EXT-C3: Go 包语义不匹配 | Parse go.mod first; group by package declaration (not directory); filter exports by case |
| EXT-C5: 知识层生成泛化无用内容 | Only generate verifiable facts (exports, deps, structure); never generate "why" |
| C5: 镜像成为第二真相源 | Every mirror has `source_artifacts` field; gate blocks mirror-only changes |

---

## Research Flags

- **Phase 7 (Module Aggregation):** Module boundary definition needs ADR — Python `__init__.py` vs Go `package` vs TS directory unification
- **Phase 8 (Knowledge Layer):** Output depth/format needs alignment with actual Agent consumption patterns

---

*Roadmap created: 2026-03-21*
*Ready for: `/gsd-plan-phase 5`*
