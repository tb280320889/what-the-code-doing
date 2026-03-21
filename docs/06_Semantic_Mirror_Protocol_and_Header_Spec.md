# 06. 语义镜像协议与文档头规范

状态：Normative  
版本：v1  
优先级：P0

---

## 1. 目的

本文件定义文件级语义镜像的协议，包括：

- 文档头字段；
- 正文结构；
- 语义压缩规则；
- 指纹定义要求；
- 人工补充边界；
- 生成与更新约束。

该协议是 ANRSM 的核心。  
如果没有稳定协议，所谓“镜像”就只是一组风格不一的说明文档，无法被 Agent、门禁和索引系统可靠消费。

---

## 2. 协议定位

文件级镜像协议是 **机器优先、文本承载、可审计的语义压缩协议**。  
它不是：

- prose 写作规范；
- 注释风格指南；
- 任意文档模板；
- README 约定。

一个文件只有在满足本协议时，才能被视为 `file_mirror`。

---

## 3. 文档头的规范格式

v1 推荐使用 YAML Front Matter。格式如下：

```markdown
---
anrsm_version: 1
artifact_type: file_mirror
artifact_id: file_mirror:src/auth/login.ts
source_path: src/auth/login.ts
source_language: typescript
module_id: auth
source_commit: 9f8c2ab
source_fingerprint: sha256:...
semantic_fingerprint: sha256:...
freshness_state: fresh
confidence_band: high
generator_name: anrsm
generator_version: 0.1.0
generated_at: 2026-03-20T10:30:00Z
exports:
  - LoginController
  - validateCaptcha
dependencies:
  - src/core/http/client.ts
  - src/core/auth/session.ts
risk_tags:
  - auth
  - side_effect
  - external_api
---
```

要求：

1. 文档头 MUST 出现在文件起始位置；
2. 字段名 MUST 使用 ASCII `snake_case`；
3. 所有必填字段 MUST 存在；
4. 文档头 MUST 可被无歧义解析；
5. 正文 MUST 紧随其后。

---

## 4. 必填字段说明

### 4.1 anrsm_version

类型：integer  
含义：镜像协议版本。  
要求：

- MUST 填写；
- v1 中固定为 `1`。

### 4.2 artifact_type

类型：string  
要求：

- MUST 为 `file_mirror`；
- 不得省略。

### 4.3 artifact_id

类型：string  
要求：

- MUST 唯一标识当前工件；
- MUST 采用规范格式；
- SHOULD 与 `source_path` 保持可推导关系。

### 4.4 source_path

类型：string  
含义：仓库相对路径。  
要求：

- MUST 指向真实源码文件；
- MUST 使用 POSIX 风格路径；
- MUST 与文件映射规则一致。

### 4.5 source_language

类型：string  
含义：主语言标识。  
示例：

- `typescript`
- `python`
- `go`
- `kotlin`

### 4.6 module_id

类型：string  
含义：该文件所属逻辑模块。  
要求：

- MUST 可由配置或规则推导；
- 对无法确定模块归属的文件，可使用 `global` 或显式保守值，但 SHOULD 减少这种情况。

### 4.7 source_commit

类型：string  
含义：本镜像基于哪个源码提交生成。  
要求：

- MUST 填写短 SHA 或全 SHA；
- SHOULD 能用于调试与回溯。

### 4.8 source_fingerprint

类型：string  
含义：原始源码内容指纹。  
要求：

- MUST 带算法前缀；
- MUST 在源码文本变化时变化。

### 4.9 semantic_fingerprint

类型：string  
含义：基于语义对象计算的指纹。  
要求：

- MUST 带算法前缀；
- SHOULD 对非语义噪声保持稳定；
- MUST 用作 Material Change 判断的重要依据。

### 4.10 freshness_state

类型：enum  
允许值：

- `fresh`
- `stale`
- `invalid`
- `unknown`

要求：

- MUST 存在；
- MUST 由受管流程写入，而不是自由手改。

### 4.11 confidence_band

类型：enum  
允许值：

- `high`
- `medium`
- `low`

要求：

- MUST 存在；
- 当解析器能力有限或摘要存在推断时，不得维持虚假的 `high`。

### 4.12 generator_name / generator_version

作用：记录生成器来源和版本。  
要求：

- MUST 同时存在；
- MUST 用于调试和迁移。

### 4.13 generated_at

类型：RFC 3339 时间戳。  
要求：

- MUST 存在；
- MUST 表示本镜像本次生成时间。

---

## 5. 推荐字段说明

以下字段建议在 v1 中支持：

- `exports`
- `dependencies`
- `risk_tags`
- `contracts`
- `adjacent_artifacts`
- `notes_locked`
- `manual_appendix_present`

其中：

- `exports` 用于快速了解外部可见能力；
- `dependencies` 用于快速路由与影响分析；
- `risk_tags` 用于提醒鉴权、事务、缓存、并发、外部 I/O 等风险域；
- `manual_appendix_present` 用于声明是否存在人工补充段。

---

## 6. 正文协议

### 6.1 正文的职责

正文不是对源码的逐行解释。  
正文的职责是把完成工程判断所需的关键语义压缩到最小充分上下文中。

### 6.2 正文的标准结构

v1 推荐严格采用以下结构：

```markdown
## 职责
## 对外契约
## 输入与前置条件
## 状态与控制流
## 副作用与外部依赖
## 关键不变量与风险
## 变更影响
## 何时必须展开源码
```

要求：

- 段落标题 SHOULD 保持完全一致；
- 标题顺序 SHOULD 稳定；
- 段落缺失 MUST 有明确理由；
- 不应为每个文件自由发明新结构。

### 6.3 各段落应回答的问题

#### 职责

这个文件在系统中的唯一职责是什么。  
不应堆砌历史背景和无关上下文。

#### 对外契约

它向外暴露什么：函数、类、类型、路由、事件、命令、配置键等。  
这里只写外部依赖它时需要知道的事实。

#### 输入与前置条件

调用前必须满足什么条件，依赖什么环境、配置、状态或数据形态。

#### 状态与控制流

它如何推进主要流程；有哪些关键分支、状态转换、失败路径、回滚点。

#### 副作用与外部依赖

它会读写什么：数据库、网络、缓存、日志、队列、本地文件、系统时钟等。

#### 关键不变量与风险

什么约束不能被破坏；什么地方最容易引入错误；是否涉及鉴权、幂等性、事务、并发、一致性等。

#### 变更影响

修改本文件最可能影响哪些邻域、模块、契约或测试。

#### 何时必须展开源码

在哪些场景下，仅凭镜像不足以做正确决策，需要阅读源码、测试或调用方。

---

## 7. 语义压缩规则

### 7.1 压缩目标

目标不是尽可能短，而是 **在最小 Token 内保留决策关键语义**。

### 7.2 必须保留的信息

镜像正文 SHOULD 尽量保留：

- 责任边界；
- 对外接口；
- 状态变化点；
- 关键副作用；
- 关键异常路径；
- 不变量；
- 风险点；
- 高影响依赖；
- 修改影响面。

### 7.3 应删除的信息

正文 SHOULD 删除或显著压缩：

- 逐行实现细节；
- 重复的参数名列表；
- 可从签名直接读出的冗余描述；
- 与当前任务路由无关的修辞性背景；
- 纯样板逻辑描述。

### 7.4 不得做的事

正文 MUST NOT：

1. 粘贴大段源码；
2. 把实现细节全部翻译成自然语言；
3. 用一堆形容词替代事实；
4. 把未知当已知；
5. 隐藏关键副作用。

---

## 8. 语义指纹规则

### 8.1 源码指纹与语义指纹的区别

- `source_fingerprint` 反映输入文本是否变化；
- `semantic_fingerprint` 反映镜像有效性相关的语义是否变化。

### 8.2 语义指纹的推荐构成

语义指纹 SHOULD 主要由以下对象归一后计算：

1. 导出符号与签名；
2. 关键输入输出结构；
3. 路由/事件/命令名；
4. 关键依赖边；
5. 关键状态对象与状态转移；
6. 关键副作用类型；
7. 关键风险标签；
8. 关键控制流锚点。

### 8.3 应排除的噪声

语义指纹 SHOULD 尽量排除：

- 空白；
- 注释；
- import 排序；
- 无影响的局部变量重命名；
- 纯格式化；
- 无影响的语法糖替换。

### 8.4 升级原则

指纹算法升级 MUST：

- 记录版本；
- 提供迁移说明；
- 允许全量重建；
- 不得静默改变语义含义。

---

## 9. 新鲜度状态机

### 9.1 状态定义

- `fresh`：当前镜像与当前语义状态一致；
- `stale`：检测到潜在或确定漂移，尚未完成更新与验证；
- `invalid`：文档头缺失、字段错误、解析失败、校验失败或结构破损；
- `unknown`：系统无法确定是否仍然有效。

### 9.2 状态转移建议

```text
fresh -> stale     当源码或语义发生相关变化
stale -> fresh     当镜像更新并通过验证
any   -> invalid   当文档损坏或生成失败
any   -> unknown   当系统缺少足够信息判断
unknown -> fresh   当补齐信息并通过验证
invalid -> fresh   当修复后重新生成并验证
```

### 9.3 策略建议

- `fresh` 可作为默认高可信输入；
- `stale` 不应作为唯一决策依据；
- `invalid` 在阻塞策略下 SHOULD 直接失败；
- `unknown` 在主干和发布策略下 SHOULD 被保守对待。

---

## 10. 人工补充边界

### 10.1 为什么允许人工补充

有些信息难以从静态结构化提取中获得，例如：

- 历史坑位；
- 线上事故教训；
- 业务约束来源；
- 外部系统不可见约束。

因此，系统 MAY 允许受限人工补充区块。

### 10.2 人工补充的边界

人工补充 MUST：

1. 放在明确标记的区块；
2. 不得篡改机器头字段；
3. 不得改写机器受管段落的事实含义；
4. 应说明责任人和更新时间。

推荐格式：

```markdown
## 人工补充
- 线上约束：...
- 已知坑：...
- 复核建议：...
```

### 10.3 人工补充不应解决的问题

人工补充 SHOULD NOT 被用来：

- 掩盖镜像本身结构缺失；
- 替代受管更新流程；
- 直接修正源码事实而不改源码。

---

## 11. 生成规则

### 11.1 生成顺序

Mirror Generator SHOULD 按如下顺序工作：

1. 装载结构化事实；
2. 装载旧镜像以保持稳定性；
3. 计算新指纹；
4. 填充头字段；
5. 生成标准段落；
6. 合并人工补充区块；
7. 写回文件；
8. 重新校验解析性。

### 11.2 稳定更新原则

生成器 SHOULD 尽量保持：

- 段落顺序不变；
- 术语不变；
- 已知事实表达方式不随意改写；
- 只有在事实变化时才改对应段落。

这样做的价值是：

- diff 更干净；
- Reviewer 更容易定位真实变化；
- Agent 更容易缓存和增量读取。

---

## 12. 镜像质量的最低要求

一个合格的文件镜像至少必须满足：

1. 文档头完整且可解析；
2. 文本与 `source_path` 一一对应；
3. `freshness_state` 可解释；
4. 正文包含关键责任与副作用；
5. 当需要展开源码时，正文明确指出触发条件；
6. 不存在大段无效 prose；
7. 不隐瞒关键风险。

---

## 13. 常见坏味道

以下情况说明镜像协议执行失败：

1. 正文看起来像 README，而不是文件镜像；
2. 标题命名每个文件都不一样；
3. 几乎没有副作用和风险描述；
4. 完全没有“何时必须展开源码”段；
5. `semantic_fingerprint` 每次格式化都会变化；
6. 开发者开始直接手改机器头。

---

## 14. 示例

### 14.1 规范示意

```markdown
---
anrsm_version: 1
artifact_type: file_mirror
artifact_id: file_mirror:src/auth/login.ts
source_path: src/auth/login.ts
source_language: typescript
module_id: auth
source_commit: 9f8c2ab
source_fingerprint: sha256:9e...
semantic_fingerprint: sha256:7a...
freshness_state: fresh
confidence_band: high
generator_name: anrsm
generator_version: 0.1.0
generated_at: 2026-03-20T10:30:00Z
exports:
  - login
dependencies:
  - src/core/http/client.ts
  - src/core/auth/session.ts
risk_tags:
  - auth
  - external_api
---

## 职责
处理账号登录主流程，负责参数校验、验证码检查、令牌请求和会话建立。

## 对外契约
对外暴露 `login(credentials)`，成功时返回会话对象，失败时抛出鉴权或验证错误。

## 输入与前置条件
要求调用方已提供标准化凭证对象；当风控开关开启时，必须携带验证码令牌。

## 状态与控制流
流程为参数校验 -> 验证码检查 -> 请求鉴权服务 -> 建立本地会话 -> 返回会话；鉴权失败时不会落盘会话。

## 副作用与外部依赖
依赖 HTTP Client 调用鉴权服务；写入会话存储；记录失败日志。

## 关键不变量与风险
不得在鉴权失败时创建会话；验证码校验与主登录调用必须共享同一风控上下文。

## 变更影响
会影响登录页、会话恢复逻辑、鉴权错误展示和相关契约测试。

## 何时必须展开源码
当修改错误分支、重试策略、风控上下文或会话持久化实现时，必须展开源码与调用方。
```

---

## 15. 一句话结论

文件级镜像协议的本质，是 **把“代码旁边的一篇说明”升级成“有标准头、有新鲜度、有指纹、有正文骨架、有生成与验证规则的语义工件”**。
