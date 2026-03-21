---
phase: 01-foundation
plan: 05
subsystem: testing
tags: [integration-tests, golden-fixtures, tree-sitter, serde, cargo-workspace]

requires:
  - phase: 01-foundation
    provides: All prior plans (01-04) — workspace, core types, adapters, CLI
provides:
  - 8 golden test fixtures for TS/JS parsing scenarios
  - 10 integration tests verifying full pipeline
  - Dedicated test crate for workspace-level test infrastructure
affects: "所有后续 phase — 回归保护覆盖解析、提取、序列化"

tech-stack:
  added: [wtcd-tests crate]
  patterns: [golden fixtures, integration tests at workspace level, fixture_path helper]

key-files:
  created:
    - crates/wtcd-tests/Cargo.toml
    - crates/wtcd-tests/src/lib.rs
    - crates/wtcd-tests/tests/integration_run.rs
    - crates/wtcd-tests/tests/integration_init.rs
    - tests/fixtures/ts/basic_exports.ts
    - tests/fixtures/ts/type_exports.ts
    - tests/fixtures/ts/named_imports.ts
    - tests/fixtures/ts/commonjs.ts
    - tests/fixtures/ts/side_effects.ts
    - tests/fixtures/ts/broken_syntax.ts
    - tests/fixtures/ts/function_signatures.ts
    - tests/fixtures/js/basic_exports.js
  modified:
    - Cargo.toml (added wtcd-tests to workspace members)

key-decisions:
  - "Created dedicated wtcd-tests crate instead of workspace-root tests — Cargo virtual manifest cannot have dev-dependencies"
  - "Used CARGO_MANIFEST_DIR + fixture_path() helper for cross-crate fixture resolution"

patterns-established:
  - "Golden fixture pattern: deterministic TS/JS files in tests/fixtures/ for regression testing"
  - "Test crate pattern: dedicated wtcd-tests crate for integration tests that span multiple workspace crates"
  - "fixture_path() helper: resolves fixture files relative to workspace root using CARGO_MANIFEST_DIR"

requirements-completed: [CORE-01, CORE-06, LANG-01, LANG-02, LANG-03, LANG-04, LANG-05, LANG-06]

duration: 6min
completed: 2026-03-21
---

# Phase 1 Plan 5: Golden Test Fixtures + Integration Tests Summary

**8 golden fixtures + 10 integration tests covering exports, imports, CJS, side effects, broken syntax, function signatures, JS parsing, and JSON serialization**

## Performance

- **Duration:** 6 min
- **Started:** 2026-03-21T07:25:00Z
- **Completed:** 2026-03-21T07:31:02Z
- **Tasks:** 2
- **Files modified:** 12 created, 1 modified

## Accomplishments

- 8 golden test fixtures covering all TS/JS parsing scenarios (exports, types, imports, CJS, side effects, broken syntax, signatures, JS)
- 9 integration tests verifying full adapter pipeline (parse → extract → confidence → JSON output)
- 1 config template validation test
- Dedicated `wtcd-tests` crate for workspace-level integration testing
- Full test suite: 54 tests passing (44 prior + 10 new)

## Task Commits

Each task was committed atomically:

1. **Task 1: Create golden test fixtures** - `9d5c3c7` (docs — fixture files committed in prior pack)
2. **Task 2: Write integration tests** - `c8a76d8` (test)

**Plan metadata:** (will be committed with SUMMARY/STATE/ROADMAP)

## Files Created/Modified

- `tests/fixtures/ts/basic_exports.ts` — 3 exports: function, class, const
- `tests/fixtures/ts/type_exports.ts` — 3 exports: type, interface, enum
- `tests/fixtures/ts/named_imports.ts` — 3 import styles: named, type-only, namespace
- `tests/fixtures/ts/commonjs.ts` — 2 require() calls: fs, path
- `tests/fixtures/ts/side_effects.ts` — 4 side effects: fs, fetch, console, localStorage
- `tests/fixtures/ts/broken_syntax.ts` — Deliberately broken TS for error recovery testing
- `tests/fixtures/ts/function_signatures.ts` — 2 functions with typed params and return types
- `tests/fixtures/js/basic_exports.js` — 3 JS exports (no type annotations)
- `crates/wtcd-tests/Cargo.toml` — Test crate manifest with dev-dependencies
- `crates/wtcd-tests/src/lib.rs` — Placeholder lib for test crate
- `crates/wtcd-tests/tests/integration_run.rs` — 9 integration tests for parsing pipeline
- `crates/wtcd-tests/tests/integration_init.rs` — 1 config template validation test
- `Cargo.toml` — Added wtcd-tests to workspace members

## Decisions Made

- **Created dedicated wtcd-tests crate:** Cargo virtual manifest (workspace root without `[package]`) cannot have `[dev-dependencies]`. Standard workaround is a dedicated test crate.
- **Used fixture_path() helper:** Tests need absolute paths since working directory varies. `env!("CARGO_MANIFEST_DIR")` + relative `../../tests/fixtures/` provides reliable resolution.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Moved integration tests to dedicated test crate**
- **Found during:** Test compilation
- **Issue:** Cargo virtual manifest (workspace root `Cargo.toml` without `[package]`) does not allow `[dev-dependencies]` section — integration tests at `tests/` root fail to compile
- **Fix:** Created `crates/wtcd-tests/` crate with proper `[dev-dependencies]`, moved test files there, added to workspace members
- **Files modified:** Cargo.toml, crates/wtcd-tests/ (new crate)
- **Verification:** `cargo test -p wtcd-tests` passes
- **Committed in:** c8a76d8 (Task 2 commit)

**2. [Rule 3 - Blocking] Fixed fixture path resolution for cross-crate tests**
- **Found during:** Test execution
- **Issue:** Relative paths like `tests/fixtures/ts/basic_exports.ts` don't resolve from test crate working directory
- **Fix:** Added `fixture_path()` helper using `env!("CARGO_MANIFEST_DIR")` to build absolute paths
- **Files modified:** crates/wtcd-tests/tests/integration_run.rs
- **Verification:** All 10 integration tests pass
- **Committed in:** c8a76d8 (Task 2 commit)

---

**Total deviations:** 2 auto-fixed (2 blocking)
**Impact on plan:** Both were infrastructure fixes for Cargo workspace constraint. No scope creep — test coverage matches plan exactly.

## Issues Encountered

None — existing adapter code passed all integration tests on first run.

## User Setup Required

None — no external service configuration required.

## Next Phase Readiness

- Phase 1 (Foundation) complete — all 5 plans done
- Full regression protection in place: 54 tests covering workspace, scope, adapters, CLI
- Ready for Phase 2 (semantic mirror generation)

---
*Phase: 01-foundation*
*Completed: 2026-03-21*
