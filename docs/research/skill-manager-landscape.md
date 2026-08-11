# 开发者自用 Skill Manager / Agent Skill Management 系统调研

> 调研日期：2026-08-11。本文只把官方文档、官方源码仓库、公开规范和厂商一方资料作为事实来源。  
> 标记约定：**事实** = 一手资料明确支持；**推断** = 基于多个事实的产品判断；**未公开** = 官方资料不足，不以“没有”冒充事实。

## 0. 结论先行

1. **“Skill Manager”已经从文件复制工具分化成三类产品**：跨 Agent 的本地包管理器（`gh skill`、Vercel `skills`、OpenSkill/OpenSkills）、Agent 内置扩展管理器（Claude Code、Codex/ChatGPT、Cursor、Kiro、OpenClaw），以及生产级 Skill 控制面（LangSmith Context Hub、SkillHub、Claude Skills API）。它们解决的是同一生命周期的不同切片。
2. **Agent Skills 的事实标准正在收敛到“目录 + `SKILL.md` + 渐进式披露”**。开放规范只规定可移植的内容格式，不规定来源、版本、安装所有权、依赖求解、权限、评价、更新事务或市场治理；这些恰好是 Skill Manager 的产品空间。[Agent Skills Specification](https://agentskills.io/specification)
3. **截至本次调研，最接近“个人开发者通用 Skill Manager”基线的是 GitHub CLI 的 `gh skill`**：搜索、预览、跨 host 安装、tag/SHA pin、来源与 tree SHA provenance、更新、发布均已具备，但仍是 preview，缺少运行期权限、使用效果评价和个性化推荐。[`gh skill`](https://cli.github.com/manual/gh_skill)
4. **最完整的开源“注册表 + 运行时 + 治理”案例是 OpenClaw + ClawHub**：版本/tag、pin、更新、向量搜索、moderation、扫描、每 Agent allowlist，以及实验性 Skill Workshop 自动生成形成闭环；代价是与 OpenClaw 运行时耦合较深。[OpenClaw skills](https://github.com/openclaw/openclaw/blob/main/docs/tools/skills.md)、[ClawHub](https://github.com/openclaw/clawhub)
5. **最大空白不是再做一个下载器，而是“可证明的效果与安全闭环”**：安装前 capability/provenance 审计，安装后真实触发与结果观测，with-skill vs baseline 评价，共存回归，基于失败自动提出改进但不静默覆盖。Anthropic 已公开五维评价框架，却明确指出 Skills API 没有 usage analytics；阿里 `skill-up` 开始补 eval/evolution，但尚不是完整 manager。[Anthropic enterprise skills](https://platform.claude.com/docs/en/agents-and-tools/agent-skills/enterprise)、[Alibaba skill-up](https://github.com/alibaba/skill-up)
6. **个人产品的最佳切入点是 local-first control plane，而不是先建市场**：用一个本地 registry 统一发现和对账多个 Agent 目录，记录来源/版本/权限/内容指纹，提供事务式安装与回滚，再加最小 eval。搜索市场可直接聚合 GitHub、skills.sh 或 ClawHub，没必要在 MVP 自建供给侧。

---

## 1. 定义与边界

### 1.1 什么是开发者自用 Skill Manager

本文把它定义为：

> 面向单个开发者或小团队，对 Agent Skill 的**发现、获取、验证、安装、暴露、调用策略、版本、依赖、权限、观测、评价、演化、分享和退役**提供统一控制面的软件。

Skill 本身是可复用能力包，而 Manager 管的是它的生命周期。开放规范把 Skill 定义为至少包含一个 `SKILL.md` 的目录；必需元数据是 `name` 和 `description`，并允许 `scripts/`、`references/`、`assets/` 等内容。[Agent Skills Specification](https://agentskills.io/specification) OpenAI 的当前定义同样是“指令、资源和可选脚本”的目录，并明确采用渐进式披露：先给模型 name/description，命中后才加载全文。[OpenAI Build skills](https://learn.chatgpt.com/docs/build-skills)

一个真正的 Manager 至少应回答五个问题：

- 机器上有哪些 Skill，来自哪里，哪些 Agent 正在看到它？
- 当前运行的究竟是哪一版，内容是否被本地修改，能否安全更新/回滚？
- Skill 会读写什么、执行什么、需要哪些工具、命令、网络与凭据？
- 它什么时候触发，触发是否准确，是否真的提高成功率或只是增加 token/风险？
- 谁可以发布、批准、安装、启用、修改和退役它？

### 1.2 与相邻概念的区别

| 概念 | 管理对象 | 典型运行时语义 | 与 Skill Manager 的边界 |
|---|---|---|---|
| **MCP Server** | 远程/本地的 tools、resources、prompts | MCP tool 是模型可调用的结构化函数；resource 是上下文数据；prompt 是用户选择的模板。[MCP server primitives](https://modelcontextprotocol.io/specification/2025-06-18/server/index) | MCP 提供“能调用什么”；Skill 提供“何时、为何、按什么步骤组合知识和工具”。Skill 可依赖 MCP，但 Manager 不必自己成为 MCP server。 |
| **Agent Framework** | 模型、agent loop、工具、中间件、状态、编排 | 负责规划—行动—观察循环和应用运行。Microsoft Agent Framework 的 Skill provider只是 framework 的一个 context provider，并提供 load/read/run 工具。[Microsoft Agent Framework Skills](https://learn.microsoft.com/en-us/agent-framework/agents/skills) | Framework 运行 Agent；Manager 管可移植能力资产。二者可集成，但不应混为同一层。 |
| **Plugin System** | 多种扩展组件的分发包 | 一个 plugin 可同时打包 Skills、MCP、hooks、agents、rules、UI。Claude、OpenAI、Cursor 均采用此模式。[Claude plugins](https://code.claude.com/docs/en/plugins-reference)、[OpenAI plugins](https://learn.chatgpt.com/docs/plugins)、[Cursor plugins](https://cursor.com/blog/marketplace) | Plugin 是更粗粒度的发行/隔离单元；Skill 是其中一种可移植能力。Skill Manager 可只管理 standalone Skill，也可把 plugin 当来源。 |
| **Tool Registry** | 可执行工具或 MCP server 的可发现目录 | 目录记录连接、schema、transport、认证等，使 host 找到可调用服务。[MCP Registry](https://registry.modelcontextprotocol.io/docs) | Registry 告诉 Agent 哪里有工具；Skill Manager 还管理指令、资源、触发、版本、效果和跨 host 安装。 |
| **Prompt Library** | Prompt/template 及其变量、模型配置 | 重点是 push/pull、版本、tag、环境和模板复用。LangSmith Prompt Hub 支持 commit、staging/production tag、权限与公共 Hub。[LangSmith Manage prompts](https://docs.langchain.com/langsmith/manage-prompts) | Prompt 通常是一次输入模板；Skill 是有目录边界的能力包，可含脚本/资源并由 Agent 动态发现。Skill Manager 的生命周期更像软件包管理。 |
| **Workflow Engine** | 确定或半确定的步骤图、状态和重试 | 强调步骤顺序、持久状态、重试、补偿和长任务。 | Skill 可以“描述”工作流，但执行是否严格、持久、可恢复取决于 agent/runtime；需要确定性和 durable execution 时仍应交给 workflow engine。 |

**推断：** Skill Manager 位于“内容资产管理、包管理、Agent runtime 适配、质量工程”四者交叉处。它不是上述任一系统的改名，而是把它们之间目前靠手工维护的缝隙收拢起来。

---

## 2. 市场地图

### 2.1 分类

- **开放格式/基础设施**：Agent Skills Specification、Microsoft Agent Framework Skill Provider、NVIDIA Verified Skills Catalog/Trust Pipeline。
- **跨 Agent 本地包管理器**：GitHub CLI `gh skill`、Vercel `skills`、OpenSkill、OpenSkills。
- **开源 registry / runtime 闭环**：OpenClaw + ClawHub、iFlytek SkillHub。
- **评价与演化专用工具**：Alibaba `skill-up`，不是完整 Manager，但补齐关键缺口。
- **商业闭源内置管理器**：Claude Code Plugin Marketplace、OpenAI universal plugin directory、Cursor Marketplace、Kiro Powers、LangSmith Context Hub、GitHub Copilot/VS Code Customizations。

以下“支持”状态使用：✅ 明确支持；◐ 部分/依赖 host/仅相邻能力；❌ 官方资料明确无此能力或产品范围不包含；? 未公开或本次一手资料无法确认。

### 2.2 项目逐项分析

#### A. GitHub CLI `gh skill`

- **链接**：[官方手册](https://cli.github.com/manual/gh_skill)、[install](https://cli.github.com/manual/gh_skill_install)、[update](https://cli.github.com/manual/gh_skill_update)、[publish](https://cli.github.com/manual/gh_skill_publish)
- **开源/闭源**：GitHub CLI 开源；GitHub 搜索/Release 平台为商业服务。Skill 命令当前是 public preview。
- **目标用户 / 定位**：已使用 GitHub 的开发者；把 GitHub repository/release 直接变成跨 Agent Skill 包源。
- **Skill 定义 / 存储**：Agent Skills 目录；项目或用户级 host 目录。安装时向 frontmatter 注入 source、版本和 tree SHA provenance。
- **生命周期**：search → preview → install → list → update；publish 会校验规范并创建 GitHub Release。可用 tag/commit SHA pin；未指定版本时按最新 release、否则 default branch HEAD 解析。
- **调用机制**：不运行 Skill；写入 Copilot、Claude Code、Cursor、Codex、Gemini CLI 等 host 目录，由 host 自动或显式调用。
- **能力**：自动发现 ✅；安装/卸载 ◐（install/list/update 已公开，`gh skill` 顶层手册未列 remove）；版本 ✅；权限 ◐（继承 GitHub repo 权限，不管运行权限）；依赖 ❌；组合 ◐（`--all` 批量安装，不是依赖组合）；推荐 ◐（search，不是个性化推荐）；评估 ◐（publish validation，不评效果）；共享 ✅（repo/release）；市场 ◐（GitHub 搜索而非独立审核市场）。
- **最大特点 / 不足**：来源与 tree SHA 可追溯、跨 host 面广；但 runtime 权限、效果评价、使用观测和事务回滚不在范围内。

#### B. Vercel `skills` CLI + skills.sh

- **链接**：[官方源码/README](https://github.com/vercel-labs/skills)、[CLI 文档](https://www.skills.sh/docs/cli)、[目录与排行榜](https://www.skills.sh/docs)
- **开源/闭源**：CLI 开源；skills.sh 托管目录/排行榜为 Vercel 服务，后端开放程度未公开。
- **目标用户 / 定位**：希望一条命令把 GitHub Skill 安装到多种 coding agent 的开发者；“npm-like”轻量分发。
- **Skill 定义 / 存储**：`SKILL.md` 目录；默认 canonical `.agents/skills`，通过 symlink/junction 或 copy 暴露给各 Agent。来源和 folder hash 存在 lock 文件。
- **生命周期**：`add/list/find/check/update/remove/init/use`；支持 repo 子路径、global/project、多 Agent、copy/symlink。[README](https://github.com/vercel-labs/skills/blob/main/README.md)
- **调用机制**：安装到 Agent 原生目录；`use` 可在隔离临时目录运行指定 agent，但最终调用语义仍由 host 决定。
- **能力**：自动发现 ✅；安装/卸载 ✅；版本 ◐（hash/update，有来源锁；不是通用 semver resolver）；权限 ❌；依赖 ❌；组合 ✅（一仓多 Skill、多 Agent）；推荐 ✅（find + 基于匿名安装遥测的排行榜）；评估 ❌；共享 ✅（Git repo）；市场 ✅（skills.sh）。
- **最大特点 / 不足**：最低摩擦、生态覆盖广；安全页明确不能保证每个 Skill 的质量或安全，排行榜主要反映安装量，不等于任务效果。[skills.sh security/ranking](https://www.skills.sh/docs)

#### C. OpenSkills

- **链接**：[官方仓库](https://github.com/numman-ali/openskills)
- **开源/闭源**：Apache-2.0 开源。
- **目标用户 / 定位**：需要把 Agent Skills 格式用于没有原生加载能力的 coding agent 的开发者；通用 loader。
- **Skill 定义 / 存储**：Agent Skills 风格目录，安装到 `.claude/skills` 等本地目录，并可向 `AGENTS.md` 同步可发现目录。
- **生命周期**：install、list、read、sync、update；Git 来源可记录和刷新。
- **调用机制**：Agent 通过 CLI `openskills read <name>` 按需读取，或使用同步到 `AGENTS.md` 的说明。
- **能力**：自动发现 ✅；安装/卸载 ◐；版本 ◐（来源更新，不是 registry version）；权限 ❌；依赖 ❌；组合 ◐（多 Skill read/sync）；推荐 ❌；评估 ❌；共享 ✅（Git）；市场 ❌。
- **最大特点 / 不足**：静态文件 + CLI，无 MCP server；简单、可移植，但治理、质量、权限和 marketplace 都很薄。

#### D. OpenSkill (`osk`)

- **链接**：[官方文档](https://www.openskill.sh/docs/getting-started/introduction)
- **开源/闭源**：MIT CLI 开源；OpenSkill marketplace/团队治理是托管商业服务，服务端源码开放程度未公开。[Pricing](https://www.openskill.sh/pricing)
- **目标用户 / 定位**：个人开发者以及需要 private registry、audit policy、RBAC 的团队；agent-agnostic Git-based Skill package manager + hosted governance。
- **Skill 定义 / 存储**：带 YAML frontmatter 的 Markdown Skill；GitHub/GitLab/self-hosted Git 为来源，转换后落到各 Agent 目录。
- **生命周期**：search、install、list、update、remove、publish；API 公开 discover、versions、download、security audit 和 telemetry endpoints。官方把它直接定义为“package manager for AI agent skills”。[API](https://www.openskill.sh/docs/api)
- **调用机制**：写到 host 目录，由 host 调用。
- **能力**：自动发现 ✅；安装/卸载 ✅；版本 ✅（registry versions + Git source）；权限 ✅（付费团队的 private registry、allow/block policy、RBAC）；依赖 ?；组合 ◐；推荐 ◐（search/browse/tag）；评估 ◐（45-rule security audit，不是任务效果 eval）；共享 ✅；市场 ✅。
- **最大特点 / 不足**：从个人免费 CLI 平滑升级到团队治理，并公开安全 audit；但安全评分不是 task effectiveness，公开资料也未显示事务式多 host rollback 或共存 eval。

#### E. OpenClaw + ClawHub

- **链接**：[OpenClaw Skills](https://github.com/openclaw/openclaw/blob/main/docs/tools/skills.md)、[ClawHub 仓库](https://github.com/openclaw/clawhub)、[CLI](https://github.com/openclaw/clawhub/blob/main/docs/cli.md)
- **开源/闭源**：开源。
- **目标用户 / 定位**：OpenClaw 个人 Agent 用户与 Skill/Plugin 发布者；runtime 内置管理 + 公共 registry。
- **Skill 定义 / 存储**：`SKILL.md` 与支持文件；workspace、managed `~/.openclaw/skills`、personal `~/.agents/skills` 等多层 root。ClawHub 保存版本、tag、changelog、文件和统计。
- **生命周期**：discover/search/inspect/install/list/pin/update/uninstall/publish/sync/rename/merge/soft-delete；Git/local 来源也能安装，但 OpenClaw 文档明确 Git/local 不参加 ClawHub update。
- **调用机制**：runtime 按优先级发现、构建 prompt catalog，可限制每 Agent 最终 allowlist；插件 Skill 合入同一加载体系。Skill Workshop 可从观察到的流程生成/更新 workspace Skill，先扫描并支持 pending approval/quarantine。
- **能力**：自动发现 ✅；安装/卸载 ✅；版本 ✅；权限 ✅（agent allowlist、registry RBAC/moderation）；依赖 ◐（env/bin/config gating 与 plugin package metadata，不是通用 Skill DAG）；组合 ✅（plugin/bundle/per-agent sets）；推荐 ◐（向量搜索、stars，无个性化证据）；评估 ◐（安全扫描，不是任务效果 eval）；共享 ✅；市场 ✅。
- **最大特点 / 不足**：开源系统里闭环最完整；但多数能力围绕 OpenClaw 语义，跨 host 管理较弱。第三方 Skill 仍被官方要求视作不可信代码。

#### F. iFlytek SkillHub

- **链接**：[官方仓库](https://github.com/iflytek/skillhub)、[CLI 指南](https://github.com/iflytek/skillhub/blob/main/docs/skillhub/en/guide/cli.md)
- **开源/闭源**：开源，可私有化部署。
- **目标用户 / 定位**：需要自建企业 Skill registry 的团队；强调 namespace、版本、RBAC、审计、安全扫描和本地/云部署。
- **Skill 定义 / 存储**：Skill package/zip，registry 侧 PostgreSQL/对象存储；客户端在 Agent 目录写 `.skillhub/metadata.json`，本地 `~/.skillhub/inventory.json` 对账。
- **生命周期**：login/search/install/list/remove/doctor/publish；支持 namespace、public/namespace-only/private、显式 version。注意官方 CLI 的 `update` 当前描述为 **CLI 自更新**，不能据此声称已支持已安装 Skill 自动升级。
- **调用机制**：安装到 Codex 等 Agent 目录，或被 astron-agent/相关产品引用加载。
- **能力**：自动发现 ✅；安装/卸载 ✅；版本 ✅（发布/安装 version；本地升级流程未完整公开）；权限 ✅（RBAC/namespace/visibility/audit）；依赖 ?；组合 ◐；推荐 ◐（search）；评估 ◐（安全扫描）；共享 ✅；市场 ✅（自建 registry）。
- **最大特点 / 不足**：企业治理和自托管最强；个人开发者部署栈较重，跨 Agent 路径适配与本地 lifecycle 仍在演进。

#### G. Alibaba `skill-up`（评价/演化补充件）

- **链接**：[官方仓库](https://github.com/alibaba/skill-up)
- **开源/闭源**：开源。
- **目标用户 / 定位**：Skill 作者和 CI；不是安装器，而是 Agent Skills 的 evaluation/evolution 工具。
- **Skill 定义 / 存储**：现有 Skill + `evals/eval.yaml` 与 case YAML；结果为 JSON、JUnit、HTML、Anthropic-compatible benchmark/grading reports。
- **生命周期**：创建 eval → 多 engine 运行 → rule/script/agent judge → 诊断失败 → `skill-upper` 协助修改 Skill/用例 → 回归。
- **调用机制**：调用 Claude Code、Codex、Qoder/Qwen 或 custom engine 运行真实任务。
- **能力**：自动发现 ◐；安装/卸载 ❌；版本 ❌；权限 ❌；依赖 ❌；组合/共存 ◐（可通过 suites 建模）；推荐 ❌；评估 ✅；共享 ✅（eval 与 Skill 同仓）；市场 ❌。
- **最大特点 / 不足**：目前少数直接度量 Skill 边际效果并推动演化的开源工具；尚未与 registry、usage telemetry、发布门禁形成默认闭环。

#### H. Claude Code Skills + Plugins Marketplace

- **链接**：[Skills](https://code.claude.com/docs/en/skills)、[Plugins](https://code.claude.com/docs/en/plugins-reference)、[Marketplace](https://code.claude.com/docs/en/discover-plugins)、[Enterprise guidance](https://platform.claude.com/docs/en/agents-and-tools/agent-skills/enterprise)
- **开源/闭源**：产品闭源；Agent Skills 规范和部分 Skills 仓库开放。
- **目标用户 / 定位**：Claude Code 个人、项目和企业用户；standalone Skill 用于个人/项目工作流，plugin 用于可版本化分发。
- **Skill 定义 / 存储**：`SKILL.md`，personal/project/enterprise/plugin 多 scope；plugin 是自包含目录并复制到本地 cache。Marketplace 用 `marketplace.json` 管 catalog/source/version。
- **生命周期**：自动发现和变更监听；plugin discover/install/enable/disable/uninstall/reload；marketplace add/update/remove 和自动更新；依赖插件可自动安装。
- **调用机制**：slash command 显式调用或按 description 自动调用；plugin Skill 带 namespace；可在当前对话或隔离 subagent context 中运行。
- **能力**：自动发现 ✅；安装/卸载 ✅；版本 ✅（plugin version/git commit/auto-update）；权限 ✅（managed scope、allowed tools/visibility、组织治理）；依赖 ✅（plugin dependencies）；组合 ✅；推荐 ◐（curated marketplace/search）；评估 ◐（官方方法论成熟，但非内置一键 gate）；共享 ✅；市场 ✅。
- **最大特点 / 不足**：从个人目录到 enterprise governance 的层次最完整；但 surface 分裂明显，Anthropic 明确说明 API、claude.ai、Claude Code 的 custom Skill 不会自动同步，需自行同步。Skills API 也未提供 usage analytics。[Enterprise guide](https://platform.claude.com/docs/en/agents-and-tools/agent-skills/enterprise)

#### I. OpenAI ChatGPT/Codex Skills + Universal Plugin Directory

- **链接**：[Build skills](https://learn.chatgpt.com/docs/build-skills)、[Plugins](https://learn.chatgpt.com/docs/plugins)
- **开源/闭源**：ChatGPT/Codex 产品与目录服务闭源；Codex CLI 有开源代码，Skill 建立在开放 Agent Skills 标准上。
- **目标用户 / 定位**：ChatGPT/Codex 用户与插件开发者；Skill 是 workflow authoring format，plugin 是跨 ChatGPT/Codex 的分发单元。
- **Skill 定义 / 存储**：`SKILL.md` 目录，repo/user/admin/system scopes；可选 `agents/openai.yaml` 声明 UI、调用策略和 MCP tool dependencies。Plugin 可打包 Skills、connectors、MCP、hooks、scheduled task templates。
- **生命周期**：Codex 自动发现本地 Skill，可用 `$skill-creator` 生成、`$skill-installer` 安装 curated/GitHub Skill，config 禁用；universal plugin directory 提供浏览、安装、已安装管理和 workspace/personal 分类。
- **调用机制**：ChatGPT 用 `@`、Codex 用 `/skills`/`$` 显式调用，也可按 description 隐式匹配。Codex 初始 Skill 列表有 2% context/8,000 字符预算，过多时会截断描述或省略 Skill。
- **能力**：自动发现 ✅；安装/卸载 ✅（plugin；direct Skill 的卸载/更新协议较弱）；版本 ◐（plugin marketplace 有发行层，direct Skill 无标准版本）；权限 ✅（connector scope/approval、workspace controls、sandbox；Skill 自身仅声明 allowed tools/依赖）；依赖 ✅（MCP tool dependencies）；组合 ✅（plugin）；推荐 ◐（目录浏览/search，个性化未公开）；评估 ◐（creator/生态工具，不是统一 gate）；共享 ✅；市场 ✅。
- **最大特点 / 不足**：同一公共插件目录覆盖 ChatGPT 与 Codex，多组件分发强；但本地 direct Skill 与 plugin 生命周期不同，IDE extension 目前不支持 plugin，跨 surface 行为仍不完全一致。

#### J. GitHub Copilot / VS Code Agent Customizations

- **链接**：[VS Code Agent Skills](https://code.visualstudio.com/docs/agent-customization/agent-skills)、[GitHub About Agent Skills](https://docs.github.com/en/copilot/concepts/agents/about-agent-skills)
- **开源/闭源**：VS Code 开源核心 + GitHub Copilot 商业服务；`gh skill` 是公开 CLI 管理面。
- **目标用户 / 定位**：在 IDE、CLI、cloud agent 和 code review 间复用 Skill 的 GitHub 开发者。
- **Skill 定义 / 存储**：开放 `SKILL.md`；project `.github/skills`/`.claude/skills`/`.agents/skills`，personal `~/.copilot/skills`/`~/.agents/skills`。VS Code 可配置额外发现位置。
- **生命周期**：Agent Customizations editor 可发现、创建、管理；`/create-skill` 能从描述或当前对话生成 Skill；共享内容目前官方流程仍包括复制目录或 `gh skill` 安装。
- **调用机制**：slash command 或相关性自动加载；可用 `context: fork` 在专用 subagent 中运行并只返回结果。
- **能力**：自动发现 ✅；安装/卸载 ✅（结合 `gh skill`）；版本 ✅（结合 `gh skill`）；权限 ◐（terminal/tool policy 与组织设置，非 Skill capability manifest）；依赖 ❌；组合 ✅（可由 Agent 同时使用多个 Skill）；推荐 ◐；评估 ❌；共享 ✅；市场 ◐（awesome-copilot/GitHub search，不是单一审核市场）。
- **最大特点 / 不足**：IDE 内 authoring/management 与 GitHub CLI supply chain 结合紧密；但使用效果和权限仍跨 Copilot、VS Code 和 GitHub 设置分散。

#### K. Cursor Plugins Marketplace

- **链接**：[官方发布说明](https://cursor.com/blog/marketplace)、[Marketplace](https://cursor.com/marketplace)、[Changelog](https://cursor.com/changelog/2-5)
- **开源/闭源**：闭源商业产品。
- **目标用户 / 定位**：Cursor 用户和工具厂商；一键安装包含 Skill、subagent、MCP、hook、rule 的插件。
- **Skill 定义 / 存储**：plugin bundle 内的 domain prompts/code；公开市场条目可展示 Skill/MCP/command 数量，source 常可跳转 GitHub。
- **生命周期**：网页或 IDE `/add-plugin` 发现/安装；Teams/Enterprise 支持 team marketplace 分发和管理 first-party plugin install behavior。[Team Marketplace Updates](https://cursor.com/changelog/05-01-26)
- **调用机制**：Agent 自动发现和运行 Skill；其他组件按 Cursor runtime 语义工作。
- **能力**：自动发现 ✅；安装/卸载 ✅；版本 ?；权限 ✅（sandbox/network controls、team governance）；依赖/组合 ✅（bundle 多组件）；推荐 ◐（curated marketplace）；评估 ?；共享 ✅；市场 ✅。
- **最大特点 / 不足**：供应商插件生态和组件融合强；公开资料不足以确认独立 Skill 的版本 pin、来源锁、eval 和跨 Agent portability。

#### L. AWS Kiro Powers / Agent Skills

- **链接**：[Powers](https://kiro.dev/docs/powers/)、[Agent Skills](https://kiro.dev/docs/skills/)
- **开源/闭源**：Kiro 产品闭源；支持开放 Agent Skills 和 GitHub 分发。
- **目标用户 / 定位**：Kiro IDE 用户；Skill 处理可移植工作流，Power 把 MCP tools、`POWER.md` steering 和 hooks 组合成按需激活包。
- **Skill 定义 / 存储**：`.kiro/skills` 或 global Skill；Power 可从 curated marketplace 或 GitHub 安装。
- **生命周期**：IDE panel 查看/管理 Skill；Power 支持 browse、一键安装、创建、GitHub 分享。
- **调用机制**：根据 description/keyword 自动激活，也可 slash command；Power 动态启停相关 MCP 上下文。
- **能力**：自动发现 ✅；安装/卸载 ✅；版本 ?；权限 ◐（MCP/tool trust 依赖 Kiro controls）；依赖/组合 ✅（Power bundle，可 stack）；推荐 ◐（curated partners/community）；评估 ❌；共享 ✅；市场 ✅。
- **最大特点 / 不足**：把“工具发现 + 操作知识”合成惰性加载单元，直接处理 MCP context overload；但 Power 是 Kiro 专属格式，且当前官方资料称 Power 仅 IDE 可用。

#### M. LangSmith Context Hub

- **链接**：[Context Hub](https://docs.langchain.com/langsmith/use-the-context-hub)、[SDK](https://docs.langchain.com/langsmith/manage-contexts-sdk)、[Concepts](https://docs.langchain.com/langsmith/context-engineering-concepts)
- **开源/闭源**：LangSmith 商业闭源服务；客户端 SDK 有开放实现。
- **目标用户 / 定位**：生产 Agent 团队；对 instructions + tools 组成的 agent/skill context 做版本化、环境化管理。
- **Skill 定义 / 存储**：Skill 是包含 `SKILL.md` 的 versioned repo；Hub 保存文件树和 commit history。
- **生命周期**：create/edit/commit/compare/revert，staging/production promote，SDK push/pull/list/search/delete，可按 commit pin 运行。
- **调用机制**：应用在 runtime 从 Hub 拉 latest/tag/commit，将 context 注入自有 Agent；Hub 自己不是 Agent host。
- **能力**：自动发现 ◐（Hub search/list，不扫描任意本机 host）；安装/卸载 ◐（push/pull/delete，不是 desktop install）；版本 ✅；权限 ✅（workspace）；依赖/组合 ✅（context 可含 instructions/tools，可链接 repo）；推荐 ❌；评估 ◐（可与 LangSmith eval/observability 组合，Context Hub 页未承诺自动 Skill gate）；共享 ✅；市场 ❌（workspace control plane，不是公共 marketplace）。
- **最大特点 / 不足**：最像生产 Skill GitOps/control plane；对个人本机多 Agent 文件管理和社区供给不是重点。

#### N. Microsoft Agent Framework Skill Provider（大厂公开实现）

- **链接**：[Agent Skills](https://learn.microsoft.com/en-us/agent-framework/agents/skills)、[GitHub](https://github.com/microsoft/agent-framework)
- **开源/闭源**：开源 framework。
- **目标用户 / 定位**：构建自有 Agent 应用的开发者；把 filesystem、inline、class、MCP 来源聚合成统一 Skill provider。
- **Skill 定义 / 存储**：开放 `SKILL.md`，也支持 code/class Skill；MCP 来源通过实验性的 `skill://index.json` + `skill://` resource 暴露。
- **生命周期**：source discovery、aggregation、dedup、cache、filter；不是发行市场或桌面 installer。
- **调用机制**：provider 广告 Skill 元数据，并注册 `load_skill`、`read_skill_resource`、`run_skill_script`，Agent 按需调用。远程 archive Skill 的脚本明确禁止执行。
- **能力**：自动发现 ✅；安装/卸载 ❌；版本 ?；权限 ✅（tool approval middleware、script runner、安全限制）；依赖 ◐；组合 ✅（aggregating/filtering/dedup sources）；推荐 ❌；评估 ❌；共享 ◐（MCP/filesystem）；市场 ❌。
- **最大特点 / 不足**：展示了 Skill Runtime 应有的深模块边界和安全默认；但不是面向最终开发者的 lifecycle manager。

#### O. NVIDIA Verified Agent Skills Catalog / Trust Pipeline（大厂公开治理系统）

- **链接**：[官方 catalog 与源码](https://github.com/NVIDIA/skills)
- **开源/闭源**：catalog、pipeline 相关代码与 Skill 公开；各 Skill 的上游产品和运行服务开放程度不一。
- **目标用户 / 定位**：NVIDIA 各产品团队和使用其技术的 Agent 用户；不是通用 installer，而是经过供应链验证的官方 Skill 发布与联合目录。
- **Skill 定义 / 存储**：标准 `SKILL.md`；每个已发布 Skill 还必须带 `skill-card.md`、detached OMS signature `skill.oms.sig`、Tier-3 eval dataset 和 `BENCHMARK.md`。Skill 维护在各产品上游 repo，再由自动 pipeline 每日镜像到 catalog。
- **生命周期**：上游维护 → sync → instruction/supply-chain security scan → 必需 artifact/compliance gate → signature → catalog/plugin/外部 marketplace 分发；缺 signature/card/eval 的 Skill 被 sync pipeline 排除。
- **调用机制**：通过 Vercel `skills` CLI 或 NVIDIA 打包的 Codex/Claude plugins 安装，再由目标 host 调用；签名可用 NVIDIA trust anchor 验证内容未被签名后修改。
- **能力**：自动发现 ◐（catalog/sync）；安装/卸载 ◐（依赖外部 CLI/plugin）；版本 ✅（upstream commit/catalog sync）；权限 ◐（governance card 描述边界，不等于 host enforcement）；依赖 ◐；组合 ✅（curated plugins）；推荐 ◐（catalog）；评估 ✅（强制 dataset + benchmark uplift）；共享 ✅；市场 ◐（catalog 并 syndicate 到外部市场）。
- **最大特点 / 不足**：把 provenance、签名、治理卡、安全扫描和可验证 uplift 变成发布必需品，是当前最值得借鉴的“大厂 Skill supply-chain gate”；但它是 NVIDIA 官方内容的策展/发布系统，不是任意个人 Skill 的本地 Manager。

---

## 3. 横向比较

### 3.1 核心比较表

| 产品 | Skill 抽象 | 存储 | 调用方式 | 自动发现 | 版本管理 | 最大特点 | 不足 |
|---|---|---|---|---|---|---|---|
| `gh skill` | 标准 `SKILL.md` 目录 | GitHub repo/release → host 目录 + provenance | host 原生调用 | 是，repo convention + host dirs | tag/SHA pin、tree SHA update | 当前最完整的跨 host Git 原生基线 | preview；无权限/eval/telemetry |
| Vercel `skills` | 标准目录 | Git clone/canonical `.agents/skills` + link/copy + lock | host 原生；可 `use` | 是 | source/hash update | 摩擦最低、Agent 覆盖广、skills.sh 流量入口 | 排行≈安装量；安全与效果不闭环 |
| OpenSkills | 静态 Skill 目录 | 本地目录 + `AGENTS.md` catalog | CLI read / host prompt | 是 | Git source refresh | 无 server、极轻 | lifecycle/governance 薄 |
| OpenSkill | 跨 Agent Skill + hosted governance | Git/marketplace → host 目录 | host 原生 | 是 | registry versions + Git | 个人 CLI 到 private registry/RBAC/audit | 缺效果 eval、事务回滚证据 |
| OpenClaw + ClawHub | Skill + plugin/package | 多 scope 本地 root + versioned registry | OpenClaw prompt catalog/tool dispatch | 是 | semver/tag/pin/update | 开源 registry-runtime-governance 闭环 | 强耦合 OpenClaw |
| iFlytek SkillHub | namespaced package | 自建 registry + Agent dirs + inventory | host/astron-agent | 是 | 发布/安装显式 version | RBAC、审计、扫描、私有化 | 对个人偏重；Skill update 语义未完整 |
| Alibaba `skill-up` | Skill + eval suite | repo 内 YAML cases + reports | 多 Agent engine 真实运行 | 部分 | 无 | with/baseline、judge、CI、evolution | 不是 manager/registry |
| Claude Code | standalone Skill / plugin Skill | 多 scope dirs + plugin cache/marketplace | 自动、slash、subagent | 是 | plugin version/commit/auto-update | scope、依赖、市场、企业治理成熟 | surfaces 不同步；缺统一 usage/eval gate |
| OpenAI ChatGPT/Codex | Skill authoring format + universal plugin | local multi-scope + shared plugin directory | `@`/`$`/自动匹配 | 是 | plugin 层较强，direct Skill 较弱 | ChatGPT/Codex 共用目录，多组件 bundle | surface/IDE 支持不一致 |
| Copilot/VS Code | 标准 Skill | repo/home 多兼容目录 | 自动、slash、fork context | 是 | 经 `gh skill` | IDE authoring + GitHub supply chain | 权限、效果数据分散 |
| Cursor Marketplace | plugin 内 Skill | Cursor plugin marketplace/cache | Agent 自动调用 | 是 | 未公开 | 高质量厂商 bundle | 独立 Skill lifecycle 不透明 |
| Kiro Powers | Skill；或 POWER.md+MCP+hooks | IDE/global/GitHub/marketplace | 关键词动态激活 | 是 | 未公开 | 动态 MCP tool loading | 专属 Power 格式，IDE-only |
| LangSmith Context Hub | versioned skill/agent repo | 云端 commit store + env tags | 应用 pull 后注入 | Hub 内是 | commit/tag/staging/prod | 生产 GitOps/control plane | 非本机跨 Agent manager、无公共市场 |
| MS Agent Framework | Skill provider/source | filesystem/inline/class/MCP/cache | load/read/run tools | 是 | 未公开 | 聚合、去重、缓存、安全 runner seam | 不是安装/分发产品 |
| NVIDIA Skills Catalog | 标准 Skill + skill card + signature + eval | 上游 repo → 自动镜像 catalog/plugins | 外部 CLI 安装、host 调用 | catalog 内是 | commit/sync | 签名、扫描、强制 eval/benchmark 发布门禁 | 官方策展目录，非个人本机 manager |

### 3.2 能力覆盖矩阵

| 产品 | 安装卸载 | 版本 | 权限 | 依赖 | 组合 | 推荐 | 效果评估 | 共享 | 市场 |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| `gh skill` | ◐ | ✅ | ◐ | ❌ | ◐ | ◐ | ◐ | ✅ | ◐ |
| Vercel `skills` | ✅ | ◐ | ❌ | ❌ | ✅ | ✅ | ❌ | ✅ | ✅ |
| OpenSkills | ◐ | ◐ | ❌ | ❌ | ◐ | ❌ | ❌ | ✅ | ❌ |
| OpenSkill | ✅ | ✅ | ✅ | ? | ◐ | ◐ | ◐ | ✅ | ✅ |
| OpenClaw + ClawHub | ✅ | ✅ | ✅ | ◐ | ✅ | ◐ | ◐ | ✅ | ✅ |
| iFlytek SkillHub | ✅ | ✅ | ✅ | ? | ◐ | ◐ | ◐ | ✅ | ✅ |
| Alibaba `skill-up` | ❌ | ❌ | ❌ | ❌ | ◐ | ❌ | ✅ | ✅ | ❌ |
| Claude Code | ✅ | ✅ | ✅ | ✅ | ✅ | ◐ | ◐ | ✅ | ✅ |
| OpenAI ChatGPT/Codex | ✅ | ◐ | ✅ | ✅ | ✅ | ◐ | ◐ | ✅ | ✅ |
| Copilot/VS Code + `gh` | ✅ | ✅ | ◐ | ❌ | ✅ | ◐ | ❌ | ✅ | ◐ |
| Cursor | ✅ | ? | ✅ | ✅ | ✅ | ◐ | ? | ✅ | ✅ |
| Kiro | ✅ | ? | ◐ | ✅ | ✅ | ◐ | ❌ | ✅ | ✅ |
| LangSmith Context Hub | ◐ | ✅ | ✅ | ✅ | ✅ | ❌ | ◐ | ✅ | ❌ |
| NVIDIA Skills Catalog | ◐ | ✅ | ◐ | ◐ | ✅ | ◐ | ✅ | ✅ | ◐ |

**解读：** 市场已经把“安装 + 搜索 + 多 Agent 路径适配”迅速商品化；真正稀缺的列是 **运行期权限、可解释 provenance、效果评估、跨 Skill 共存测试、基于真实使用的演化**。

---

## 4. 深度分析

### 4.1 为什么现在 Agent 需要 Skill Manager

#### 原因一：Skill 数量上升后，渐进式披露本身也有召回上限

OpenAI 明确给 Codex 的初始 Skill catalog 设置 2% context 或 8,000 字符预算；Skill 太多会先缩短 description，再省略部分 Skill。[OpenAI Build skills](https://learn.chatgpt.com/docs/build-skills) Anthropic 也建议限制同时加载的 Skill 数量，因为所有 name/description 会竞争模型注意力，API 每次请求最多 8 个 Skills。[Anthropic enterprise skills](https://platform.claude.com/docs/en/agents-and-tools/agent-skills/enterprise)

因此“把所有 Skill 都扔进目录”不是可扩展策略。Manager 必须承担过滤、推荐、role/project set 和共存测试。

#### 原因二：Skill 已经是代码供应链，而不只是 prompt

Skill 可包含 Python/Shell/JavaScript、外部 URL、MCP references、广泛文件访问和凭据需求。Anthropic 要求像生产软件一样审计第三方 Skill，并明确列出代码执行、prompt manipulation、MCP、网络、硬编码凭据和路径穿越风险。[Anthropic enterprise skills](https://platform.claude.com/docs/en/agents-and-tools/agent-skills/enterprise) OpenClaw 同样要求把第三方 Skill 当作 untrusted code，并对 upload archive、symlink、dangerous installer metadata 设置边界。[OpenClaw skills](https://github.com/openclaw/openclaw/blob/main/docs/tools/skills.md)

没有 Manager，就没有可追溯来源、审查记录、hash、权限差异、撤销和回滚。

#### 原因三：同一 Skill 正在跨多个 host 和 scope 漂移

Codex、Claude、Copilot、Cursor 等有不同 project/user/admin/plugin 目录、优先级、禁用方式和热加载行为。Anthropic 明确说明 API、claude.ai、Claude Code 的 custom Skills 不同步。[Anthropic enterprise skills](https://platform.claude.com/docs/en/agents-and-tools/agent-skills/enterprise) 跨 Agent CLI 能复制文件，但复制不等于持续一致性。

Manager 的核心价值是“一个 package revision，多份 installation 的可核对投影”，而不是把相同目录复制三次后失去来源。

#### 原因四：Skill 的质量是模型、任务和邻居 Skill 的函数

同一个 Skill 可能在 Sonnet 上有效、在另一个模型上退化；description 过宽会抢走其他 Skill 的触发。Anthropic 的正式评估维度包括 triggering accuracy、isolation、coexistence、instruction following、output quality，并要求 3–5 个代表 query、跨模型回归。[Anthropic enterprise skills](https://platform.claude.com/docs/en/agents-and-tools/agent-skills/enterprise)

所以 Skill 版本不能只用 semver/hash 管；还要附带“在哪些模型/host/任务集上通过了什么 eval”。

### 4.2 当前方案最大的痛点

1. **格式标准化但 lifecycle 未标准化。** Agent Skills spec 没有 source、lockfile、dependency、permission、signature、update、rollback、telemetry、eval 等协议。[Specification](https://agentskills.io/specification)
2. **发现指标与效果指标混淆。** skills.sh 公开说明排行榜基于匿名安装遥测；安装数只能表示分发，不表示成功率、触发准确率或安全。[skills.sh docs](https://www.skills.sh/docs)
3. **权限是 host 级，不是 package capability 级。** `allowed-tools` 只是部分 host 支持的声明；Skill 正文仍可能指示 shell/network/file 操作。跨 host 没有统一 capability manifest 或 least-privilege enforcement。
4. **版本通常记录“来源变了”，不记录“效果是否变好”。** `gh skill` 的 tree SHA 和 Vercel 的 folder hash 能判断变化，却不能判断质量回归。
5. **本地修改、复制安装和 symlink 拓扑难以对账。** 多个 Agent 的独立 copy 容易漂移；第三方 lockfile 通常没有完整 installation topology 和 ownership 证明。
6. **调用观测缺失。** Anthropic 的企业指南明确说 Skills API 当前没有 usage analytics，只能由应用层记录请求包含了哪些 Skill；甚至“包含”也不等于“实际使用/贡献”。[Enterprise guide](https://platform.claude.com/docs/en/agents-and-tools/agent-skills/enterprise)
7. **自动演化存在高风险闭环。** 从对话生成 Skill 已在 VS Code、OpenAI Record & Replay、OpenClaw Skill Workshop 出现，但若没有 pending review、eval gate、版本 diff 和回滚，Agent 可能把偶然成功或恶意输入固化成长期指令。
8. **市场治理与本地信任脱节。** registry 的星标、扫描和 moderation 不能替代用户机器上的实际权限、依赖、secret 和运行环境检查。

### 4.3 未来会演化成什么

**推断：未来的 Skill Manager 会成为 Agent Capability Control Plane。** 演化顺序大概率是：

1. **包管理器**：install/update/remove/pin/provenance。
2. **跨 host reconciler**：一个 desired state，多个 Agent/scope installation。
3. **策略执行点**：capability manifest、签名/审查、tool/network/file/secret policy。
4. **质量门禁**：with-skill vs baseline、trigger/coexistence regression、模型矩阵。
5. **上下文路由器**：根据 repo、任务、角色、token budget 选择最小 Skill set。
6. **学习闭环**：真实使用事件 → 失败聚类 → 建议 patch/eval → 人审批准 → canary/A-B → promote/rollback。
7. **可移植能力图谱**：Skill、工具、MCP、凭据、runtime、模型、任务和产出之间的依赖/证据图。

Microsoft Agent Framework 已公开 filesystem/code/MCP 多来源聚合、过滤、去重、缓存的 Runtime 形态；LangSmith 已公开 versioned context + staging/production promotion；ClawHub 已有 registry/version/moderation；`skill-up` 已有 eval/evolution。未来产品不是凭空发明，而是把这些断开的控制面合并。

---

## 5. 个人开发者 Skill Manager 产品设计建议

### 5.1 产品原则

- **Local-first，Git-native，host-adapter based。** 本地清单是控制面；Git 是来源，不自创包协议。
- **Manager 不执行第三方脚本。** MVP 只审计和部署，运行交给 host sandbox/approval；否则产品会不必要地变成第二个 Agent runtime。
- **一个 package revision，多份 installation。** Registry 中只有一个受管副本，各 host 是可验证投影。
- **未知即未知。** 不把“官方未公开”显示为“不支持”，不从路径/同名自动推断所有权。
- **所有演化都生成候选版本。** 禁止后台静默自改正在使用的 Skill。

### 5.2 核心模块

| 模块 | 功能 | 技术方案 | MVP 优先级 |
|---|---|---|---|
| **Skill Registry** | 统一 inventory、来源、revision、installations、owner、status | SQLite（单机事务/索引足够）；package 内容放 app data 的 content-addressed store；记录 Git URL/subpath/ref/commit、SHA-256、host/scope/path、deployment mode、审查状态 | **P0** |
| **Skill Runtime** | 解析 Skill、构建 host 投影、控制启停、暴露调用策略 | 不造 agent loop；`AgentAdapter` 只实现 discover/install/enable/disable/restart-hint。macOS/Linux symlink，Windows junction，失败需显式 copy fallback；所有写操作 plan/commit/rollback | **P0** |
| **Skill Discovery** | 扫本机、搜索远端、repo-aware 推荐 | 本地扫描已知 host roots + GitHub `gh skill search`/skills.sh/ClawHub adapter；抽取 language/framework/tool signals；先规则匹配，不上 embedding 服务 | **P0 本地发现；P1 远端推荐** |
| **Skill Permission** | 安装前展示并约束 shell/file/network/MCP/secret 能力 | 静态 analyzer 扫 frontmatter、scripts、URLs、commands、path patterns；生成 normalized capability disclosure；把约束映射到 host 原生 sandbox/approval，无法强制时明确标“advisory” | **P0 披露；P1 策略映射** |
| **Skill Evaluation** | 测触发、隔离、共存、指令遵循、输出质量和成本 | repo 内 `evals/`；复用 `skill-up` 作为 runner 或兼容其 YAML；最小 with-skill vs baseline、3–5 cases、rule/script judge；结果绑定 revision+host+model | **P1** |
| **Skill Memory** | 记录何时安装/启用/包含/实际调用、结果、用户反馈和失败模式 | SQLite append-only events；host 有 hook 时采事件，无 hook 只记录“included/selected”并标置信度；默认不存 prompt/output，显式 opt-in 才存摘要；支持手动 👍/👎 和 failure note | **P1** |
| **Skill Marketplace** | 搜索、预览、来源/版本/评分/兼容性展示，发布 | MVP 不自建 registry；做 federated adapters（GitHub、skills.sh、ClawHub）和统一 detail view。到有原创供给与治理需求时再建服务端 | **P2** |

### 5.3 最小数据模型

```text
SkillPackage
  id, name, source_type, source_uri, source_subpath

Revision
  package_id, content_sha256, source_ref, source_commit, created_at

Installation
  revision_id, agent, scope, logical_path, resolved_path,
  deployment_mode, enabled_state, last_verified_hash

Capability
  revision_id, kind(tool|command|network|file|secret|mcp), value,
  evidence_path, enforceability(advisory|host-enforced)

EvaluationRun
  revision_id, agent, model, suite_hash, baseline_revision,
  trigger_score, task_score, cost, latency, result_uri

UsageEvent
  installation_id, event_type(included|invoked|completed|failed|feedback),
  timestamp, task_fingerprint?, confidence
```

不在 P0 引入用户账号、云同步、通用 dependency solver、远程执行、无限版本历史或自建 marketplace。

### 5.4 MVP 端到端流程

1. **Inventory**：扫描 Codex/Claude/Copilot/Cursor 等已知根目录；普通目录、symlink、copy 分开识别；未有本产品 provenance 的均标 External。
2. **Import**：从本地目录或 Git URL staging；校验规范、路径、大小、symlink、脚本/URL；显示 capability diff。
3. **Adopt/Install**：用户选择 Agent/scope；冲突时停止；落 content-addressed revision，再原子创建 projections；写 manifest。
4. **Update**：fetch → 比较 commit/hash → staging 校验 → 展示文件/capability diff → 全部 installations 事务切换；保留一个 previous revision。
5. **Evaluate（P1）**：对候选 revision 运行 baseline/with-skill；显示触发、成功、成本、延迟；未过 gate 不推荐 promote。
6. **Rollback/Detach/Remove**：rollback 切回 previous；detach 把投影变 standalone；remove 只删 manager-owned path，零 installation 后才可删 library revision。

---

## 6. 值得实现的创新方向

### 6.1 市场缺失程度与个人适配度

| 方向 | 市场现状 | 个人工具适配 | 潜在壁垒 | 建议 |
|---|---|---:|---:|---|
| **跨 Agent inventory/reconciliation** | 多数 CLI 会安装，少数能准确解释“同一 Skill 的多份投影、漂移、所有权” | 很高 | 中 | **首发核心** |
| **Capability diff + permission mapping** | 有安全扫描/host sandbox，但缺跨 host 标准化能力清单 | 很高 | 高 | **首发披露，后续 enforcement** |
| **真实使用统计** | 安装量多，实际 invocation/effect 少；部分 host 无 hook | 高 | 高 | P1，必须区分 included/invoked/succeeded 的证据等级 |
| **with-skill vs baseline eval** | Anthropic 方法论、`skill-up` 工具已出现，Manager 内置仍少 | 高 | 高 | P1，直接复用兼容格式 |
| **Trigger/coexistence A/B** | 官方强调但产品化不足 | 中高 | 高 | 从 offline replay/canary 开始，不做在线自动切流 |
| **Skill Dependency Graph** | plugin 能声明 MCP 依赖，但跨 Skill/tool/secret/runtime 图缺失 | 高 | 中高 | 先做“声明 + 静态推断图”，不做 solver |
| **上下文预算推荐** | Vercel 排行/搜索、ClawHub vector search；缺基于 repo+token+冲突的本地路由 | 很高 | 高 | 规则 MVP，数据积累后学习排序 |
| **Skill 自动生成** | OpenAI/VS Code/OpenClaw 已有 | 高 | 低 | 不单独作为卖点；生成必须同时生成 eval |
| **Skill 自动优化** | Anthropic skill-creator、Alibaba `skill-up` 已开始 | 中高 | 高 | 只生成 PR/候选 revision，永不静默改 current |
| **Skill 质量评分** | 星标/安装量/安全分已有，跨模型任务效果分缺失 | 高 | 高 | 做多维 evidence card，不压成一个“魔法总分” |
| **Skill Evolution lineage** | 版本存在，但“失败→patch→eval→promote”证据链少 | 高 | 高 | 绑定 issue/eval/result/revision，形成长期壁垒 |

### 6.2 最有价值的三条创新主线

#### 1. Evidence-backed Skill Scorecard

不要只显示 87/100。展示五组可追溯证据：

- Provenance：来源、签名/commit、审查者、内容 hash。
- Safety：静态能力、扫描器结果、host 可强制程度。
- Trigger：positive/negative/ambiguous queries 的 precision/recall。
- Outcome：with-skill vs baseline 成功率、成本、延迟。
- Compatibility：在哪些 host/model/version 上测试通过。

壁垒来自长期积累的 revision × model × host × task 结果，而不是 UI。

#### 2. Local Skill Router

输入 repo signals、当前任务、active Skill catalog、token budget、冲突图，输出最小 Skill set，并解释：

```text
selected: tdd, diagnosing-bugs
excluded: generic-testing (overlaps tdd, lower eval score)
budget: 2,140 / 4,000 catalog tokens
```

先用确定性规则和 description BM25/embedding 可选索引；只有真实反馈积累后才做 learning-to-rank。个人设备上的项目/使用信号可保持本地，形成隐私优势。

#### 3. Safe Evolution Pipeline

```text
usage/failure
  -> failure cluster
  -> candidate SKILL.md patch + new regression case
  -> static capability diff
  -> baseline/coexistence eval
  -> human review
  -> canary one project/host
  -> promote or rollback
```

OpenClaw Skill Workshop 已证明“从观察生成 Skill + 扫描 + pending approval”可行；Alibaba `skill-up` 已证明 eval/evolution loop 可工具化。差异化在于把二者接到 package provenance 和 deployment promotion 上，而不是再做一个生成 prompt。[OpenClaw Skill Workshop](https://github.com/openclaw/openclaw/blob/main/docs/tools/skills.md)、[skill-up](https://github.com/alibaba/skill-up)

---

## 7. Skill Manager 产品 PRD 草案

### 7.1 产品名与一句话

**暂名：Skill Deck**  
一个 local-first 的 Agent Skill 控制台：统一管理多种 AI coding agent 的 Skill 来源、版本、权限、安装和效果。

### 7.2 背景与问题

个人开发者同时使用 Codex、Claude Code、Copilot、Cursor 等工具时，Skill 分散在不同目录和 scope；现有 CLI 擅长下载，却难回答来源、漂移、权限、实际效果和安全更新。随着 Skill 能执行脚本并依赖 MCP/凭据，它已成为本地软件供应链的一部分。

### 7.3 目标用户

- **Primary**：同时使用 2 个以上 coding agent、维护 10–100 个个人/项目 Skill 的开发者。
- **Secondary**：维护内部 Skill 仓库、尚不需要重型企业平台的 2–20 人团队。
- **非目标**：需要集中式千人 RBAC/合规审批的企业；需要托管 Agent execution 的平台团队。

### 7.4 Jobs to be Done

1. 当我换 Agent 或新开项目时，希望复用已审查的 Skill，不重复复制和配置。
2. 当 Skill 更新时，希望看到来源、文件和能力变化，确认后一次安全升级所有安装。
3. 当结果变差时，希望知道是否由某个 Skill/版本/冲突导致，并能快速回滚。
4. 当我发现重复流程时，希望生成候选 Skill 和回归用例，而不是把未经验证的经验直接固化。

### 7.5 产品目标与非目标

**MVP 目标**

- 发现并对账 3–4 个主流 Agent 的 user/project Skills。
- 本地/Git 导入，记录 commit/hash/provenance。
- 原子安装、更新、回滚、detach、remove，永不覆盖未知内容。
- 安装前显示结构验证和 capability disclosure。
- 提供本地搜索、过滤、冲突/漂移诊断。

**MVP 非目标**

- 自建公共 marketplace、账号、云同步、支付。
- 自己执行 Skill 脚本或托管 Agent。
- 自动安装依赖、自动修改安全策略。
- 后台静默更新、静默演化、跨 Skill dependency solver。
- 用一个不可解释总分宣称 Skill “安全”。

### 7.6 功能需求

#### P0：可信包管理

- Inventory：按 Skill package 聚合 Agent/scope installations；显示 managed/external/drift/conflict/broken。
- Import：本地目录、公开 HTTPS Git URL、repo subpath；staging 后才进入 library。
- Validation：Agent Skills 必需字段、目录名、路径穿越、symlink/junction、资源大小、文件数。
- Capability disclosure：脚本、命令、网络域名、文件范围、MCP/tool/secret 引用，附 evidence path。
- Install plan：目标 Agent/scope、logical/resolved path、link/copy mode、冲突和 restart hint。
- Provenance：source URI/subpath/ref/commit/content hash、installed-at、manager ownership。
- Lifecycle：enable/disable（host 支持时）、update preview、transaction commit、single previous rollback、detach、uninstall、remove library。
- Diagnostics：本地修改、源不可达、源分叉、重复名、legacy path、config drift。

#### P1：质量与个性化

- Eval suites：兼容 `skill-up`/Anthropic 风格；3–5 starter cases。
- Baseline comparison：without vs with candidate revision。
- Coexistence tests：与当前 active set 一起测 trigger 和 task regression。
- Usage memory：included/invoked/result/feedback 事件和证据等级；默认隐私保护。
- Local recommendation：根据 repo manifest、语言、工具、用户查询和历史反馈推荐 Skill/set。
- Candidate evolution：从失败生成 patch + regression case，保存为未发布 revision。

#### P2：生态

- Federated marketplace：GitHub、skills.sh、ClawHub 搜索聚合。
- Team Git registry、签名/attestation、review policy。
- A/B/canary promotion、共享 scorecard、兼容性认证。

### 7.7 关键用户流程与验收标准

#### 首次扫描

- 能识别支持 host 的实际根目录和 scope。
- 不因同名自动合并不同内容。
- 未有 manifest 证明的目录均为 External，不提供破坏性操作。

#### 导入/安装

- 所有输入先进入 staging；任何校验失败不改变现有文件。
- 展示来源、hash、目标、文件列表与 capability disclosure。
- 目标冲突时默认停止；`force` 也不得覆盖非 managed path。
- 多 Agent 安装要么全部成功，要么全部回滚。

#### 更新

- 显示 old/new commit、内容 diff、capability diff、已有 eval 状态。
- 本地 drift 时停止，不自动 merge/rebase。
- 成功后所有 installations 指向同一 revision；可一键回上一版。

#### 评价

- 每个结果绑定 revision、host、model、suite hash。
- 至少报告 trigger、task success、cost/latency；禁止只给综合分。
- 未运行 eval 显示“未评估”，不能显示为 0 分或安全。

#### 删除

- uninstall 只删除 manifest 证明为本产品创建的 projection。
- external symlink target、第三方 lock 和未知配置永不删除。
- 有 installations 的 package 不能 Remove from Library。

### 7.8 成功指标

- **Activation**：首次启动 10 分钟内成功完成 inventory + 1 个 managed install。
- **Reliability**：写操作故障注入下 100% 保持旧版可用或完整回滚。
- **Trust**：100% managed revisions 有 source + content hash；100% 更新有 capability diff。
- **Utility**：活跃用户中跨 2+ Agent 安装同一 package 的比例；每月成功 update/rollback 数。
- **Quality（P1）**：推荐接受率、trigger precision/recall、with-skill 相对 baseline 的任务成功率提升。
- **Safety**：对未知/漂移路径的误删为 0；所有高风险 capability 必须显式确认。

不要把下载量、安装量或 DAU 单独当作 Skill 质量指标。

### 7.9 风险与缓解

| 风险 | 缓解 |
|---|---|
| Agent 路径/配置经常变化 | adapter 独立版本化；发布前用真实 host contract tests |
| 静态扫描产生“安全错觉” | 用 capability disclosure/证据，不宣称 safe；运行权限交给 host |
| Host 不暴露 Skill 调用事件 | 事件标置信度；只报告 included/selected，不伪造 invoked |
| 自动演化污染长期指令 | candidate-only、human review、eval gate、single-click rollback |
| 多 copy 漂移与误删 | content-addressed library、ownership manifest、事务、external 默认只读 |
| 市场冷启动 | 聚合现有 GitHub/skills.sh/ClawHub，不自建供给侧 |
| Eval 成本高且不稳定 | 小型代表集、缓存、可复现配置、rule judge 优先、模型 judge 可选 |

### 7.10 路线图

**Phase 0 — 4–6 周：可信 inventory**  
Codex/Claude/Copilot 三适配器；扫描、外部/受管区分、Git/local import、hash/provenance、只读诊断。

**Phase 1 — 4–6 周：事务 lifecycle**  
link/copy install、multi-target plan/commit/rollback、update diff、single previous、detach/uninstall。

**Phase 2 — 4–8 周：评价闭环**  
兼容 `skill-up` 的 eval runner、baseline、coexistence、revision scorecard、manual feedback。

**Phase 3 — 6–10 周：本地推荐与演化**  
repo-aware set recommendation、usage events、candidate patch + regression case、canary project。

**Phase 4 — 有真实供给后再做**  
federated marketplace UI、team Git source、签名/attestation；只有出现明确私有共享需求才建设服务端 registry。

---

## 8. 最终产品判断

如果只做“把 GitHub 的 `SKILL.md` 复制到几个目录”，`gh skill` 和 Vercel `skills` 已经足够好，重复实现没有产品空间。

值得做的产品是：

> **一个能证明“这个 Skill 从哪来、当前在哪运行、拥有什么能力、是否发生漂移、在我的 Agent/模型/任务上是否真的有效”的 local-first control plane。**

首个可出售/可形成口碑的组合不是 Marketplace，而是 **Cross-Agent Inventory + Provenance + Capability Diff + Transactional Update/Rollback**；第一个长期壁垒是 **Revision-level Eval & Usage Evidence Graph**。市场、自动生成和自动优化都应建立在这层证据之上。


基于当前实现校准后的执行路线见 [Skill Deck Roadmap](../roadmap.md)。
