# Roadmap: ANRSM — AI Native Repo Semantic Mirror

**Created:** 2026-03-21
**Granularity:** Coarse
**Total v1 Requirements:** 38
**Phases:** 4

---

## Phases

- [ ] **Phase 1: Foundation** — 核心类型、CLI 框架、TS/JS 解析、配置与范围管理
- [ ] **Phase 2: Mirror + Fingerprint** — 文件镜像生成、语义指纹计算
- [ ] **Phase 3: Drift + Gate** — 漂移检测、CI 门禁、增量更新
- [ ] **Phase 4: Index + Route** — 路由索引构建、Agent 查询接口

---

## Phase Details

### Phase 1: Foundation
**Goal**: Users can install ANRSM, initialize a repo, configure scope, and parse TS/JS files to extract structured facts
**Depends on**: Nothing
**Requirements**: CORE-01, CORE-02, CORE-03, CORE-06, SCOP-01, SCOP-02, SCOP-03, LANG-01, LANG-02, LANG-03, LANG-04, LANG-05, LANG-06
**Success Criteria** (what must be TRUE):
  1. User can install anrsm via `cargo install` or download a pre-built binary from GitHub Releases
  2. User can run `anrsm init` to scaffold an `anrsm.yaml` config and directory structure in any repo
  3. User can configure include/exclude glob rules in `anrsm.yaml` and see the managed file list change deterministically
  4. User can run `anrsm run` to parse all scoped TS/JS files and see exported symbols, function signatures, dependency edges, and detected side effects
  5. Parser gracefully degrades on broken files — output includes `confidence: low` instead of crashing
  6. All CLI output is JSON-structured and machine-parseable
**Plans**: 5 plans

Plans:
- [x] 01-01-PLAN.md — Cargo workspace + wtcd-core 核心类型定义
- [x] 01-02-PLAN.md — wtcd-scope 配置解析 + 文件扫描 ✅
- [x] 01-03-PLAN.md — wtcd-adapters TS/JS tree-sitter 解析适配器
- [x] 01-04-PLAN.md — wtcd-cli init/run 命令 + JSON 输出
- [x] 01-05-PLAN.md — Golden test fixtures + 集成测试

### Phase 2: Mirror + Fingerprint
**Goal**: Parsed semantic facts are rendered as version-controllable mirror files with dual fingerprints distinguishing content changes from semantic changes
**Depends on**: Phase 1
**Requirements**: MIRR-01, MIRR-02, MIRR-03, MIRR-04, MIRR-05, FP-01, FP-02, FP-03
**Success Criteria** (what must be TRUE):
  1. User runs `anrsm run` and finds Markdown mirror files under `mirror/file/` with YAML Front Matter headers matching the `mirror-header.schema.json`
  2. Each mirror body contains the 8 standard sections: responsibilities, external contracts, preconditions, state/control flow, side effects, invariants/risks, change impact, expand conditions
  3. Mirror path follows deterministic mapping: `mirror/file/<source_relative_path>.md`
  4. `source_fingerprint` changes on any text edit; `semantic_fingerprint` stays stable across whitespace/comment/formatting changes
  5. Fingerprint algorithm includes `fp_version` field for future algorithm upgrades
**Plans**: TBD

### Phase 3: Drift + Gate
**Goal**: Users and CI can detect when source changes require mirror updates, with configurable gate policies that block or warn based on drift severity
**Depends on**: Phase 2
**Requirements**: FP-01, DRFT-01, DRFT-02, DRFT-03, DRFT-04, GATE-01, GATE-02, GATE-03, GATE-04, GATE-05, INCR-01, INCR-02, INCR-03
**Success Criteria** (what must be TRUE):
  1. User runs `anrsm check` and gets a drift report classifying changed files as C0 (non-material), C1 (local material), C2 (contract material), or C3 (systemic material)
  2. Drift analysis expands to affected neighbors — changing a file reports not just itself but files that import/depend on it
  3. `anrsm check` outputs a structured drift report conforming to `drift-report.schema.json`
  4. Gate policy is configurable in `anrsm.yaml` — initially defaults to warn-only mode to avoid adoption-killing false positives
  5. Gate returns standard exit codes (0 pass, 1 fail) with ANRSM-001 to ANRSM-010 failure codes for CI pipeline integration
  6. `anrsm run` defaults to incremental mode — only re-parses changed files and their neighbors via git diff; `--full` flag triggers full rebuild
**Plans**: TBD

### Phase 4: Index + Route
**Goal**: Agents and developers can query the mirror system with natural language tasks and get ranked candidate modules/files with freshness signals
**Depends on**: Phase 3
**Requirements**: CORE-04, CORE-05, RTIX-01, RTIX-02, RTIX-03
**Success Criteria** (what must be TRUE):
  1. A routing index is generated containing artifact IDs, module IDs, semantic keywords, and exported symbols for all mirrored files
  2. User runs `anrsm route "modify auth logic"` and gets ranked candidate files with relevance scores and freshness state
  3. Route results support `--top-k` flag to limit returned candidates
  4. `anrsm check` command works end-to-end for drift detection against the current git working tree
  5. `anrsm route` command works end-to-end, reading from the generated routing index
**Plans**: TBD

---

## Progress Table

| Phase | Plans Complete | Status | Completed |
|-------|----------------|--------|-----------|
| 1. Foundation | 4/5 | In Progress | - |
| 2. Mirror + Fingerprint | 0/8 | Not started | - |
| 3. Drift + Gate | 0/12 | Not started | - |
| 4. Index + Route | 0/5 | Not started | - |

---

## Coverage Validation

| Requirement | Phase | Status |
|-------------|-------|--------|
| CORE-01 | Phase 1 | Pending |
| CORE-02 | Phase 1 | ✅ Complete (01-04) |
| CORE-03 | Phase 1 | ✅ Complete (01-04) |
| CORE-04 | Phase 4 | Pending |
| CORE-05 | Phase 4 | Pending |
| CORE-06 | Phase 1 | ✅ Complete (01-04) |
| SCOP-01 | Phase 1 | ✅ Complete (01-02) |
| SCOP-02 | Phase 1 | ✅ Complete (01-02) |
| SCOP-03 | Phase 1 | ✅ Complete (01-02) |
| LANG-01 | Phase 1 | Pending |
| LANG-02 | Phase 1 | Pending |
| LANG-03 | Phase 1 | Pending |
| LANG-04 | Phase 1 | Pending |
| LANG-05 | Phase 1 | Pending |
| LANG-06 | Phase 1 | Pending |
| MIRR-01 | Phase 2 | Pending |
| MIRR-02 | Phase 2 | Pending |
| MIRR-03 | Phase 2 | Pending |
| MIRR-04 | Phase 2 | Pending |
| MIRR-05 | Phase 2 | Pending |
| FP-01 | Phase 2 | Pending |
| FP-02 | Phase 2 | Pending |
| FP-03 | Phase 2 | Pending |
| DRFT-01 | Phase 3 | Pending |
| DRFT-02 | Phase 3 | Pending |
| DRFT-03 | Phase 3 | Pending |
| DRFT-04 | Phase 3 | Pending |
| GATE-01 | Phase 3 | Pending |
| GATE-02 | Phase 3 | Pending |
| GATE-03 | Phase 3 | Pending |
| GATE-04 | Phase 3 | Pending |
| GATE-05 | Phase 3 | Pending |
| RTIX-01 | Phase 4 | Pending |
| RTIX-02 | Phase 4 | Pending |
| RTIX-03 | Phase 4 | Pending |
| INCR-01 | Phase 3 | Pending |
| INCR-02 | Phase 3 | Pending |
| INCR-03 | Phase 3 | Pending |

**Coverage: 38/38 v1 requirements mapped ✓**

---

## Pitfall Mitigations Embedded

| Pitfall | Mitigation |
|---------|------------|
| C1: 镜像沦为装饰 | Phase 4 builds routing index + Agent read interface as validation point |
| C2: 门禁误报 | Phase 3 defaults to warn-only mode; FP-02 in Phase 2 ensures fingerprint normalization |
| C3: AST 解析地雷 | Phase 1 establishes golden test suite for tree-sitter; LANG-06 mandates graceful degradation |
| C4: 指纹跨版本不稳定 | Phase 2 FP-03 mandates fp_version field from the start |
| C5: 镜像变第二真相源 | Phase 3 gate detects mirror-only changes; MIRR-02 includes source_artifacts |
| C6: 全量重建杀死性能 | Phase 3 INCR-01 makes incremental the default path |

---

## Research Flags

- **Phase 2:** 语义指纹算法的具体设计 — C1 vs C2 阈值、归一化规则需要 PoC 验证
- **Phase 3:** C0/C1/C2 分类边界的实际验证 — 需要真实 diff 数据集
- **Phase 4:** Agent 行为变更验证 — 需要实际使用数据

---

---

## Backlog

### Phase 999.1: 还是要做成MCP, MCP 比 cli 更有优势, 会自动加载到 agent的上下文中,看看如何开发 (BACKLOG)

**Goal:** [Captured for future planning]
**Requirements:** TBD
**Plans:** 0 plans

Plans:
- [ ] TBD (promote with /gsd-review-backlog when ready)

---

*Roadmap created: 2026-03-21*
*Ready for: `/gsd-plan-phase 1`
