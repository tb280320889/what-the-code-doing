# 05. 信息模型与仓库目录布局

状态：Normative  
版本：v1  
优先级：P0

---

## 1. 目的

本文件定义 ANRSM 的信息模型、工件类型、目录布局、路径映射规则、元数据组织方式，以及各类镜像工件之间的关系。

如果没有统一的信息模型，ANRSM 会在以下环节失控：

- 路径映射不稳定；
- 多语言工件冲突；
- 索引和镜像难以互相引用；
- 漂移报告无法定位问题；
- Agent 难以建立固定读取顺序。

---

## 2. 设计原则

信息模型 MUST 满足以下原则：

1. **确定性**  
   相同源码路径在相同配置下 MUST 映射到相同镜像路径。

2. **双向可追溯**  
   任意镜像工件 MUST 能追溯到其来源源码；任意源码 SHOULD 能找到其镜像工件。

3. **文本化与可 diff**  
   v1 的核心工件 SHOULD 使用文本格式，便于版本控制、差异审阅和生成器调试。

4. **机器优先，人工可读**  
   头字段和索引首先服务于机器；正文同时服务于机器和人类审阅。

5. **局部更新友好**  
   目录结构与索引设计 SHOULD 支持按文件和模块做局部更新。

---

## 3. 顶层目录建议

ANRSM v1 推荐在仓库根目录引入以下结构：

```text
repo/
  src/
  apps/
  packages/
  tests/
  mirror/
    file/
    module/
    index/
    reports/
  knowledge/
    architecture/
    features/
    operations/
    onboarding/
  .anrsm/
    anrsm.yaml
    cache/
    logs/
```

说明：

- `mirror/` 是机器优先语义镜像层；
- `knowledge/` 是人类优先知识层；
- `.anrsm/` 是系统配置、缓存和运行状态目录；
- 真实源码目录保持原样，不应为了适配 ANRSM 强行改造业务布局。

---

## 4. 工件类型模型

### 4.1 file_mirror

作用：

- 与单个源码文件一一映射；
- 提供任务定位、语义快速阅读、漂移判断和依赖入口。

最小组成：

- 文档头；
- 正文规范段落；
- 可选的人工补充区块。

### 4.2 module_mirror

作用：

- 聚合一组文件镜像；
- 表达模块责任、模块边界、外部契约、依赖关系、稳定性和常见风险。

生成来源：

- 文件镜像；
- 模块规则；
- 依赖图；
- 契约汇总。

### 4.3 routing_index

作用：

- 从任务文本映射到候选模块/文件/符号；
- 缩短 Agent 定位路径。

### 4.4 symbol_index

作用：

- 管理导出符号、关键内部符号、定义位置和使用邻域；
- 辅助影响分析与精确路由。

### 4.5 dependency_index

作用：

- 管理 import、调用、事件、配置读取、数据流邻接等关系；
- 支撑变更邻域扩展。

### 4.6 drift_report

作用：

- 记录当前变更集引发的镜像漂移结果；
- 作为门禁输入与诊断输出。

### 4.7 knowledge_doc

作用：

- 为架构师、接手工程师、值班工程师和 Reviewer 提供可读知识材料；
- 来源必须可追溯。

---

## 5. 路径映射规则

### 5.1 文件镜像路径

推荐规则：

```text
<repo_root>/<mirror_root>/file/<source_relative_path>.md
```

示例：

```text
src/auth/login.ts
-> mirror/file/src/auth/login.ts.md

apps/web/pages/home.tsx
-> mirror/file/apps/web/pages/home.tsx.md
```

要求：

1. 目录层级 MUST 保持与源码相同；
2. 源码扩展名 MUST 保留在文件名中；
3. 镜像扩展名 MUST 追加 `.md`；
4. 不允许用路径哈希取代可读路径作为默认规则，除非发生文件系统限制。

### 5.2 模块镜像路径

推荐规则：

```text
mirror/module/<module_id>.md
```

若 `module_id` 包含多级路径，则直接映射为层级目录：

```text
module_id = billing/invoice
-> mirror/module/billing/invoice.md
```

### 5.3 知识文档路径

推荐规则：

```text
knowledge/<category>/<logical_name>.md
```

类别建议：

- `architecture`
- `features`
- `operations`
- `onboarding`

---

## 6. 文件级镜像的信息模型

### 6.1 文件级镜像由两部分组成

1. **机器可读文档头**  
   用于追踪、校验、策略判断和索引关联。

2. **机器优先、人工可读的正文**  
   用于压缩语义、表达边界和提供展开提示。

### 6.2 文档头最小字段集

v1 建议最小字段如下：

- `anrsm_version`
- `artifact_type`
- `artifact_id`
- `source_path`
- `source_language`
- `module_id`
- `source_fingerprint`
- `semantic_fingerprint`
- `source_commit`
- `freshness_state`
- `confidence_band`
- `generator_name`
- `generator_version`
- `generated_at`
- `exports`
- `dependencies`
- `risk_tags`

说明：

- `exports` 和 `dependencies` MAY 做轻量摘要，不必在头中放入全部细节；
- 详细字段可由索引承载。

### 6.3 正文的标准段落

v1 推荐以下标准段落结构：

1. `职责`
2. `对外契约`
3. `输入与前置条件`
4. `状态与控制流`
5. `副作用与外部依赖`
6. `关键不变量与风险`
7. `变更影响`
8. `何时必须展开源码`

这个结构兼顾：

- Agent 的快速定位；
- Reviewer 的快速核验；
- 人类维护者的快速理解。

---

## 7. 模块级镜像的信息模型

模块级镜像不是多个文件摘要的简单拼接。  
它 SHOULD 回答模块层面的七个问题：

1. 这个模块的唯一职责是什么；
2. 对外暴露了哪些稳定契约；
3. 依赖了哪些外部模块或基础设施；
4. 内部有哪些关键子域或子流程；
5. 该模块的主要状态与一致性边界是什么；
6. 哪些文件是高影响入口；
7. 哪些变更会波及上游或下游。

### 7.1 模块级镜像最小头字段建议

- `artifact_type: module_mirror`
- `artifact_id`
- `module_id`
- `source_scope`
- `aggregate_semantic_fingerprint`
- `freshness_state`
- `generator_name`
- `generator_version`
- `generated_at`
- `critical_files`
- `external_contracts`

### 7.2 模块级正文建议

1. 模块职责
2. 模块边界
3. 稳定契约
4. 依赖关系
5. 状态与一致性
6. 关键文件与入口
7. 高频变更面
8. 常见故障与风险

---

## 8. 索引的信息模型

### 8.1 routing_index

路由索引 SHOULD 至少包含：

- `artifact_id`
- `module_id`
- `source_path`
- `semantic_keywords`
- `exported_symbols`
- `contract_terms`
- `risk_tags`
- `adjacent_artifacts`
- `confidence_band`

### 8.2 symbol_index

符号索引 SHOULD 至少包含：

- `symbol_name`
- `symbol_kind`
- `defined_in`
- `exported`
- `module_id`
- `references`
- `related_contracts`

### 8.3 dependency_index

依赖索引 SHOULD 至少包含：

- `from_artifact_id`
- `to_artifact_id`
- `dependency_kind`
- `strength`
- `path`
- `evidence`

`dependency_kind` 可取值示例：

- `import`
- `call`
- `event_publish`
- `event_consume`
- `config_read`
- `schema_bind`
- `storage_access`

---

## 9. 漂移报告的信息模型

漂移报告是门禁的直接输入。  
它必须同时能服务于机器判断和人工诊断。

最小字段建议：

- `report_version`
- `report_id`
- `repo_commit_before`
- `repo_commit_after`
- `generated_at`
- `drift_level`
- `changed_files`
- `affected_artifacts`
- `material_changes`
- `policy_result`
- `recommendations`

说明：

- `changed_files` 记录源码变更集；
- `affected_artifacts` 记录受影响镜像；
- `material_changes` 记录实质变更分类与证据。

---

## 10. 知识文档的信息模型

知识文档不要求与单个源码文件一一映射，但 MUST 具备来源追踪。  
建议头字段：

- `artifact_type: knowledge_doc`
- `artifact_id`
- `knowledge_category`
- `source_modules`
- `source_artifacts`
- `freshness_state`
- `generator_name`
- `generator_version`
- `generated_at`

知识文档正文结构视类别而定，但 SHOULD 避免与镜像正文冲突和重复。

---

## 11. 目录布局与权限边界

### 11.1 机器层与人工层分离

推荐将 `mirror/` 与 `knowledge/` 分离，理由如下：

1. 明确机器优先与人类优先的职责差异；
2. 减少人工误编辑机器区块；
3. 便于策略和 Code Owner 隔离；
4. 便于后续独立发布知识层。

### 11.2 `.anrsm/` 目录的角色

`.anrsm/` SHOULD 用于：

- 配置；
- 缓存；
- 运行日志；
- 临时构建状态。

`.anrsm/` 中的缓存和日志 SHOULD NOT 被视为规范工件。  
真正要进入版本管理的规范工件应位于 `mirror/` 和 `knowledge/`。

---

## 12. 版本管理建议

### 12.1 应进入版本管理的内容

- `mirror/file/**`
- `mirror/module/**`
- `mirror/index/**`（视团队策略，可只提交稳定索引）
- `knowledge/**`
- `.anrsm/anrsm.yaml`

### 12.2 不应进入版本管理的内容

- 临时缓存；
- 本地诊断日志；
- 解析中间产物；
- 大型临时图数据库快照；
- 运行时锁文件。

---

## 13. 稳定性规则

### 13.1 路径稳定性

一旦确定路径映射规则，v1 周期内 SHOULD 保持稳定。  
频繁修改路径规则会导致：

- 历史 diff 失真；
- Code Review 成本上升；
- 索引失配；
- Agent 引用失效。

### 13.2 字段稳定性

头字段的删除、改名和语义改写 MUST 经过版本升级或迁移计划。  
否则上游门禁和下游 Agent 读取器都会失效。

---

## 14. 统一引用规则

工件之间引用 SHOULD 优先使用 `artifact_id`，而不是自由文本标题。  
这样做的好处：

- 稳定；
- 可机读；
- 可跨语言；
- 可用于索引和图谱。

正文中 MAY 出现可读标题，但机器引用 SHOULD 以 `artifact_id` 为准。

---

## 15. 信息模型的最小闭环

一个符合 v1 的实现，至少要形成以下闭环：

1. `source_path`
2. `artifact_id`
3. `source_fingerprint`
4. `semantic_fingerprint`
5. `freshness_state`
6. `module_id`
7. `drift_report`

只要这些对象能稳定联通，系统就已经具备：

- 可追溯；
- 可增量更新；
- 可阻断漂移；
- 可为 Agent 提供入口

这四项核心能力。

---

## 16. 一句话结论

ANRSM 的信息模型不是“把代码旁边再放几篇 Markdown”，而是 **把源码、镜像、索引、漂移报告和知识层组织成一套有稳定路径、有固定字段、有双向追踪关系的工件系统**。
