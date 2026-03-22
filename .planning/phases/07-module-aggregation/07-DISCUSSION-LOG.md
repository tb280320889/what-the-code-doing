# Phase 7: Module Aggregation - Discussion Log

**Date:** 2026-03-22
**Status:** complete

## Decisions (recommended)

1. Phase 7 直接在 `wtcd-mirror` 内实现，不拆新 crate。
2. 模块边界自动识别：Python `__init__.py`，Go `package`，TS/JS 按目录。
3. 模块语义指纹 = 子文件语义信息排序后哈希。
4. 模块漂移等级采用文件级 C0-C3 rollup（max severity）。
5. 输出产物路径：`mirror/module/*.md`。
