# Domain Pitfalls: Code Analysis & Semantic Mirror Systems

**Domain:** AI-native code semantic mirror / documentation generation tooling
**Researched:** 2026-03-21
**Confidence:** HIGH (project anti-pattern docs + external failure mode research)

---

## Executive Summary

代码分析和文档生成工具的失败模式极其集中。外部研究和项目自身的反模式文档高度一致：**最常见的失败不是技术做不到，而是团队把工具悄悄做成了没有语义闭环的"高级文档生成器"。** 以下是按严重程度排序的领域陷阱，每条都包含预警信号、预防策略和阶段映射。

---

## Critical Pitfalls

### Pitfall C1: 镜像沦为装饰——Agent 行为未改变

**What goes wrong:** 系统有镜像，但 Agent 仍然默认全量扫源码。镜像存在但没人读。

**Why it happens:**
- 路由索引无效或不存在（Agent 不知道该先读哪个镜像）
- 镜像质量不够高，Agent 读了也做不了决策
- Agent 工作流没有硬编码"先读镜像"步骤
- 没有运行时契约强制读取顺序

**Consequences:** 整个系统价值归零。投入了大量工程做镜像，但 token 消耗不降反升。

**Warning signs:**
- Agent 在 PR 中仍然大面积读取源码文件
- 镜像目录更新但无任何下游消费
- 开发者说"我看了镜像但还是得去看源码"

**Prevention:**
- CLI 输出必须包含明确的"读取顺序"引导（`read_order` 字段）
- Agent 集成第一步就强制先读镜像再决策
- 在 dogfood 阶段持续监控 Agent 实际读取路径

**Phase mapping:** M1（文件级镜像生成）必须同时产出路由索引原型；M4（Agent 集成）作为验证阶段必须有明确的行为变更指标。

---

### Pitfall C2: 门禁误报过高导致团队绕过

**What goes wrong:** CI 门禁频繁阻塞开发者，大量失败来自格式化改动或 C0 级噪声。团队开始 `--no-verify` 或关闭门禁。

**Why it happens:**
- 语义指纹对格式（空格、换行、注释）过于敏感
- C0/C1 分类边界不清
- 缺少保守豁免策略
- 缺少失败样本回归测试

**Consequences:** 门禁一旦被绕过，整个漂移检测失去意义。团队把 ANRSM 视为"烦人的 lint"。

**Warning signs:**
- 门禁失败率 > 30%
- 大量 failure 来自 C0（空格/格式）
- 开发者在 PR 讨论中抱怨"ANRSM 又拦我了"
- 出现 `ANRSM_SKIP=1` 之类的绕过环境变量

**Prevention:**
- 指纹算法必须排除纯格式差异（normalize whitespace）
- C0 改动不阻断门禁，仅报告
- 提供 `anrsm ci --warn-only` 模式用于初期磨合
- 建立失败样本回归集，每次指纹算法变更必须通过

**Phase mapping:** M2（语义指纹与漂移检测）是核心风险阶段。指纹算法设计必须有 ADR。M3（CI 门禁）初期应以 warn-only 模式运行至少 2 周。

---

### Pitfall C3: AST 解析的隐藏地雷

**What goes wrong:** AST 解析在边缘情况下产生不稳定结果：增量解析破坏 parse tree、某些语法构造导致解析器挂起、不同版本产生不同 AST。

**Why it happens:**
- Tree-sitter 增量解析在特定编辑操作后会产生 ERROR 节点，即使源码是合法的（[tree-sitter#4001](https://github.com/tree-sitter/tree-sitter/issues/4001)）
- 特定字符序列可导致解析器无限循环（[tree-sitter#322](https://github.com/tree-sitter/tree-sitter-javascript/issues/322)）
- 大型语法的 GLR 解析可能丢弃正确分支（[tree-sitter#3243](https://github.com/tree-sitter/tree-sitter/issues/3243)）
- 增量解析和全量解析结果不一致
- 语法版本更新改变 AST 结构

**Consequences:** 指纹不稳定 → 漂移报告假阳性 → 门禁不可信 → 团队放弃。

**Warning signs:**
- 同一文件在增量模式和全量模式下指纹不同
- parse tree 中出现 unexpected ERROR 节点
- tree-sitter 版本升级后大面积指纹变更
- 特定文件始终解析异常

**Prevention:**
- 指纹计算必须基于全量解析结果，不依赖增量解析缓存
- 建立 AST 解析的 golden test suite：已知文件 → 预期 parse tree
- parser 版本锁定，升级走 ADR
- 对解析失败的文件显式输出 `confidence: low` 而非静默忽略
- 设定解析超时（如 5 秒），超时视为解析失败

**Phase mapping:** M1（TS/JS 适配器）阶段必须建立 golden test suite。M2 指纹算法必须有"解析不稳定"的降级路径。

---

### Pitfall C4: 指纹跨版本/跨环境不稳定

**What goes wrong:** 工具版本升级后，同一仓库的指纹大面积变更。或不同 CI 环境产生不同指纹。团队无法信任指纹作为稳定标识。

**Why it happens:**
- 指纹算法包含了 parser 版本敏感的 AST 节点
- hash 算法或序列化方式在版本间变化
- 环境差异（行尾符、编码）影响 parse tree
- 缺少指纹版本化协议

**Consequences:** 大面积假漂移 → 门禁崩溃 → 需要全量重建 → 团队信任丧失。

**Warning signs:**
- 工具升级后 `drift report` 显示 > 50% 文件漂移
- CI 和本地产生不同指纹
- 同一 commit 在不同机器上指纹不同

**Prevention:**
- 指纹必须包含 schema 版本号（`fp_version: "1.0.0"`）
- 指纹算法只基于语义等价的 AST 节点（排除位置信息、token 顺序等 parser 实现细节）
- normalize 行尾符和编码
- 提供 `anrsm migrate` 命令在升级时重建指纹
- 升级路径必须有 ADR 记录

**Phase mapping:** M2 指纹设计阶段必须定义 `fp_version` 协议。每次适配器或 parser 升级触发 M5（版本化与重建）。

---

### Pitfall C5: 镜像变成第二真相源

**What goes wrong:** 工程师习惯先改镜像再改代码。知识层独立维护，与源码脱节。镜像描述的系统已不存在。

**Why it happens:**
- 镜像质量高，团队开始"信任"它超过源码
- 缺少强制的源码→镜像同步机制
- 人类知识层没有 `source_artifacts` 追踪
- 没有 freshness 信号

**Consequences:** 决策污染、Review 失真、Agent 被过期上下文误导。这是项目文档明确列为"最危险信号 #1"的反模式。

**Warning signs:**
- PR 中先改 `mirror/` 再改 `src/`
- 知识文档中没有 `source_artifacts` 引用
- 镜像内容与源码明显不一致但无人处理
- "这个镜像上次更新是 3 个月前"

**Prevention:**
- 每个镜像必须有 `source_artifacts` 字段，明确声明来源
- 门禁阻断"只改镜像不改源码"的提交
- 知识层必须从镜像"编译"，不允许独立事实
- 定期 `anrsm verify` 检查镜像与源码一致性

**Phase mapping:** 贯穿所有阶段。M3 门禁必须包含"镜像无源码变更"检测。M5 版本化必须确保镜像可重建。

---

### Pitfall C6: 默认全量重建杀死性能

**What goes wrong:** 每次提交都全量扫描全仓。仓库一大就慢到无法使用。团队绕过工具。

**Why it happens:**
- 增量能力长期不上线
- 缺少差异驱动的变更检测
- 全量重建作为默认路径而非恢复手段

**Consequences:** CI 时间暴增 → 开发者等不及 → 绕过 → 工具死亡。

**Warning signs:**
- CI 中 ANRSM 步骤 > 5 分钟
- 本地 `anrsm run` > 30 秒
- 开发者说"太慢了，我直接跳过"

**Prevention:**
- 增量更新从 M1 就必须是默认路径
- 全量重建只作为 `anrsm rebuild` 显式命令
- 基于 git diff 驱动变更检测
- 性能预算：单文件镜像 < 100ms，100 文件增量 < 10s

**Phase mapping:** M1 必须有增量能力原型。M2/M3 持续优化。M5 提供全量重建作为升级手段。

---

## Moderate Pitfalls

### Pitfall M1: 镜像正文写成"漂亮空话"

**What goes wrong:** 镜像段落很多，但没有责任边界、副作用、不变量。全是"这个文件负责处理 XX 逻辑"之类低信息密度句子。

**Why it happens:**
- 过度依赖 LLM 直接生成自然语言总结
- 没有结构化事实层约束
- 模板过于宽泛

**Consequences:** Agent 和人类都无法基于镜像做决策。Token 节约了但信息价值消失了。（项目文档反模式 A6）

**Warning signs:**
- 镜像中没有出现 `side_effects`、`invariants`、`boundary` 等关键词
- 不同文件的镜像内容高度相似
- Agent 读了镜像后仍然需要读源码才能做决策

**Prevention:**
- 镜像模板必须强制包含：责任边界、控制流摘要、副作用、不变量、展开条件
- 结构化事实层先于自然语言生成
- 建立镜像质量审计：抽样检查是否有具体契约信息

**Phase mapping:** M1 镜像生成器设计阶段必须定义严格的模板 schema。

---

### Pitfall M2: Prompt 直接生成镜像事实（无结构化提取）

**What goes wrong:** 没有 AST 级结构化提取，让 LLM 直接看源码输出镜像。结果不可复现、不可验证。

**Why it happens:**
- 结构化提取工程量大，LLM 直接生成"看起来不错"
- 早期 demo 效果好，团队认为够用

**Consequences:** 幻觉、不可复现、无法做差异分析、无法自动门禁。（项目文档反模式 A2）

**Warning signs:**
- 镜像中出现源码中不存在的函数名或参数
- 同一文件两次生成镜像内容不同
- 镜像描述的行为与实际代码逻辑矛盾

**Prevention:**
- **必须**先建立结构化事实层（AST 提取 → JSON → 然后才允许 LLM 压缩/重组）
- LLM 只允许在结构化事实基础上做自然语言润色，不允许"发明"事实
- 建立事实一致性校验

**Phase mapping:** M1 架构决策。这是核心设计约束，不可妥协。

---

### Pitfall M3: 把所有文件镜像到同一强度

**What goes wrong:** 低价值样板代码和高风险核心文件使用相同镜像粒度。系统在低价值区域浪费成本。

**Consequences:** 整体 ROI 下降、维护成本过高。（项目文档反模式 B3）

**Warning signs:**
- `node_modules/` 或 `dist/` 附近文件有镜像
- 配置文件和核心业务逻辑使用相同模板
- 镜像生成时间与文件数量线性增长

**Prevention:**
- Scope Manager 优先级分层：高风险/高频/高价值文件优先
- 排除范围明确（依赖、锁文件、编译输出）
- 灰区工件需要显式配置判断

**Phase mapping:** M1 Scope Manager 设计必须包含优先级机制。

---

### Pitfall M4: 知识层提前平台化

**What goes wrong:** 在文件镜像和门禁没稳定前，先做漂亮 UI 和知识门户。

**Consequences:** 表层体验好但底层事实不稳定，平台放大错误。（项目文档反模式 B4）

**Warning signs:**
- 有 Web UI 但没有 CLI
- 知识层比镜像层先完成
- 团队在讨论"UI 用什么框架"而非"指纹算法怎么设计"

**Prevention:**
- **严格**遵循 M0-M5 顺序：先规范 → 镜像 → 指纹 → 门禁 → 路由 → Agent
- 知识层推迟到里程碑后段
- v1 不做 Web UI

**Phase mapping:** 全阶段。项目文档已明确"不做 Web UI"。

---

### Pitfall M5: 忽略"何时必须展开源码"

**What goes wrong:** 团队假设镜像足够强大，可以替代源码阅读。Agent 在复杂变更上只看镜像不看源码。

**Consequences:** 精确修改失败、影响面漏判、高风险变更引入回归。（项目文档反模式 A7）

**Warning signs:**
- Agent 提交的 PR 引入回归但"镜像看起来没问题"
- 没有 `expand_condition` 字段
- 团队认为"有了镜像就不需要看源码了"

**Prevention:**
- 每个镜像必须有 `expand_condition`（何时必须展开源码）
- 高风险变更默认要求源码展开
- Agent 集成必须有"镜像→源码"的升级路径

**Phase mapping:** M1 模板设计。M4 Agent 集成必须实现升级路径。

---

### Pitfall M6: 缺少低置信度机制

**What goes wrong:** 解析不稳定时仍输出高自信镜像。门禁无法区分高信与低信工件。

**Consequences:** 系统表面正常，实际在关键时刻误导最严重。（项目文档反模式 A8）

**Warning signs:**
- 所有镜像都是 `confidence: high`
- 没有 `confidence_band` 字段
- 门禁对低置信度镜像和高置信度镜像同等对待

**Prevention:**
- 每个镜像必须输出 `confidence_band`
- 门禁策略对低置信度做保守处理（warn-only 而非 block）
- 低置信度镜像标记为需要人工复核

**Phase mapping:** M1 模板设计必须包含 confidence 字段。M2 指纹算法必须有置信度输出。M3 门禁必须根据置信度分级。

---

## Minor Pitfalls

### Pitfall m1: 术语不固定

**What goes wrong:** 团队一会儿说 mirror，一会儿说 doc，一会儿说 note。工具接口混乱。

**Prevention:** 术语表已冻结（项目文档 01），代码/文档/CLI/报告统一命名。每次 ADR 必须检查术语一致性。

**Phase mapping:** M0 已完成（规范冻结）。后续阶段持续检查。

---

### Pitfall m2: 语言适配器过度工程化

**What goes wrong:** 为"未来可能支持的语言"设计过度抽象的适配器接口。实际只用 TS/JS。

**Prevention:** 适配器接口从 TS/JS 实际需求出发，不过度抽象。扩展语言时再泛化。

**Phase mapping:** M1 TS/JS 适配器。第二个语言适配器时再评估接口泛化。

---

### Pitfall m3: 配置文件变成事实上的第二真相源

**What goes wrong:** `anrsm.yaml` 中的手动配置（排除路径、优先级）与实际代码结构脱节。

**Prevention:** 配置变更必须与代码变更同步。提供 `anrsm config validate` 检查配置与实际目录结构一致性。

**Phase mapping:** M1 配置系统设计。

---

### Pitfall m4: 生成产物被错误镜像

**What goes wrong:** 盲目镜像所有代码生成产物（如 protobuf 生成的 .ts），而不是镜像上游 schema。

**Consequences:** 数量大、变化频繁、价值低。噪声漂移淹没真实信号。（项目文档 03 §8）

**Prevention:** 优先镜像生成器源头/模板/schema。生成产物默认排除，需要显式声明纳入。

**Phase mapping:** M1 Scope Manager 灰区处理。

---

### Pitfall m5: dogfood 仓库太特殊导致无法泛化

**What goes wrong:** ANRSM 自身作为试点仓库，但它的结构（文档驱动、规范先行）不代表典型用户仓库。

**Prevention:** 在 dogfood 之后尽快找一个"脏"的真实仓库试点（有遗留代码、混合风格、不规范目录）。

**Phase mapping:** M1 完成后、M3 之前找第二个试点仓库。

---

## Adoption Pitfalls

### Pitfall A1: Time-to-First-Value 太长

**What goes wrong:** 开发者需要 30 分钟以上才能看到 ANRSM 的价值。95% 的 DevTool 用户在首次体验后放弃。（外部研究：DevTools 转化率 2-5%）

**Warning signs:**
- "Getting Started" 需要 > 5 步
- 没有 `anrsm init && anrsm run` 就能出结果的快速路径
- 首次运行输出一堆 warning/error 而非有价值的结果

**Prevention:**
- `anrsm init` 必须在 30 秒内生成可读的镜像预览
- 首次运行只处理最关键的 10 个文件，快速展示价值
- 提供 `anrsm demo` 用示例仓库即时演示

**Phase mapping:** M1 CLI 设计。这是决定工具生死的 UX 问题。

---

### Pitfall A2: 文档工具被视为"额外负担"

**What goes wrong:** 镜像更新被团队视为"在写代码之外还要做的事"。没有人主动维护。

**Warning signs:**
- PR 中代码改了但镜像没改
- "这个等有空再更新"
- 镜像更新总是由同一个人完成

**Prevention:**
- 门禁强制同步（M3）
- 镜像更新必须是开发工作流的一部分，而非额外步骤
- `anrsm watch` 模式：代码变更自动触发镜像更新

**Phase mapping:** M3 门禁 + M4 Agent 集成。

---

### Pitfall A3: 输出格式不适合消费端

**What goes wrong:** CLI 输出人类可读但 Agent 不好解析，或 Agent 格式但人类看不懂。

**Prevention:**
- JSON 为机器消费格式（CLI `--format json`）
- Markdown 为人类消费格式（默认）
- 两套格式必须包含相同语义信息

**Phase mapping:** M1 CLI 设计。

---

## Phase-Specific Warnings Summary

| Phase | Critical Pitfalls | Mitigation |
|-------|------------------|------------|
| M0 (规范冻结) | m1 术语不固定 | 已完成，持续检查 |
| M1 (文件级镜像) | C3 AST 解析地雷, C6 全量重建, M1 空话镜像, M2 无结构化提取, M3 等强度镜像, M6 无置信度 | Golden test suite, 增量默认, 严格模板, 优先级分层, confidence 字段 |
| M2 (指纹与漂移) | C2 门禁误报, C4 指纹不稳定 | normalize 算法, fp_version, 失败样本回归 |
| M3 (CI 门禁) | C2 门禁误报, A2 额外负担 | warn-only 初期模式, 保守豁免策略 |
| M4 (Agent 集成) | C1 Agent 行为未改变, M5 忽略展开源码 | 强制读取顺序, 升级路径 |
| M5 (版本化) | C4 指纹跨版本不稳定 | anrsm migrate, schema 版本化 |
| 全阶段 | C5 第二真相源, M4 知识层提前平台化 | source_artifacts 追踪, 严格阶段顺序 |

---

## 三个最危险的信号

项目文档（docs/11）已明确定义。这里从 pitfall 视角补充：

1. **团队开始直接手改镜像** → 说明镜像已成为第二真相源（C5）
2. **Agent 依然默认大面积读源码** → 说明整个系统价值归零（C1）
3. **门禁经常被关闭或绕过** → 说明门禁信任已破产（C2）

出现任意两个，必须停下来做规范回正。

---

## Sources

### Internal
- `docs/11_AntiPatterns_FailureModes_and_Decision_Record_Guide.md` — 一级/二级反模式、失败模式
- `docs/03_System_Boundaries_Assumptions_and_NonGoals.md` — 边界、非目标、假设
- `.planning/PROJECT.md` — 项目约束和关键决策

### External (Research)
- Tree-sitter issues: [#4001](https://github.com/tree-sitter/tree-sitter/issues/4001) (incremental vs fresh parse inconsistency), [#322](https://github.com/tree-sitter/tree-sitter-javascript/issues/322) (parser infinite loop), [#3243](https://github.com/tree-sitter/tree-sitter/issues/3243) (GLR discards valid branch), [#1444](https://github.com/tree-sitter/tree-sitter/issues/1444) (incremental parse corruption)
- [Inconsistent Fingerprint (Semgrep→Opengrep)](https://github.com/opengrep/opengrep/issues/230) — fingerprint instability across tool versions
- [AI Documentation Debt](https://techdebt.guru/ai-documentation-debt/) — stale-on-arrival, verbose emptiness, hallucinated references
- [Quality Gates in CI/CD 2026](https://agileverify.com/quality-gates-in-ci-cd-what-should-really-block-a-release-in-2026/) — gate bypass patterns, false positive fatigue
- [Over-Reliance on Static Analysis](https://medium.com/@kurdish_57861/the-unseen-cost-of-over-reliance-on-static-analysis-in-your-ci-cd-pipeline-4500310c0a2c) — behavioral debt, false confidence from green gates
- [DevTool Adoption Failure](https://medium.com/@houseofarby/why-95-out-of-100-developers-abandon-your-devtool-and-its-not-your-features-1c70bfed2435) — 95% abandonment, time-to-first-value problem
- [Internal Tool Adoption](https://www.h-systems.dev/en/blog/why-most-internal-tools-fail-adoption-not-ux) — systems thinking failure, unclear ownership
- [AI Code Quality Gate](https://codeintelligently.com/blog/ai-code-quality-gate-ci-cd) — SCAN pipeline, pattern-specific gates
