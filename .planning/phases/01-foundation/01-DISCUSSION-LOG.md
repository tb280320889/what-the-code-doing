# Phase 1: Foundation - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-03-21
**Phase:** 01-foundation
**Areas discussed:** JSON 输出结构, anrsm.yaml 配置格式, 错误恢复策略, tree-sitter 提取范围, 项目目录架构

---

## JSON 输出结构

| Option | Description | Selected |
|--------|-------------|----------|
| 单层扁平 | 顶层直接包含 files[], errors[], summary | ✓ |
| 嵌套分组 | data / meta / errors 三块嵌套 | |
| JSONL 流式 | 每行一个 JSON 对象 | |

| Option | Description | Selected |
|--------|-------------|----------|
| 事实层 + 元数据 | exports/imports/symbols/signatures/side_effects + confidence/file_path/parse_time | ✓ |
| 纯事实层 | 只包含事实，不含 confidence | |
| 事实层 + 元数据 + AST 摘要 | 额外包含 tree-sitter 节点统计 | |

| Option | Description | Selected |
|--------|-------------|----------|
| 同级 files 数组 + error 标记 | 失败文件也在 files[], confidence=low/none + error_message | ✓ |
| 单独 errors 数组 | 成功在 files[], 失败在 errors[] | |
| 混合 + 顶层 errors 数组 | 严重错误放 errors[], 解析降级放 files[] | |

| Option | Description | Selected |
|--------|-------------|----------|
| 带 version 字段 | 顶层 api_version: "1" | ✓ |
| 不带版本 | 通过 JSON Schema 文件约定 | |
| 仅 schema URL | 顶层 $schema 指向 Schema | |

**依赖边格式:**
| Option | Description | Selected |
|--------|-------------|----------|
| 结构化对象数组 | {source, symbols, kind} | ✓ |
| 原始字符串数组 | 只保留 import 语句原始文本 | |
| 对象数组 + 路径解析 | 额外解析模块路径到绝对路径 | |

**副作用表示:**
| Option | Description | Selected |
|--------|-------------|----------|
| 分类对象数组 | {kind, target, line} 按类别分类 | ✓ |
| 统一字符串数组 | ["fs.readFile", "console.log"] | |
| 按类别分组 | {io: [...], network: [...]} | |

**summary 内容:**
| Option | Description | Selected |
|--------|-------------|----------|
| 运行级统计 | total_files, parsed_ok, confidence_low/none, total_exports/imports, elapsed_ms | ✓ |
| 仅文件计数 | total_files, parsed_ok, failed | |
| 详细统计 | per-extension, per-confidence-band, parser 版本 | |

**User's choices:** All recommended options selected.
**Notes:** 结构清晰，Agent 可直接消费 JSON 输出。

---

## anrsm.yaml 配置格式

| Option | Description | Selected |
|--------|-------------|----------|
| scope + mirror + output | 三块分离，Phase 1 只需 scope | ✓ |
| scope + output | 只分两块 | |
| 扁平单层 | 所有配置项平铺 | |

| Option | Description | Selected |
|--------|-------------|----------|
| 标准 glob | gitignore 风格 glob，与 ignore crate 兼容 | ✓ |
| 自定义 DSL | ANRSM 特有范围语法 | |
| JSON 数组 + glob | include: [...] exclude: [...] | |

| Option | Description | Selected |
|--------|-------------|----------|
| 拒绝运行 | 要求显式配置 | ✓ |
| 扫描当前目录所有 TS/JS | 默认开箱即用 | |
| 扫描 + 自动排除 node_modules | 智能排除 | |

| Option | Description | Selected |
|--------|-------------|----------|
| Phase 1 不需要 | 模块映射留到 Phase 2 | ✓ |
| 预留空 modules 块 | 放 modules: [] 空块 | |
| Phase 1 就实现 | 按目录自动推断 | |

**User's choices:** All recommended options selected.
**Notes:** 配置格式简洁实用，Phase 1 只关心 scope 块。

---

## 错误恢复策略

| Option | Description | Selected |
|--------|-------------|----------|
| 三级 confidence | high / low / none | ✓ |
| 两级 | ok / degraded | |
| 四级 | high / medium / low / none | |

| Option | Description | Selected |
|--------|-------------|----------|
| 有 AST 就输出 partial | tree-sitter error recovery 产出部分 CST → low | ✓ |
| 有 exports 才输出 partial | 至少有导出符号才降级 | |
| 按错误比例阈值 | 错误节点 < 30% → low | |

| Option | Description | Selected |
|--------|-------------|----------|
| 空事实 + error_message | confidence=none 时附带错误原因 | ✓ |
| 仅文件路径 + confidence | 不含空数组 | |
| 不输出 none 级文件 | 完全失败文件不进入 files[] | |

| Option | Description | Selected |
|--------|-------------|----------|
| 不影响，退出码始终 0 | anrsm run 是数据提取工具 | ✓ |
| 有 none 级时退出码 1 | CI 严格模式 | |
| 可配置 fail_on_confidence | anrsm.yaml 中配置 | |

**User's choices:** All recommended options selected.
**Notes:** 降级策略简洁，Agent 通过 confidence 字段自行判断结果质量。

---

## tree-sitter 提取范围

| Option | Description | Selected |
|--------|-------------|----------|
| 值导出 + 类型导出 | function/class/const/let/var + type/interface/enum | ✓ |
| 仅值导出 | 只提取值导出 | |
| 值 + 类型 + re-exports | 额外提取 export * from | |

| Option | Description | Selected |
|--------|-------------|----------|
| ESM + CommonJS | import/export + require() | ✓ |
| 仅 ESM | 只提取 import/export | |
| ESM + CommonJS + 动态 import | 额外提取 import() | |

| Option | Description | Selected |
|--------|-------------|----------|
| 名称 + 参数 + 返回类型 | funcName(params: Type): ReturnType | ✓ |
| 仅名称 + 参数名 | funcName(a, b) | |
| 完整签名 + JSDoc | 参数+返回类型+JSDoc | |

| Option | Description | Selected |
|--------|-------------|----------|
| 已知 API 集 | 硬编码副作用 API 列表 | ✓ |
| 仅 I/O 和网络 | 只检测 fs.* 和 fetch | |
| 所有外部调用 | 任何非本地函数调用 | |

**User's choices:** All recommended options selected.
**Notes:** 提取范围覆盖 TS/JS 生态的主要模式。

---

## 项目目录架构

**重要变更:** 项目名从 ANRSM 改为 **WTCD**（What The Code Doing），所有 crate 和配置使用 wtcd- 前缀。

| Option | Description | Selected |
|--------|-------------|----------|
| 多 crate workspace | 7+ crate 职责分离 | ✓ |
| 单 crate 多模块 | 一个 crate src/ 下分模块 | |
| 精简拆分（3 crate） | core + lib + cli | |

| Option | Description | Selected |
|--------|-------------|----------|
| core + scope + adapters + cli | Phase 1 四个 crate | ✓ |
| core + scope + adapters（无独立 cli） | main.rs 放 workspace root | |
| 全部 8 crate 都创建 | Phase 1 就创建所有 crate | |

| Option | Description | Selected |
|--------|-------------|----------|
| wtcd-xxx | crates/wtcd-core, crates/wtcd-scope 等 | ✓ |
| wtcd_xxx（下划线） | crates/wtcd_core 等 | |
| 缩写 | crates/core, crates/scope | |

| Option | Description | Selected |
|--------|-------------|----------|
| 扁平文件 | src/lib.rs + types.rs + ... | ✓ |
| 按关注点分子目录 | src/types/, src/adapter/ | |
| lib.rs 全放 | Phase 1 所有代码放 lib.rs | |

**wtcd-core files:** types + adapter + error + config (recommended)
**wtcd-scope files:** scanner + scope (recommended)
**wtcd-adapters files:** ts + lib.rs 注册 (recommended)
**wtcd-cli files:** commands/ 子目录 (recommended)

| Option | Description | Selected |
|--------|-------------|----------|
| workspace root tests/ | 集成测试在根目录 | |
| 各 crate 内部 tests/ | 就近测试 | |
| tests/ + 各 crate 单元测试 | 单元测试在 crate，集成测试在 root | ✓ |

| Option | Description | Selected |
|--------|-------------|----------|
| tests/fixtures/ 按语言分目录 | tests/fixtures/ts/, tests/fixtures/js/ | ✓ |
| 各 crate 自带 fixtures | wtcd-adapters/tests/fixtures/ | |
| examples/ 目录复用 | 复用现有 examples/ | |

| Option | Description | Selected |
|--------|-------------|----------|
| wtcd-cli 内嵌 include_str! | 配置模板嵌入二进制 | ✓ |
| 独立 templates/ 目录 | workspace root templates/ | |
| 代码中硬编码 | init.rs 中字符串拼接 | |

**User's choices:** All recommended options selected.
**Notes:** 项目从 ANRSM 正式改名为 WTCD，workspace 多 crate 结构。

---

## Agent's Discretion

- tree-sitter query 的具体实现方式（S-expression 语法细节）
- 各类型的精确字段定义（ArtifactId, ConfidenceBand 等）
- Cargo.toml 中的精确版本号和 feature flags
- CLI help 文本和 shell completion 生成

## Deferred Ideas

None — discussion stayed within phase scope.
