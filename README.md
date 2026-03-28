# AI Native Repo Semantic Mirror (ANRSM) 开发指导文档集

[![gitcgr](https://gitcgr.com/badge/tb280320889/what-the-code-doing.svg)](https://gitcgr.com/tb280320889/what-the-code-doing)

状态：v1 设计母稿  
读者：首次接触 ANRSM 概念的架构师、资深工程师、基础设施工程师、AI Agent 平台工程师  
用途：作为立项、系统设计、原型开发、规范评审、CI 接入、Agent 工作流集成的统一开发依据

## 1. 这套文档集解决什么问题

当仓库开发、维护、排障和知识交接越来越依赖 AI Agent 时，传统“Agent 直接读源码”的模式会同时暴露四个基础性问题：

1. 全量源码上下文昂贵，Token 消耗高，且随着仓库增长非线性恶化。
2. Agent 很难在第一次读取时快速定位真正相关的文件、模块、边界和风险点。
3. 普通 README、注释和 Wiki 不是面向模型的语义表示，既不稳定，也不具备强同步机制。
4. 文档与代码一旦分离，就会出现双真相源，反而降低 Agent 与人类的决策质量。

ANRSM 的目标不是再做一套普通文档系统，而是定义一种 **AI Native 的仓库语义镜像基础设施**：

- 源码保持唯一真相源；
- 机器优先的语义镜像层为 Agent 提供低 Token、高密度、可同步、可审计的上下文入口；
- 人类优先的知识层从同一语义基础派生，服务于架构理解、交接、排障和学习；
- 改动检测、漂移分析和 CI 门禁确保镜像层不会脱离代码演化。

## 2. 文档集内容

### 核心文档

1. `docs/01_ANRSM_v1_Design_Mother_Draft.md`  
   v1 设计母稿。给出问题定义、目标、架构总览、核心不变量、v1 边界和实施原则。

2. `docs/02_Normative_Language_and_Glossary.md`  
   规范语言、术语体系、保留词、命名规则、状态枚举和文档/工件命名约束。

3. `docs/03_System_Boundaries_Assumptions_and_NonGoals.md`  
   明确系统边界、前提假设、适用条件、非目标和不在 v1 解决范围内的问题。

4. `docs/04_Architecture_and_Component_Model.md`  
   组件模型、控制面/数据面、语言适配器策略、初始化与增量更新流水线。

5. `docs/05_Information_Model_and_Repository_Layout.md`  
   仓库目录结构、工件映射规则、文件级/模块级/知识层工件模型。

6. `docs/06_Semantic_Mirror_Protocol_and_Header_Spec.md`  
   文件镜像协议、文档头字段、语义指纹、正文结构、压缩原则、生成规则。

7. `docs/07_Drift_Detection_Synchronization_and_CI.md`  
   漂移检测、Material Change 分类、Git Diff 使用规则、门禁策略和失败码。

8. `docs/08_Agent_Runtime_Contract_and_Workflows.md`  
   Agent 运行时契约、读取顺序、任务路由、按需展开源码、写回义务和失败回退。

9. `docs/09_Validation_Acceptance_Testing_and_Metrics.md`  
   验证维度、测试设计、指标、验收标准、发布门槛和质量审计建议。

10. `docs/10_Milestones_Roadmap_and_Delivery_Plan.md`  
    开发里程碑大纲、各阶段交付物、入口/出口条件、实施风险与依赖。

11. `docs/11_AntiPatterns_FailureModes_and_Decision_Record_Guide.md`  
    反模式、常见失败模式、纠偏措施，以及架构决策记录的建议模板。

### 机器可读参考资料

- `schemas/mirror-header.schema.json`  
  文件镜像文档头 JSON Schema。

- `schemas/drift-report.schema.json`  
  漂移报告 JSON Schema。

### 示例

- `examples/anrsm.example.yaml`  
  v1 配置样例。

- `examples/file_mirror_example.md`  
  文件级语义镜像样例。

- `examples/drift_report_example.json`  
  漂移报告样例。

## 3. 建议阅读顺序

### 面向架构负责人

1. 01 设计母稿  
2. 03 系统边界  
3. 04 架构与组件模型  
4. 10 里程碑与交付计划  
5. 09 验收标准

### 面向基础设施/平台工程师

1. 02 规范语言与术语  
2. 05 信息模型与目录布局  
3. 06 语义镜像协议  
4. 07 漂移检测与 CI  
5. 08 Agent 运行时契约

### 面向业务线工程师或集成方

1. 01 设计母稿  
2. 05 信息模型与目录布局  
3. 08 Agent 运行时契约  
4. 09 验收与测试

## 4. v1 的最小一致结论

这套文档集对 v1 的结论是明确的：

- **唯一真相源** 必须是源码与仓库受管配置。
- **文件级语义镜像** 是 v1 的强制能力，而不是可选增强。
- **模块级镜像** 是 v1 推荐能力，用于任务路由和上下文聚合。
- **人类知识层** 在目标架构中成立，但不要求成为 v1 MVP 的阻塞项。
- **任何 Material Change** 都必须可被检测为镜像漂移，且必须具备门禁策略。
- **Agent 默认读取顺序** 必须是“任务 -> 索引 -> 模块镜像 -> 文件镜像 -> 按需展开源码”，而不是“默认直接吞源码”。
- **Prompt 不得直接替代结构化提取**。LLM 可参与摘要和知识编译，但必须锚定到结构化事实与可追溯来源。
- **镜像文档不是普通 prose**。它是受约束的、低冗余的、可验证的语义压缩工件。

## 5. 适用对象与推荐场景

ANRSM 最适合以下仓库形态：

- 中小型到中大型跨端仓库；
- 日常开发大量依赖 Agent；
- 代码、配置、测试、脚本和接口契约耦合较强；
- 人员流动高，交接成本高；
- 需要同时优化 Agent 开发效率和人类维护理解成本；
- 希望把“文档同步”从软约束升级为硬门禁。

## 6. 不应误解为

ANRSM 不是：

- 普通的代码注释生成器；
- README 聚合器；
- 任意文本索引系统；
- 单纯的代码搜索引擎；
- 只给人看的知识库；
- 允许镜像脱离源码独立演化的文档系统；
- 替代编译、测试、审查和回归验证的万能机制。

## 7. 使用方式

这套文档可以直接作为以下工作的输入：

- 立项 PRD 和技术方案说明；
- 架构评审会议材料；
- 技术 Spike 与 PoC 实施清单；
- CLI/API 设计和数据契约设计；
- CI 门禁策略设计；
- Agent 开发工作流规范；
- 仓库治理规范。

## 8. 版本说明

本套文档为 **v1 设计母稿文档集**。  
它的角色是“统一概念、锁定 P0 规范、压缩实现歧义”，而不是提前固化所有实现细节。  
凡是涉及未来可扩展能力的内容，文档中会明确标记是否属于：

- `P0`：v1 必须落地；
- `P1`：v1 推荐但可后置；
- `P2`：未来增强方向，不是 v1 阻塞项。
