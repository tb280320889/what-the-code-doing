# Roadmap: WTCD — What The Code Doing

**Milestone:** v0.2.0 — Polyglot Adapters
**Created:** 2026-03-22
**Granularity:** Coarse
**Total v0.2.0 Requirements:** 24
**Phases:** 4 (Phase 9–12)
**Status:** Drafted (awaiting phase planning)

---

## Milestones

- ✅ **v1.0 MVP** — Phases 1–4 + 999.1
- ✅ **v0.1.1 Multi-Language & Knowledge** — Phases 5–8
- 🚧 **v0.2.0 Polyglot Adapters** — Phases 9–12 (current)

---

## Phases

- [ ] **Phase 9: Polyglot Adapter Coverage** - 新增 9 语言解析与适配器自动接线，打通 `run` 主链路
- [ ] **Phase 10: Aggregation Consistency Hardening** - 统一跨语言聚合/依赖/漂移语义，消除语义错配
- [ ] **Phase 11: Route & Knowledge Signal Quality** - 为消费端输出语言能力与置信度信号，提升可决策性
- [ ] **Phase 12: Stability Gates & Regression Reliability** - 建立全链路回归、性能预算、版本锁与完整门禁

---

## Phase Details

### Phase 9: Polyglot Adapter Coverage
**Goal**: 用户在不改变 CLI 使用方式的情况下，可对 9 种新增语言完成稳定解析与结构化语义提取。
**Depends on**: Phase 8
**Requirements**: ADPT-01, ADPT-02, ADPT-03, ADPT-04, ADPT-05, ADPT-06, ADPT-07, ADPT-08, ADPT-09, ADPT-10, ADPT-11
**Success Criteria** (what must be TRUE):
  1. 用户运行 `wtcd run` 时，Rust/Dart/Java/Kotlin/Swift/C++/C#/C/Zig 文件都能被解析并输出 exports/imports/signatures/side effects 与 confidence。
  2. 用户在混合语言仓库中可以看到新增扩展名被扫描器正确识别并分派到对应适配器。
  3. 用户无需学习任何新增 CLI 子命令，即可通过既有 `run/check/route` 流程使用新语言能力。
  4. 用户在新语言存在语法错误时，流程不会崩溃，结果会以低置信度可见降级呈现。
**Plans**: TBD

### Phase 10: Aggregation Consistency Hardening
**Goal**: 用户可获得跨 13 种语言（既有 + 新增）语义一致的依赖聚合与漂移判定结果。
**Depends on**: Phase 9
**Requirements**: CONS-01, CONS-02, CONS-03, CONS-04
**Success Criteria** (what must be TRUE):
  1. 用户在增量影响分析中可看到跨语言 import 被统一归一，影响扩散结果准确且可复现。
  2. 用户查看模块聚合结果时，TS/JS/Python/Go 与新增 9 语言的导出/依赖语义保持一致口径。
  3. 用户在包含条件编译或预处理的代码中执行 `wtcd check`，可得到稳定且可解释的漂移分类结果。
  4. 用户可明确识别 generated/uncertain 符号及其置信度影响，而不是被当作确定事实。
**Plans**: TBD

### Phase 11: Route & Knowledge Signal Quality
**Goal**: 用户可在 route/knowledge 输出中直接判断语言覆盖边界与置信度风险，降低误命中。
**Depends on**: Phase 10
**Requirements**: ROUT-01, ROUT-02, ROUT-03
**Success Criteria** (what must be TRUE):
  1. 用户执行 `wtcd route <task>` 时，可按语言与置信度信号缩小候选文件范围并减少明显误报。
  2. 用户在知识层文档中可查看语言能力矩阵，了解每种语言可提取项与覆盖范围。
  3. 用户在知识层中可直接定位低置信度区域与 generated/uncertain 区域，以便优先人工复核。
**Plans**: TBD

### Phase 12: Stability Gates & Regression Reliability
**Goal**: 用户可持续信任 polyglot 扩展后的质量与性能，不会因版本漂移或接线错误在合并后失稳。
**Depends on**: Phase 11
**Requirements**: ADPT-12, ROUT-04, STAB-01, STAB-02, STAB-03, STAB-04
**Success Criteria** (what must be TRUE):
  1. 用户提交变更后，CI 会对 9 种新增语言执行 `run/check/route` 矩阵验证并在异常时阻断合并。
  2. 用户可看到解析与增量更新性能预算阈值，超阈值时会被检测并报告。
  3. 用户可依赖 parser/grammar 版本锁与升级流程，升级后结果可复现且无隐式 ABI 漂移。
  4. 用户可通过自动化测试在合并前发现 scanner 扩展名与 adapter registry 不一致问题。
  5. 用户在代表性多语言仓库回归中可验证 route 质量基线保持可信（扩展后不显著退化）。
**Plans**: TBD

---

## Progress Table

| Phase | Plans Complete | Status | Completed |
|-------|----------------|--------|-----------|
| 9. Polyglot Adapter Coverage | 0/0 | Not started | - |
| 10. Aggregation Consistency Hardening | 0/0 | Not started | - |
| 11. Route & Knowledge Signal Quality | 0/0 | Not started | - |
| 12. Stability Gates & Regression Reliability | 0/0 | Not started | - |

---

## Coverage Validation (v0.2.0)

| Requirement | Phase | Status |
|-------------|-------|--------|
| ADPT-01 | Phase 9 | Pending |
| ADPT-02 | Phase 9 | Pending |
| ADPT-03 | Phase 9 | Pending |
| ADPT-04 | Phase 9 | Pending |
| ADPT-05 | Phase 9 | Pending |
| ADPT-06 | Phase 9 | Pending |
| ADPT-07 | Phase 9 | Pending |
| ADPT-08 | Phase 9 | Pending |
| ADPT-09 | Phase 9 | Pending |
| ADPT-10 | Phase 9 | Pending |
| ADPT-11 | Phase 9 | Pending |
| CONS-01 | Phase 10 | Pending |
| CONS-02 | Phase 10 | Pending |
| CONS-03 | Phase 10 | Pending |
| CONS-04 | Phase 10 | Pending |
| ROUT-01 | Phase 11 | Pending |
| ROUT-02 | Phase 11 | Pending |
| ROUT-03 | Phase 11 | Pending |
| ADPT-12 | Phase 12 | Pending |
| ROUT-04 | Phase 12 | Pending |
| STAB-01 | Phase 12 | Pending |
| STAB-02 | Phase 12 | Pending |
| STAB-03 | Phase 12 | Pending |
| STAB-04 | Phase 12 | Pending |

**Coverage: 24/24 v0.2.0 requirements mapped ✅**

---

*Roadmap created: 2026-03-22*
*Ready for: `/gsd-plan-phase 9`*
