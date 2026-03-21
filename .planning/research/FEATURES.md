# Feature Landscape: ANRSM (AI Native Repo Semantic Mirror)

**Domain:** Code Analysis & Documentation — Agent-Oriented Semantic Mirror
**Researched:** 2026-03-21
**Confidence:** MEDIUM-HIGH (based on verified 2025-2026 sources)

---

## Executive Summary

ANRSM occupies a unique niche between four established tool categories:

1. **传统文档生成器**（Doxygen、TypeDoc、JSDoc、Sphinx）— 从注释生成 API 文档
2. **代码智能平台**（Sourcegraph、CodeQL）— 代码搜索、导航、静态分析
3. **AI 编码助手**（Claude Code、Cursor、GitHub Copilot）— 语义理解和代码生成
4. **新兴语义索引工具**（CocoIndex Code、Swimm Deep Index）— AST 感知的语义搜索

ANRSM 不是这四类中任何一类的替代品，而是**填补了一个结构性空白**：现有工具要么面向人类（文档生成器），要么不保存持久化中间层（AI 助手做一次性搜索），要么关注安全/搜索而非"为 Agent 建立可验证的语义镜像"。

关键洞察：2026 年，AI 编码助手普遍面临"上下文工程"（context engineering）瓶颈 — Agent 花 60%+ 的时间在搜索上下文上（Morph 2026 报告）。ANRSM 直接解决这个问题：提供预计算的、可验证的语义层，而非让 Agent 每次都重扫。

---

## Table Stakes（基线特性）

用户预期存在。缺失 = 用户离开。

| Feature | Why Expected | Complexity | Notes |
|---------|--------------|------------|-------|
| **AST 级代码解析** | 所有竞争工具都有：TypeDoc、CodeQL、tree-sitter、Sourcegraph SCIP | High | TS/JS 首批；tree-sitter 或 SWC 是成熟基础 |
| **符号提取与索引** | Sourcegraph、Cursor、CocoIndex 都提供符号级索引 | Medium | 函数/类/接口/类型/导出 |
| **依赖图生成** | CodeQL、Sourcegraph 均以依赖图为核心能力 | Medium | 文件级和模块级依赖边 |
| **多语言支持架构** | Doxygen 支持 10+ 语言；CodeQL 支持 17 种；tree-sitter 支持 40+ | High | v1 聚焦 TS/JS，但必须从架构上支持插件化适配器 |
| **CLI 可用性** | Doxygen、Sphinx、tree-sitter 均为 CLI 工具 | Low | 基本的 init/generate/diff/check 子命令 |
| **配置文件驱动** | Doxygen（Doxyfile）、Sphinx（conf.py）均以配置文件为中心 | Low | anrsm.yaml |
| **输出格式化文档** | 几乎所有工具都输出 HTML/PDF/JSON | Low | ANRSM 选择 Markdown + YAML Front Matter |
| **CI/CD 集成** | CodeQL 在 GitHub Actions 原生集成；Doxygen/Sphinx 有 CI 钩子 | Medium | pre-commit + PR 级门禁 |
| **增量更新** | tree-sitter 做增量解析；Sourcegraph 做增量索引 | High | 基于 git diff 的变更集驱动更新 |
| **JSON 结构化输出** | TypeDoc、CodeQL 均支持 JSON 输出；Agent 生态期望结构化数据 | Low | CLI --format json |

---

## Differentiators（差异化特性）

ANRSM 的竞争壁垒。不是"更好的文档"，而是"不同的品类"。

### Tier 1: 核心差异化（定义品类）

| Feature | Value Proposition | Complexity | Notes |
|---------|-------------------|------------|-------|
| **语义镜像持久化层** | 不是"生成后丢弃"，而是与源码一一映射的可版本管理工件。没有竞品做这件事 — CodeQL 做数据库但不暴露；Sourcegraph 做索引但不持久化为镜像文件 | High | Markdown + YAML Front Matter，可 diff、可 git 管理 |
| **语义指纹 + 漂移检测** | 区分 C0/C1/C2/C3 四级语义变化。没有竞品做"语义级"的文档新鲜度验证 — Doxygen 只检查时间戳；Swimm 检查代码片段关联但不做语义指纹 | High | 核心差异点。让"文档是否过期"变成可验证的数学问题 |
| **Material Change 分类** | 格式化改动（C0）不触发阻塞；导出签名变更（C2）触发阻塞。竞品要么不区分（Doxygen）要么只做安全分类（CodeQL） | Medium | C0/C1/C2/C3 四级分类是 ANRSM 独有 |
| **Agent 读取顺序契约** | "先路由 → 镜像 → 源码"不是建议，是契约。没有竞品强制 Agent 的读取行为 — Cursor/Copilot 做语义搜索但不约束消费顺序 | Medium | 这是 ANRSM 价值闭环的关键：不只是提供数据，而是重塑 Agent 行为 |
| **Freshness State 机器** | fresh/stale/invalid/unknown 四态 + 失败码。竞品没有这种"文档可信度状态机" | Medium | 状态转换规则是核心知识产权 |

### Tier 2: 重要差异化（构建优势）

| Feature | Value Proposition | Complexity | Notes |
|---------|-------------------|------------|-------|
| **路由索引** | 从任务描述映射到候选模块/文件。Sourcegraph Deep Search 做类似的事，但每次实时查询；ANRSM 预计算路由索引 | Medium | 预计算 vs 实时查询 = 更快 + 更可预测 |
| **机器可读文档头** | 每个镜像文件的 YAML Front Matter 包含来源、指纹、生成器版本、置信度。这是可审计性的基础。TypeDoc/JSDoc 的文档头是面向人类的，不包含验证元数据 | Low | Schema 已定义（mirror-header） |
| **两段式生成流水线** | 结构化提取 → 语义压缩。竞品要么只做提取（Doxygen），要么只做压缩（AI 助手）。ANRSM 将两者分离，让压缩有结构化锚点 | Medium | 公理 A4 的实现 |
| **三层层级架构** | 文件级镜像 → 模块级镜像 → 人类知识层。竞品通常只有 API 文档（文件级）或架构文档（高级），没有中间的模块级语义层 | Medium | 模块镜像是独特的"中间粒度" |
| **门禁基于漂移而非存在** | ANRSM 不检查"你有没有写文档"，而是检查"文档对当前源码是否仍然有效"。这是根本不同的治理理念 | Medium | 公理 A6 |
| **Agent 写回义务** | Agent 修改代码后必须更新镜像。没有竞品强制这种双向同步 — Cursor/Copilot 修改完代码就结束了 | Medium | 这确保了镜像的长期有效性 |
| **压缩效率指标** | source_tokens / mirror_tokens。有明确的压缩收益目标。Doxygen/Sphinx 不关注 Token 经济性 | Low | 面向 Agent 使用场景的独特指标 |

### Tier 3: 生态差异化（长期壁垒）

| Feature | Value Proposition | Complexity | Notes |
|---------|-------------------|------------|-------|
| **Rust 单二进制分发** | 无依赖运行时。Doxygen 需要 Graphviz；Sphinx 需要 Python 环境；TypeDoc 需要 Node.js | High | 从 crates.io 或 GitHub Releases 下载即用 |
| **Agent 集成协议层** | JSON 输出 + CLI 驱动 = 任何 Agent 都能调用。不绑定 MCP Server（Sourcegraph/CocoIndex 走 MCP）或特定 IDE（Cursor 走 VS Code） | Low | 更广泛的兼容性 |
| **失败样本库机制** | 系统性记录漏检、误检、低置信度误导案例。没有竞品这样做 — CodeQL/Semgrep 有规则库但没有失败案例追踪 | Low | 长期质量改进的关键资产 |
| **dogfooding 自举** | ANRSM 用自己的镜像系统管理自身代码。这是可信度和可行性的终极验证 | Medium | 本仓库作为第一个试点 |

---

## Anti-Features（不应构建的特性）

明确不做的事项。避免范围蔓延。

| Anti-Feature | Why Avoid | What to Do Instead |
|--------------|-----------|-------------------|
| **Web UI / Dashboard** | v1 范围外。Sourcegraph、Swimm 都花大量精力在 UI 上，ANRSM 的差异化不在于可视化 | CLI + JSON 输出。UI 留给消费端 |
| **实时后台服务** | ANRSM 是 CLI 按需运行，不是常驻服务。Doxygen/Sphinx 也是 CLI 模式，验证了这种简单性 | 按需 CLI 调用 + pre-commit/CI hook |
| **多仓库全局图** | v1 聚焦单仓。Sourcegraph 7.0 的跨仓库能力是其差异化，但对 v1 ANRSM 是过度设计 | 单仓深度优先于多仓广度 |
| **安全漏洞扫描** | CodeQL 和 Semgrep 已经做到极致。ANRSM 不应该在安全规则编写上投入 | 聚焦语义新鲜度，不聚焦安全合规 |
| **代码生成 / 自动修复** | 这是 Cursor/Claude Code/Copilot 的领地。ANRSM 是"让 Agent 更好地理解代码"，不是"替 Agent 写代码" | 输出结构化语义数据，不输出代码建议 |
| **自由文本 Wiki** | 人类知识层必须从镜像或结构化事实派生。开放 Wiki 很快会变成双真相源（风险 R1） | 知识层从镜像自动编译（Knowledge Compiler） |
| **Prompt 直接生成文档** | 违反公理 A4（结构化事实优先）。LLM 生成无锚点的文档不可验证 | 两段式流水线：先 AST 提取，再约束压缩 |
| **替代 LSP / IDE 智能** | LSP 做实时编辑辅助（补全、跳转定义），ANRSM 做持久化语义镜像，是互补关系 | 不与 LSP 竞争，可集成 |
| **嵌入向量 / 向量搜索** | CocoIndex Code 走向量搜索路线，但 ANRSM 的路由索引是确定性的符号级映射，不需要模糊搜索 | 确定性路由（基于符号和依赖）优于概率性搜索 |
| **MCP Server** | v1 选择 CLI 驱动模式。MCP 绑定特定传输层；CLI 更通用 | CLI + JSON = 任何 Agent 可调用 |
| **实时代码监控** | 不做文件系统 watcher。增量更新基于 git diff，不是实时事件 | git diff 驱动的变更检测 |

---

## Feature Dependencies（特性依赖关系）

```
AST 解析 → 符号提取 → 语义指纹 → 漂移检测 → CI 门禁
    ↓           ↓           ↓           ↓
依赖图生成   路由索引   镜像生成   漂移报告
    ↓           ↓           ↓
模块级镜像   Agent 读取   人类知识层
                顺序契约
```

关键路径：
1. `AST 解析` 是所有下游的基础
2. `语义指纹` 是漂移检测的前提
3. `漂移检测` 是门禁的前提
4. `镜像生成` 必须在 Agent 集成之前可用

---

## Competitive Positioning Matrix

| 维度 | Doxygen/TypeDoc | Sourcegraph | Cursor/Copilot | CocoIndex Code | **ANRSM** |
|------|-----------------|-------------|----------------|----------------|-----------|
| 目标用户 | 人类开发者 | 人类开发者 | 人类开发者 | AI Agent | **AI Agent + 人类** |
| 核心产出 | API 文档 HTML | 搜索结果 | 代码建议 | 语义搜索片段 | **持久化语义镜像** |
| 与源码同步 | ❌ 时间戳检查 | ❌ 不持久化 | ❌ 无持久化 | ❌ 索引可重建 | **✅ 漂移检测** |
| Agent 行为约束 | ❌ | ❌ | ❌ | ❌ | **✅ 读取契约** |
| Token 经济性 | ❌ 不关注 | ⚠️ 搜索时消耗 | ⚠️ 大量源码输入 | ✅ 70% 节省 | **✅ 压缩优先** |
| 可验证新鲜度 | ❌ | ❌ | ❌ | ❌ | **✅ 语义指纹** |
| CI 门禁 | ❌ 格式检查 | ❌ | ❌ | ❌ | **✅ 漂移门禁** |
| 单二进制分发 | ⚠️ 需依赖 | ❌ SaaS/自建 | ❌ IDE 插件 | ⚠️ Python | **✅ Rust 二进制** |

---

## MVP Recommendation

### Phase 1: 必须（建立品类基础）
1. AST 解析（tree-sitter for TS/JS）
2. 符号提取与依赖图
3. 文件级语义镜像生成（Markdown + YAML Front Matter）
4. 语义指纹计算
5. 基本 CLI（init / generate / check）

### Phase 2: 应该（构建差异化）
6. 漂移检测（C0/C1/C2 分类）
7. 增量更新（git diff 驱动）
8. 路由索引
9. CI 门禁集成

### Phase 3: 可选（放大价值）
10. 模块级镜像
11. Agent 读取顺序契约与写回
12. 人类知识层编译
13. 多语言适配器

### 推迟（v2 或更后）
- Web UI
- 多仓库图
- 向量搜索
- MCP Server

---

## Sources

- Doxygen features: devtoolsguide.com (2026-02-09), grokipage.com comparison, zipdo.co (2026-03-12)
- TypeDoc/JSDoc: npm-compare.com feature comparison, devtoolsguide.com (2026-02-09)
- CodeQL: docs.github.com (2026-03-17), dev.to Semgrep vs CodeQL (2026-03-13), konvu.com (2026-03-16)
- Sourcegraph: sourcegraph.com/blog (2026-02-25 v7.0 announcement), sourcegraph.com docs
- Tree-sitter: atoms.dev analysis (2025-12-15), batsov.com (2026-02-27), dasroot.net (2026-02-28)
- AI coding assistants: octavehq.com (2026-02-25), emergent.sh (2026-03-12), dev.to (2026-03-10), faros.ai (2026-01-02)
- CocoIndex Code: github.com/cocoindex-io/cocoindex-code (2026-03-19), pub.towardsai.net (2026-03-18)
- Swimm: swimm.io (2026-03-04 benchmark), hpcwire.com (2026-02-10), techiehub.blog (2026-02-23)
- Agentic coding trends: Anthropic 2026 report, morphllm.com (2026-02-23)

---

## Confidence Assessment

| Area | Confidence | Reason |
|------|------------|--------|
| 竞品功能覆盖 | HIGH | 基于多个 2026 年来源交叉验证 |
| 差异化判断 | HIGH | ANRSM 设计文档与竞品功能直接对比 |
| 复杂度估算 | MEDIUM | 基于类比（tree-sitter 高复杂度、CLI 低复杂度），无实际实现验证 |
| 市场定位 | MEDIUM | 没有找到直接竞品，差异化是推断性的，需用户验证 |

## Gaps to Address

- 实际 AST 提取效果的验证（需要对真实 TS/JS 仓库做 PoC）
- 语义指纹算法的具体设计（什么算 C1 vs C2？阈值如何定？）
- Agent 实际使用 ANRSM 的行为数据（是否真的改变了读取顺序？）
- 压缩效率的量化基准（镜像 vs 源码的 Token 比例在真实场景中是多少？）
