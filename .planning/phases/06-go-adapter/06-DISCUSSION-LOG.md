# Phase 6: Go Adapter - Discussion Log

**Date:** 2026-03-22
**Status:** complete

## Decisions (recommended defaults)

1. 复用 `LanguageAdapter` + `FileResult` 统一接口，不新增 Go 专属输出结构。
2. Go 可见性规则采用首字母大写判定，并记录为 metadata（`go-meta:visibility:*`）。
3. receiver、struct field/tag、interface methods、embedded struct 使用 metadata 落在 `side_effects`，避免扩展核心 schema。
4. goroutine/channel/`//go:*` 指令统一映射为 side-effect metadata，保证可查询。
5. 语法错误按既有范式降级到 `confidence: low/none`，不中断流程。

## Out of scope

- Go 全量类型推断、泛型约束深度分析。
- `go list` / module graph 运行时解析（保持纯静态解析）。
