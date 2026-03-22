---
phase: 08-knowledge-layer
status: complete
completed_at: 2026-03-22T00:00:00Z
plans:
  - 08-01
requirements:
  - KNOW-01
  - KNOW-02
  - KNOW-03
  - KNOW-04
  - KNOW-05
  - KNOW-06
  - KNOW-07
  - KNOW-08
  - KNOW-09
---

# Phase 08 Summary — Knowledge Layer

## One-liner

新增 repository knowledge layer，自动生成 overview、依赖图、导出索引、统计、聚类、热点、读路径建议和 ADR skeleton。

## Delivered

- 新增 `crates/wtcd-mirror/src/knowledge.rs`。
- 新增 `KnowledgeResult` 类型。
- `wtcd run` 自动写出 `mirror/knowledge/*` 产物。
- 支持 token 压缩率、社区聚类、漂移热点、read-path 建议、C2/C3 ADR 骨架。

## Validation Snapshot

```bash
cargo test
# 184 passed
```
