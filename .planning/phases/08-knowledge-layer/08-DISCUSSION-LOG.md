# Phase 8: Knowledge Layer - Discussion Log

**Date:** 2026-03-22
**Status:** complete

## Decisions (recommended)

1. 知识层放在 `wtcd-mirror::knowledge`，不拆 crate。
2. 输出保持“可验证事实”，不生成不可证伪的叙述。
3. 产物路径固定为 `mirror/knowledge/`。
4. 生成内容覆盖：overview、module dep mermaid、export index、stats、clusters、hotspots、read paths、ADR skeleton。
5. token 压缩率按 `mirror_tokens/source_tokens` 计算。
