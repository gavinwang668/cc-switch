# CC Switch CLI 功能评审报告

> 评审日期：2026-07-04
> 评审范围：`docs/cli-feature-implementation-assessment.md`、`docs/cli-reference-manual.md`
> 评审维度：分类合理性、优先级与范围、实现与文档一致性、架构与依赖
> 状态：待用户审阅

---

## 一、核心评审标准

本评审基于以下核心标准（用户确认）：

- **A. 协议转换**：支持 Claude / Claude Desktop / Codex / Gemini / OpenClaw 等应用调任意模型，代理软件负责协议转换
- **B. 供应商模型配置**：供应商与模型配置是核心功能
- **C. API 格式设置**：API 格式设置是核心功能
- **D. CLI 与 GUI 等价**：CLI 和 GUI 都必须支持上述 A/B/C

任何功能若属于 A/B/C，则 CLI 必须实现（REQ）；若仅是 GUI 交互/应用自身功能，则归 N/A。

---

## 二、概览与总体结论

### 2.1 评审方法

交叉验证了以下源码：

- [src-tauri/src/bin/cc-switch-cli.rs](file:///f:/workspace/trae/cc-switch/src-tauri/src/bin/cc-switch-cli.rs) — CLI 命令定义与实现
- [src-tauri/src/lib.rs](file:///f:/workspace/trae/cc-switch/src-tauri/src/lib.rs) — lib crate 入口，Tauri Builder 与 managed state 注册
- [src-tauri/src/commands/mod.rs](file:///f:/workspace/trae/cc-switch/src-tauri/src/commands/mod.rs) — Tauri 命令 re-export
- [src-tauri/src/core/decl_config.rs](file:///f:/workspace/trae/cc-switch/src-tauri/src/core/decl_config.rs) — 声明式配置实现
- [src-tauri/Cargo.toml](file:///f:/workspace/trae/cc-switch/src-tauri/Cargo.toml) — 依赖与 crate 配置

### 2.2 总体结论

整体合理，但存在 3 类系统性问题，使得"三阶段全部完成 ✅"的表述与实际能力存在偏差：

1. **分类边界含糊**：评估文档把"成本倍率、计费来源、通用配置片段"列为 REQ，但这些是统计层配置而非协议转换核心。同时 OpenClaw/Hermes 的供应商模型配置未升入 REQ，与核心标准 B 不一致。
2. **"已实现"的水分**：至少 4 个命令是桩实现（stream-check、stream-check-all、remove-session）或半应用（apply-config 的代理字段），与"全部完成 ✅"声称冲突。
3. **架构耦合未解耦**：CLI 和 GUI 共享 `cc_switch_lib` crate，该 crate 通过 `pub use commands::*` 暴露所有 `#[tauri::command]`（含 `tauri::AppHandle` / `tauri::State` 参数），导致 CLI 编译强耦合 Tauri/webkit2gtk。

### 2.3 评级概览

| 维度 | 评级 | 说明 |
|------|------|------|
| 分类合理性 | 良好但有改进空间 | 三层划分清晰；个别 REQ 归类偏激进；OpenClaw/Hermes 模型配置应升入 REQ |
| 优先级与范围 | 中等 | 19 项 REQ 优先级基本合理；60 项 OPT 存在明显 YAGNI 隐患 |
| 实现与文档一致性 | **不及格** | "全部完成 ✅" 与 4 处桩实现/部分实现直接冲突 |
| 架构与依赖 | **不及格** | 共享 lib crate 导致 CLI 强耦合 Tauri，构建/分发受阻 |

### 2.4 关键修复优先级

| 优先级 | 问题 | 章节 |
|--------|------|------|
| P0 | 修正评估文档的"全部完成 ✅"声称，标注 4 个桩命令 | §五 |
| P0 | 修正 `apply-config` 文档，明确哪些字段不应用 | §五 |
| P1 | 重构 lib crate，将 Tauri 命令与可复用业务逻辑分离 | §六 |
| P1 | 重新评估 OPT-001~060 的实际必要性，删除长期不实现项 | §四 |
| P2 | 调整 REQ-007/008/013 的分类，移至"代理配置补齐"或降级 OPT | §三 |

---

## 三、分类合理性评审

### 3.1 评审标尺

评估文档的定义：

- **必须实现 (REQ)** = "代理和协议转换的等价能力"，"代理运行必须"
- **可实现 (OPT)** = "MCP/Prompt/Skills/环境变量/会话等附带功能，非代理核心流程"
- **没必要实现 (N/A)** = "云同步、AUTH、Keychain、强依赖 GUI 运行时"

判别问题：**"如果 CLI 不实现这项，代理还能不能跑、协议还能不能转、模型还能不能配？"** 若能，则不应列入 REQ；若属于核心标准 A/B/C，则必须列入 REQ。

### 3.2 REQ 类（19 项）— 重新评估

#### 分类正确的（14 项，无异议）

REQ-001/002/003/004/005（供应商排序、Live 导入、读取 Live、模型列表、同步 Live）、REQ-006（代理配置读写）、REQ-009（Live 接管检测）、REQ-010/011/012（熔断器/健康/可用故障转移列表）、REQ-015（请求日志）、REQ-017/018/019（备份删改、自定义端点）。

#### 归类偏激进，建议降级（3 项）

| 编号 | 当前 | 评估文档的论证 | 异议 | 建议 |
|---|---|---|---|---|
| **REQ-007** 成本倍率设置 | REQ | "代理成本统计的基础配置，影响用量统计准确性" | 成本倍率是统计层配置，与代理能否转发请求无关。代理不设倍率也能正常跑，只是用量报表里的成本不准。 | 降级 OPT |
| **REQ-008** 计费模型来源 | REQ | "切换计费模型来源影响协议转换后的成本统计" | `official` / `custom` 只影响成本计算，不影响协议转换本身。论证里"协议转换后"四个字是误导。 | 降级 OPT |
| **REQ-013** 通用配置片段管理 | REQ | "多供应商共享配置…协议转换的公共部分" | 通用片段是配置编辑便捷性功能（提取/复用 env），代理运行时只用每个供应商自己的 env，不读 snippet。 | 降级 OPT |

#### 边界可接受但需澄清（2 项）

- **REQ-014 按应用统计用量**：可保留 REQ，但需明确"仅 summary 视图"还是"包含全部按应用统计"。
- **REQ-016 供应商限额检查**：本质是"基于代理数据的二次分析"，可保留 REQ 但需说明"仅检查不阻断"。

### 3.3 OPT 类（60 项）— 范围过大问题

#### 类 A：CLI 已明确不实现，仍列 OPT 制造悬念（建议归 N/A）

| 编号 | 功能 | 理由 |
|---|---|---|
| OPT-028 工具版本检测 | "检测 CLI 工具安装状态"在无头服务器上意义有限，用户用 `which claude` 即可 |
| OPT-041 供应商复制 | 文档自承"GUI 内部实现"——GUI 专属功能不该出现在 OPT |
| OPT-046 Codex 历史迁移 | 一次性迁移操作，服务器场景几乎用不到 |

#### 类 B：应用专属功能堆叠（按核心标准 B 拆分）

OPT-047~054（OpenClaw 8 项）、OPT-055~057（Hermes 3 项）、OPT-058~060（OMO 3 项）共 14 项。按核心标准 B（供应商模型配置）重新拆分：

| 编号 | 功能 | 新分类 | 理由 |
|---|---|---|---|
| OPT-047 OpenClaw 配置健康扫描 | OPT-A | 运维辅助 |
| **OPT-048 OpenClaw 默认模型** | **REQ（新增）** | 供应商模型配置 = 核心 B |
| **OPT-049 OpenClaw 模型目录** | **REQ（新增）** | 模型配置 = 核心 B |
| OPT-050 OpenClaw Agents 默认值 | OPT-A | OpenClaw 应用配置，边缘核心 |
| OPT-051 OpenClaw 环境变量 | OPT-A | 供应商 env 配置 |
| OPT-052 OpenClaw 工具配置 | OPT-B | OpenClaw 应用自身 |
| OPT-053 OpenClaw 工作区文件 | **N/A** | 含 `open_workspace_directory`，应用自身功能 |
| OPT-054 OpenClaw 每日记忆 | **N/A** | OpenClaw 应用自身功能 |
| **OPT-055 Hermes Live 供应商导入** | **与 REQ-002 合并** | 已在 REQ-002 体现，重复 |
| **OPT-056 Hermes 模型配置查看** | **REQ（新增）** | 模型配置 = 核心 B |
| OPT-057 Hermes 记忆管理 | **N/A** | Hermes 应用自身功能 |
| OPT-058~060 OMO（3 项） | **N/A** | OMO 是独立工具，非核心场景 |

#### 类 C：合理 OPT，但优先级文档缺失

合理 OPT 项（如 OPT-001~006 MCP、OPT-007~011 Prompt、OPT-022~024 环境变量）保留，但 60 项 OPT 没有**子优先级**，"择机实现"等价于"永不实现"。建议拆为 OPT-A（高价值）/ OPT-B（低价值）。

### 3.4 API 格式设置的覆盖度需补强（核心标准 C）

核心标准 C 是"API 格式设置"。检查发现一个**遗漏**：

- `add-provider` / `update-provider` 支持 `--api-format`（[cc-switch-cli.rs:78~84](file:///f:/workspace/trae/cc-switch/src-tauri/src/bin/cc-switch-cli.rs#L78)），但**只覆盖 claude/codex/gemini/claude-desktop 四种应用**
- OpenClaw/Hermes 的 `add-provider` 是否支持 `--api-format` 未在文档中说明
- 评估文档没有把"API 格式设置"列为独立 REQ 项，散落在 REQ-004/005 里

建议新增 **REQ-020：API 格式设置覆盖全部 7 种应用**，作为核心 C 的明确条目。

### 3.5 分类调整建议清单

| 调整 | 涉及编号 | 数量 |
|---|---|---|
| REQ → OPT 降级 | REQ-007、REQ-008、REQ-013 | 3 |
| OPT → REQ 升级 | OPT-048、OPT-049、OPT-056（+ OPT-055 合并入 REQ-002） | 3 升 1 合并 |
| OPT → N/A 降级 | OPT-017~021（Skills 仓库 5 项）、OPT-028、OPT-041、OPT-046、OPT-053、OPT-054、OPT-057、OPT-058~060 | 14 |
| REQ 边界澄清 | REQ-014、REQ-016 明确"仅 summary/仅检查" | 2 |
| REQ 拆分 | REQ-006 拆为 REQ-006a（基础配置）/ REQ-006b（应用级配置） | 1 拆 2 |
| OPT 拆分子优先级 | OPT-001~045 拆为 OPT-A / OPT-B | 45 |
| 新增 REQ | REQ-020（API 格式全覆盖）、REQ-021（协议转换烟雾测试，见 §四） | 2 |

### 3.6 修订后分类概览

| 分类 | 数量 | 说明 |
|---|---|---|
| **REQ（必须）** | **21** | 19 − 3 降级 + 3 升级 + 2 新增（REQ-006 计为 1 项含 a/b 子项） |
| OPT-A（高价值附带） | 12 | MCP/Prompt/环境变量核心管理 |
| OPT-B（低价值附带） | 16 | 详细统计/会话查询等 |
| N/A（新增） | 14 | Skills 仓库 5 + 应用自身 6 + 工具/一次性 3 |
| N/A（总计） | 34 | 原 20 + 14 OPT 降级 |

> 注：REQ 总数若按子项计为 22 项（REQ-006a 与 REQ-006b 分别计），本文档统一按 21 项表述（REQ-006 计为 1 项）。

---

## 四、优先级与范围评审

### 4.1 评审标尺

按核心标准 A/B/C 重新排优先级，并检查范围合理性：

- **优先级正确性**：REQ 内部顺序是否反映了"代理能跑 → 协议能转 → 模型能配 → 运维能查"的依赖链
- **范围控制**：OPT 是否存在过度设计（YAGNI）

### 4.2 REQ 优先级问题

#### 问题 1：REQ-009（Live 接管检测）排位偏后

当前在 Phase 1 末尾。实际依赖：代理启动后**第一件事**应该是确认 Live 是否被接管，否则后续 `switch-proxy` / `takeover` 行为无法预测。

建议：移到 Phase 1 第 2 位，紧随 REQ-002（Live 导入）。

#### 问题 2：REQ-006（代理配置读写）覆盖范围需明确

REQ-006 当前涵盖 6 个子命令（global / app / proxy 配置）。降级 REQ-013 后，"通用配置片段"不再算核心，但 REQ-006 仍含 `proxy-config` 的细分项。

建议：拆 REQ-006 为 6a（基础 listen/port/retry）、6b（app 级 enabled/auto_failover）。

#### 问题 3：新增 REQ 项未排阶段

| 新 REQ | 建议阶段 | 理由 |
|---|---|---|
| REQ-020 API 格式覆盖 7 应用 | Phase 1 | 核心 C，与 add-provider 同期实现 |
| REQ-021 协议转换烟雾测试 | Phase 1 | 核心 A 的直接验证 |
| OpenClaw 048 默认模型 | Phase 2 | 依赖 OpenClaw 供应商已存在 |
| OpenClaw 049 模型目录 | Phase 2 | 同上 |
| Hermes 056 模型配置查看 | Phase 2 | 依赖 Hermes 供应商已存在 |

#### 问题 4：Phase 1 缺少"协议转换验证"项

按核心标准 A（协议转换），Phase 1 应有一项"验证各应用协议转换是否工作"。当前 `verify-key` / `speedtest` 在 Phase 2 末尾，但它们只测连通性，不测协议转换。

建议：Phase 1 新增 **REQ-021：协议转换烟雾测试**（每个应用至少跑一次最小请求验证转换链路）。

### 4.3 修订后的 Phase 1（核心能力，11 项）

```
REQ-002 Live 导入         ← 起点
REQ-009 Live 接管检测      ← 上移
REQ-001 供应商排序
REQ-003 读取 Live
REQ-004 模型列表
REQ-005 同步 Live
REQ-006a 代理基础配置
REQ-020 API 格式全覆盖    ← 新增
REQ-021 协议转换烟雾测试   ← 新增
（REQ-007/008/013 降级至 OPT）
```

### 4.4 修订后的 Phase 2（运维监控，13 项）

```
REQ-006b 应用级代理配置
REQ-010 熔断器统计
REQ-011 供应商健康
REQ-012 可用故障转移列表
REQ-014 按应用统计用量
REQ-015 请求日志
REQ-016 限额检查
REQ-017 备份删除
REQ-018 备份重命名
REQ-019 自定义端点
OpenClaw 048 默认模型     ← 新增
OpenClaw 049 模型目录     ← 新增
Hermes 056 模型配置       ← 新增
```

### 4.5 OPT 范围评审（YAGNI 检查）

对每个 OPT 项问三个问题：

1. **使用频率**：CLI 用户多久用一次？（>每月 = 高频，<每年 = 低频）
2. **替代成本**：不实现 CLI 版，用户能否用其他方式完成？（可替代 = 低价值）
3. **实现成本**：实现需要多少代码？（>500 行 = 重）

#### OPT-A 候选（高价值，建议实现，12 项）

| 编号 | 功能 | 理由 |
|---|---|---|
| OPT-001 | 添加/更新 MCP | MCP 是 Claude/Codex 核心扩展，CLI 必备 |
| OPT-002 | 删除 MCP | 与 001 配对 |
| OPT-003 | 启用/禁用 MCP | 高频操作 |
| OPT-004 | 从应用导入 MCP | 首次部署常用 |
| OPT-005 | 测试 MCP 连接 | 排障必备 |
| OPT-007 | 添加/更新 Prompt | Prompt 是核心配置 |
| OPT-008 | 删除 Prompt | 配对 |
| OPT-009 | 启用/禁用 Prompt | 高频 |
| OPT-022 | 环境变量冲突检查 | 排障高频 |
| OPT-023 | 删除环境变量 | 配对 |
| OPT-039 | 通用供应商管理 | 跨应用配置，符合核心 B |
| OPT-040 | 配置目录覆盖 | 多实例部署需要 |

#### OPT-B 候选（低价值，暂缓或拒绝，16 项）

| 类别 | 编号 | YAGNI 理由 |
|---|---|---|
| GUI 已有更优交互 | OPT-010/011/017/018/019/020/021 | Skills 仓库浏览/批量安装/备份恢复，GUI 操作远优于 CLI |
| 一次性操作 | OPT-006/024/042/043/044/045 | Claude 专属配置/迁移，CLI 用户用一次就完，不值得维护 |
| 可被外部工具替代 | OPT-025/026/027 | 会话查看用 `sqlite3` 或 GUI 即可 |
| 依赖代理运行时 | OPT-029/030/031 | 流式检查需代理在跑，与 CLI 独立部署场景冲突（详见 §五） |
| 用量统计深度 | OPT-032~038 | 7 项详细统计，CLI 用户用 `usage-summary` 已够，深度分析用 GUI |
| 应用自身功能 | OPT-050/051/052 | OpenClaw Agents/工具/env 配置 |

#### 范围过大具体证据

**证据 1：OPT-017~021（Skills 仓库管理 5 项）严重 YAGNI**

这 5 项是 GUI 的 Skills 商店功能镜像，CLI 用户极少在终端浏览仓库。建议**全部归 N/A**。

**证据 2：OPT-032~038（用量统计 7 项）过度细分**

7 项里有 5 项（035/036/037/038 + 033 或 034 之一）可归 N/A。CLI 用户用 `usage-summary` + `usage-by-app` + `provider-stats` 三个命令足够。

**证据 3：OPT-006（Claude MCP 专属管理 5 子项）冗余**

`get_claude_mcp_status` / `read_claude_mcp_config` / `upsert_claude_mcp_server` / `delete_claude_mcp_server` / `validate_mcp_command` —— 这 5 个 Claude 专属命令与通用 OPT-001~005 完全重叠。CLI 不需要为 Claude 单独做一套。

### 4.6 范围收敛建议

| 分类 | 原评估 | 修订建议 | 变化 |
|---|---|---|---|
| REQ | 19 | 21（19 − 3 降级 + 3 升级 + 2 新增，REQ-006 计 1 项）| +2 |
| OPT-A | 60 | 12 | −48 |
| OPT-B | 0 | 16 | +16 |
| N/A（新增） | 0 | 14 | +14 |
| N/A（总计） | 20 | 34（20 + 14）| +14 |

---

## 五、实现与文档一致性评审

### 5.1 评审标尺

评估文档顶部声称：

> **实现状态**：Phase 1（REQ-001~009）✅、Phase 2（REQ-010~019）✅、Phase 3（OPT 系列关键功能）✅ 已全部实现

抽样核对 `src-tauri/src/bin/cc-switch-cli.rs` 中的命令实现，与参考手册逐项对照。结论：**"全部完成 ✅" 的声称不成立**。

### 5.2 不一致问题清单

#### 🔴 问题 1：桩实现命令（P0 阻塞性）

**评估文档声称**：Phase 3 ✅ 全部完成
**实际**：4 个命令是桩实现，调用后只打印"暂不支持"提示，无实际功能

| 命令 | 源码位置 | 桩实现内容 |
|---|---|---|
| `stream-check` | [cc-switch-cli.rs:3990](file:///f:/workspace/trae/cc-switch/src-tauri/src/bin/cc-switch-cli.rs#L3990) | `eprintln!("流式检查需要代理服务器运行中且 CopilotAuthState 初始化，当前 CLI 环境不支持。")` |
| `stream-check-all` | [cc-switch-cli.rs:4001](file:///f:/workspace/trae/cc-switch/src-tauri/src/bin/cc-switch-cli.rs#L4001) | 同上 |
| `remove-session` | [cc-switch-cli.rs:3868](file:///f:/workspace/trae/cc-switch/src-tauri/src/bin/cc-switch-cli.rs#L3868) | `eprintln!("删除会话需要提供 provider_id 和 source_path，当前 CLI 命令暂不支持完整参数。")` |

**严重性**：用户读参考手册会以为这些命令可用，实际调用直接失败。参考手册第 1913~1922 行虽有小字提示"当前 CLI 环境不支持"，但**评估文档的"全部完成 ✅"声称与参考手册自相矛盾**。

#### 🔴 问题 2：apply-config 半应用（P0 阻塞性）

**评估文档声称**：REQ-019 自定义端点 ✅，声明式配置已在 Phase 1~3 实现
**参考手册 1383~1385 行明确**：

> `proxy.listen` / `proxy.port` / `proxy.takeover` 目前只参与解析和校验，**不会被 `apply-config` 实际应用**。请使用 `proxy-config` 命令、`CC_SWITCH_LISTEN` / `CC_SWITCH_PORT` 环境变量或 `takeover` 命令来设置。

**代码验证** [decl_config.rs:159](file:///f:/workspace/trae/cc-switch/src-tauri/src/core/decl_config.rs#L159)：`apply()` 方法只应用了 5 件事：

1. 供应商配置
2. 全局出站代理
3. 故障转移队列
4. 自动故障转移开关
5. 设备级设置

**未应用**：`proxy.listen`、`proxy.port`、`proxy.takeover` —— 这三项在 YAML 里看起来能配置，校验也通过，但 `apply-config` 后不生效。

**严重性**：声明式配置是 CLI 自动化部署的核心场景。用户写 YAML 配置了 `proxy.listen: 0.0.0.0`，校验通过、apply 成功，但实际仍监听 127.0.0.1。这是**静默失败**，比桩实现更危险。

#### 🟡 问题 3：api-format 应用覆盖不全（P1）

**核心标准 C（REQ-020）**：API 格式设置应覆盖全部 7 种应用
**参考手册 643~648 行**：

| 应用类型 | 支持的格式 |
|----------|-----------|
| claude | anthropic / openai_chat / openai_responses |
| codex | openai_responses / openai_chat |
| gemini | gemini_native / openai_chat / openai_responses / anthropic |
| claude-desktop | anthropic / openai_chat / openai_responses / gemini_native / bedrock |

**未列出**：opencode、openclaw、hermes 三种应用。

**代码验证** [cc-switch-cli.rs:78~84](file:///f:/workspace/trae/cc-switch/src-tauri/src/bin/cc-switch-cli.rs#L78)：`AddProvider.api_format` 字段注释也只列了 claude/codex/gemini/claude-desktop 四种。

**严重性**：按核心标准 C，这是核心功能未覆盖完整。OpenClaw/Hermes 是 §三升入 REQ 的应用，但 API 格式设置对它们的支持缺失。

#### 🟡 问题 4：add-provider env 字段命名硬编码（P1）

**代码验证** [cc-switch-cli.rs:1180](file:///f:/workspace/trae/cc-switch/src-tauri/src/bin/cc-switch-cli.rs#L1180)：

```rust
if let Some(key) = api_key {
    env.insert("ANTHROPIC_API_KEY".to_string(), ...);
}
if let Some(url) = base_url {
    env.insert("ANTHROPIC_BASE_URL".to_string(), ...);
}
```

**问题**：`add-provider` 命令无论 `--app` 是什么，env 字段名都硬编码为 `ANTHROPIC_API_KEY` / `ANTHROPIC_BASE_URL`。对 Codex（应使用 `OPENAI_API_KEY`）、Gemini（`GEMINI_API_KEY`）、OpenClaw、Hermes 等应用，写入的 env 名错误，导致供应商配置后无法被对应应用读取。

**严重性**：按核心标准 B（供应商模型配置），这是核心功能 bug。用户按手册示例添加 Codex 供应商会得到一个不可用的配置。

### 5.3 其他次要不一致

#### 🟢 问题 5：speedtest/verify-key 不依赖代理

**代码验证** [cc-switch-cli.rs:2278/2307](file:///f:/workspace/trae/cc-switch/src-tauri/src/bin/cc-switch-cli.rs#L2278)：两个命令直接发 HTTP 请求，不读代理状态。

**影响**：手册未说明这点，用户可能误以为需要代理在跑才能用。属于文档清晰度问题，不影响功能。

#### 🟢 问题 6：list-providers 输出列名误导

**代码验证** [cc-switch-cli.rs:1138](file:///f:/workspace/trae/cc-switch/src-tauri/src/bin/cc-switch-cli.rs#L1138)：

```rust
.or_else(|| provider.settings_config.pointer("/env/ANTHROPIC_BASE_URL"))
.or_else(|| provider.settings_config.pointer("/env/BASE_URL"))
```

**问题**：list-providers 显示 Base URL 时只尝试 `ANTHROPIC_BASE_URL` 和 `BASE_URL`，对 Codex（`OPENAI_BASE_URL`）等应用会显示 `-`。

**严重性**：显示问题，不影响功能。

### 5.4 "全部完成 ✅" 评级表

| 阶段 | 评估文档声称 | 实际评级 | 差距原因 |
|---|---|---|---|
| Phase 1 (REQ-001~009) | ✅ 全部完成 | **🟡 基本完成** | apply-config 半应用（REQ-019 关联）、add-provider env 硬编码 |
| Phase 2 (REQ-010~019) | ✅ 全部完成 | **🟢 完成** | 抽样核对 check-limits/takeover-status/request-logs 均真实实现 |
| Phase 3 (OPT 关键) | ✅ 全部完成 | **🔴 未完成** | 4 个桩实现命令 + apply-config 代理字段未应用 |

---

## 六、架构与依赖评审

### 6.1 评审标尺

评估文档把架构问题归入"已知限制"章节（如 webkit2gtk 构建依赖），但未分析根因。本节回答：**这些限制是技术不可避免，还是架构选择导致？**

### 6.2 问题 1：CLI 与 GUI 共享 lib crate 导致强耦合（P1 架构问题）

#### 现象

参考手册第 99 行：

> **注意**：即使只编译 CLI，当前仍需要 webkit2gtk 开发库，因为 CLI 和 GUI 共享同一个 lib crate。运行时不需要 webkit2gtk。

#### 根因分析

**Cargo.toml 第 14~17 行**：

```toml
[lib]
name = "cc_switch_lib"
crate-type = ["staticlib", "cdylib", "rlib"]
```

lib crate 同时被 GUI 二进制（`cc-switch`）和 CLI 二进制（`cc-switch-cli`）依赖。

**lib.rs 第 9 行** `pub mod commands;` + `commands/mod.rs` 第 39~71 行 `pub use auth::*; pub use balance::*; ...` —— 把所有 `#[tauri::command]` 函数全部 re-export 到 lib 根。

**这些 commands 普遍依赖 Tauri 类型**：

| 文件 | 依赖证据 |
|---|---|
| `commands/provider.rs:2` | `use tauri::{Emitter, State};` |
| `commands/provider.rs:376` | `app_handle: tauri::AppHandle` |
| `commands/proxy.rs:13` | `state: tauri::State<'_, AppState>` |
| `commands/copilot.rs:10` | `use tauri::State;` |
| `commands/stream_check.rs:14` | `use tauri::State;` |

**lib.rs 本身**第 77~80 行直接 `use tauri::{Emitter, Manager};`，第 196 行 `#[tauri::command]` 在 lib.rs 内定义。

#### 后果

1. **构建依赖**：CLI 编译必须安装 webkit2gtk / GTK / Tauri 全部开发库（Linux），即使运行时不需要
2. **二进制体积**：CLI 二进制包含 Tauri runtime 代码（虽然 strip 了 symbols）
3. **代码污染**：CLI 不能调用的命令（如 `set_window_theme`、`update_tray_menu`）仍存在于 CLI 二进制中
4. **测试隔离困难**：CLI 测试会触发 Tauri 命令的编译，无法独立测试纯业务逻辑

#### 评估文档的处理

评估文档把"webkit2gtk 构建依赖"列为"已知限制"第 4 条，描述为"运行时不需要"。这是**回避根因**——真正的问题是 lib crate 没有分层，而非"CLI 顺便编译了 GUI 代码"。

### 6.3 问题 2：State 管理割裂导致 CLI 命令只能用 Database 直接调用（P1 架构问题）

#### 现象

stream-check / stream-check-all 在 CLI 中是桩实现（§五已述）。代码 [cc-switch-cli.rs:3995](file:///f:/workspace/trae/cc-switch/src-tauri/src/bin/cc-switch-cli.rs#L3995) 给出的理由是：

> 流式检查需要代理服务器运行中且 CopilotAuthState 初始化，当前 CLI 环境不支持

#### 根因分析

**lib.rs 第 1005 行**：

```rust
app.manage(CopilotAuthState(Arc::new(RwLock::new(copilot_auth_manager))));
```

`CopilotAuthState` 是 Tauri 的 managed state，只能通过 `app.manage()` 注册，通过 `tauri::State<'_, CopilotAuthState>` 注入。

**commands/stream_check.rs:20** 流式检查命令签名：

```rust
copilot_state: State<'_, CopilotAuthState>,
```

**CLI 侧** [cc-switch-cli.rs:4119](file:///f:/workspace/trae/cc-switch/src-tauri/src/bin/cc-switch-cli.rs#L4119) 的 `init_db()` 只初始化 Database，**不创建 CopilotAuthState**。CLI 没有 Tauri AppHandle，无法 `app.manage()`。

#### 后果

1. **流式检查不可用**：CLI 无法注入 `CopilotAuthState`，导致 4 个桩命令中的 2 个
2. **未来扩展受阻**：任何依赖 Tauri managed state 的 GUI 命令都无法在 CLI 复用
3. **业务逻辑与 Tauri 绑定**：`stream_check` 业务逻辑写死在 `#[tauri::command]` 函数里，没有独立的 service 层

#### 评估文档的处理

评估文档把 stream-check 列为 OPT-029~031（"可实现"），声称 Phase 3 ✅ 完成。但实际架构上**不可能在 CLI 实现**，除非重构 service 层。这是**架构限制被掩盖为"已实现"**。

### 6.4 问题 3：apply-config 半应用的架构原因（P0 架构问题）

#### 现象（§五已述）

`decl_config.rs:159` 的 `apply()` 方法只应用 5 项，跳过 `proxy.listen/port/takeover`。

#### 根因分析

`proxy.listen` 和 `proxy.port` 实际通过环境变量 `CC_SWITCH_LISTEN` / `CC_SWITCH_PORT` 读取（参考手册 396~407 行），不在数据库里。`takeover` 状态由 `takeover on/off` 命令管理，写入 Live 配置文件，也不在数据库。

**`apply()` 方法只接收 `&Database` 参数** [decl_config.rs:159](file:///f:/workspace/trae/cc-switch/src-tauri/src/core/decl_config.rs#L159)：

```rust
pub fn apply(&self, db: &crate::database::Database) -> Result<String, String>
```

它无法写环境变量（进程已启动后环境变量不可改），也无法触发 `takeover` 命令（需要 ProxyService + tokio runtime）。

#### 后果

1. **声明式配置的"代理"字段是死代码**：解析了、校验了，但永远不生效
2. **用户被误导**：YAML 看起来能配置，实际不能
3. **架构上的正确做法未实现**：应该让 `apply()` 接收一个 "ApplyContext" 包含 db + proxy_service + settings_writer，分别应用各类配置

### 6.5 问题 4：daemon 模式的 worker 缺失（P2 架构问题）

#### 现象

参考手册"已知限制"第 1 条：

> WebDAV/S3 自动同步：daemon 模式下不启动自动同步 worker（需要 GUI 的 AppHandle）

#### 根因分析

`webdav_auto_sync` / `s3_auto_sync` service 依赖 Tauri AppHandle 来触发前端通知、读取 Tauri Store 等。CLI daemon 没有 AppHandle，无法启动这些 worker。

#### 后果

1. **CLI daemon 不等于 GUI daemon**：用户期望 daemon 是"无头版 GUI"，实际功能缩水
2. **云同步被列为 N/A**：但根因不是"非核心"，而是"架构不支持"

#### 评估文档的处理

评估文档把云同步列为 N/A-001/002，理由是"非代理核心"。但真实理由是"架构上 CLI daemon 无法启动 worker"。这是**架构限制被包装为主动选择**。

### 6.6 架构问题汇总

| 问题 | 严重性 | 根因 | 评估文档处理 |
|---|---|---|---|
| 1. lib crate 强耦合 Tauri | P1 | `pub use commands::*` 暴露全部 Tauri 命令 | 列为"已知限制"，未分析根因 |
| 2. State 管理割裂 | P1 | `CopilotAuthState` 等 managed state 无法在 CLI 创建 | 掩盖为"OPT 已实现" |
| 3. apply-config 半应用 | P0 | `apply()` 只接收 db，无法写环境变量/触发 takeover | 文档小字提示，未列入评估 |
| 4. daemon worker 缺失 | P2 | worker 依赖 AppHandle | 包装为"非核心 N/A" |

### 6.7 架构重构建议

#### 建议 1：lib crate 分层（解决问题 1、2）

将 `cc_switch_lib` 拆为三个 crate：

```
crates/
  cc-switch-core       ← 纯业务逻辑（Database、ProxyService、Provider 等）
  cc-switch-tauri-commands  ← #[tauri::command] 包装层，依赖 core
  cc-switch-lib        ← 兼容层，re-export 两者（保持外部 API 不变）
```

CLI 只依赖 `cc-switch-core`，GUI 依赖 `cc-switch-lib`。这样：
- CLI 编译不需要 webkit2gtk
- Tauri commands 隔离在单独 crate
- 业务逻辑可独立测试

#### 建议 2：service 层去 Tauri 化（解决问题 2）

`CopilotAuthState` 应改为 `Arc<RwLock<CopilotAuthManager>>`，由 service 层持有，Tauri command 包装层注入。CLI 可以直接创建 service 实例。

#### 建议 3：apply-config 接收 ApplyContext（解决问题 3）

```rust
pub struct ApplyContext<'a> {
    db: &'a Database,
    proxy_service: Option<&'a ProxyService>,
    env_writer: Option<&'a dyn EnvWriter>,
}

impl DeclConfig {
    pub fn apply(&self, ctx: &ApplyContext) -> Result<String, String> { ... }
}
```

CLI 传 `proxy_service: None`，GUI 传完整 ctx。YAML 字段标注"需要 proxy_service"才应用。

#### 建议 4：daemon worker 去 AppHandle 化（解决问题 4）

worker 通过 trait callback 通知前端，CLI 实现空 callback，GUI 实现 Tauri emit。这样 CLI daemon 也能跑 worker。

---

## 七、改进建议清单

### 7.1 P0 阻塞性修复（必须先做）

#### P0-1：修正评估文档"全部完成 ✅"声称

- **来源**：§五.2 问题 1、§五.4
- **动作**：将 `cli-feature-implementation-assessment.md` 第 9 行改为：

  ```
  实现状态：Phase 1 基本完成（apply-config 代理字段待补）、
            Phase 2 完成、Phase 3 部分完成（4 个桩命令待实现/删除）
  ```

- **估时**：S（10 分钟）
- **依赖**：无

#### P0-2：apply-config 代理字段处理

- **来源**：§五.2 问题 2、§六.4
- **动作**（二选一）：

  **方案 A（推荐，长期）：实现完整应用**
  - 修改 [decl_config.rs:159](file:///f:/workspace/trae/cc-switch/src-tauri/src/core/decl_config.rs#L159) `apply()` 签名为接收 `ApplyContext`
  - CLI 传 `proxy_service: None`，apply 时对代理字段写日志"需手动设置"
  - GUI 传完整 ctx，apply 时真正应用
  - 在 YAML schema 标注哪些字段需要 ctx.proxy_service

  **方案 B（短期止血）：从 schema 删除字段**
  - `DeclConfig::ProxySection` 只保留 `takeover`（可应用），删除 `listen` / `port`
  - 更新参考手册 1338~1385 行的 YAML 示例
  - 收敛范围但失去自动化能力

- **推荐路径**：先 B 止血、后续重构时再升级为 A
- **估时**：方案 A = M（半天），方案 B = S（1 小时）
- **依赖**：方案 A 与 P1-1 重构协同

#### P0-3：处理 4 个桩命令

- **来源**：§五.2 问题 1
- **动作**（按命令分别处理）：

  | 命令 | 处理 |
  |---|---|
  | `stream-check` / `stream-check-all` | 从 CLI 命令枚举删除（依赖 CopilotAuthState，架构上不可行，详见 §六.3）；在参考手册 1907~1922 行改为"GUI 专属"标注 |
  | `remove-session` | 补全 `--provider-id` / `--source-path` 参数；或从 CLI 删除并标注"GUI 专属" |

- **估时**：S（1 小时）
- **依赖**：无

#### P0-4：修复 add-provider env 字段硬编码

- **来源**：§五.2 问题 4
- **动作**：[cc-switch-cli.rs:1180](file:///f:/workspace/trae/cc-switch/src-tauri/src/bin/cc-switch-cli.rs#L1180) 按 `--app` 选择正确 env 名：

  ```rust
  let (key_field, url_field) = match app {
      "claude" | "claude-desktop" => ("ANTHROPIC_API_KEY", "ANTHROPIC_BASE_URL"),
      "codex" => ("OPENAI_API_KEY", "OPENAI_BASE_URL"),
      "gemini" => ("GEMINI_API_KEY", "GEMINI_BASE_URL"),
      "opencode" => ("OPENAI_API_KEY", "OPENAI_BASE_URL"),
      "openclaw" => ("OPENCLAW_API_KEY", "OPENCLAW_BASE_URL"),  // 按实际确认
      "hermes" => ("HERMES_API_KEY", "HERMES_BASE_URL"),        // 按实际确认
      _ => return Err("unsupported app".into()),
  };
  ```

- **估时**：S（1 小时）
- **依赖**：需确认 OpenClaw/Hermes 实际 env 名

### 7.2 P1 重要修复（核心标准未满足）

#### P1-1：lib crate 分层重构

- **来源**：§六.2、§六.3
- **动作**：拆分 `cc_switch_lib` 为三层：

  ```
  crates/
    cc-switch-core/          ← 纯业务逻辑（无 Tauri 依赖）
      src/
        database/
        services/
        proxy/
        core/
        lib.rs
    cc-switch-tauri-commands/  ← #[tauri::command] 包装层
      src/
        commands/             ← 从 src-tauri/src/commands/ 迁移
        lib.rs
      depends: cc-switch-core
    cc-switch-app/            ← 原 src-tauri，GUI 二进制
      src/
        lib.rs                ← Tauri Builder、tray、window
        main.rs
      depends: cc-switch-tauri-commands, cc-switch-core
  ```

  - `cc-switch-cli` 只依赖 `cc-switch-core`
  - Tauri commands 调用 core service，不直接持 managed state
  - `CopilotAuthState` 改为 `Arc<RwLock<CopilotAuthManager>>`，由 core service 持有

- **估时**：L（2~3 天）
- **依赖**：P0-2（apply-config 重构同步进行）

#### P1-2：API 格式设置覆盖 7 应用（REQ-020）

- **来源**：§五.2 问题 3
- **动作**：
  1. 确认 opencode/openclaw/hermes 支持的 API 格式
  2. 更新 [cc-switch-cli.rs:78~84](file:///f:/workspace/trae/cc-switch/src-tauri/src/bin/cc-switch-cli.rs#L78) 注释
  3. 更新参考手册 643~648 行表格
  4. 如有应用不支持任何格式转换（仅原生），文档明确说明

- **估时**：S（2 小时）
- **依赖**：无

#### P1-3：service 层去 Tauri 化

- **来源**：§六.3、§六.7 建议 2
- **动作**：
  - `stream_check` 业务逻辑从 `commands/stream_check.rs` 抽到 `services/stream_check.rs`
  - service 接收 `&CopilotAuthManager` 而非 `State<'_, CopilotAuthState>`
  - Tauri command 包装层负责从 managed state 取出 manager 调 service
  - CLI 创建独立 `CopilotAuthManager` 实例调 service

- **估时**：M（半天）
- **依赖**：P1-1 完成后

#### P1-4：评估文档分类调整

- **来源**：§三.5、§四.6
- **动作**：按 §三.5 表格更新 `cli-feature-implementation-assessment.md`：
  - REQ-007/008/013 降级 OPT
  - OpenClaw 048/049、Hermes 056 升级 REQ
  - OPT-017~021（Skills 仓库）归 N/A
  - OPT-047~060 拆为"应用专属核心"和"应用自身功能"
  - 新增 REQ-020（API 格式覆盖）、REQ-021（协议转换烟雾测试）

- **估时**：S（1 小时）
- **依赖**：无

### 7.3 P2 文档与体验改进

#### P2-1：参考手册补全说明

- **来源**：§五.3、§五.5
- **动作**：
  - 参考手册 1488~1530 行（speedtest/verify-key）补注"不依赖代理运行"
  - 参考手册 643 行 API 格式表格补全 7 应用
  - list-providers 输出 Base URL 时补全 `OPENAI_BASE_URL` / `GEMINI_BASE_URL` 读取路径（[cc-switch-cli.rs:1138](file:///f:/workspace/trae/cc-switch/src-tauri/src/bin/cc-switch-cli.rs#L1138)）

- **估时**：S
- **依赖**：P1-2 完成

#### P2-2：daemon worker 去 AppHandle 化

- **来源**：§六.5、§六.7 建议 4
- **动作**：webdav_auto_sync / s3_auto_sync service 通过 trait callback 通知，CLI 实现空 callback
- **估时**：M
- **依赖**：P1-1 完成后

#### P2-3：OPT 子优先级标注

- **来源**：§四.5
- **动作**：评估文档 OPT 列表加一列"子优先级"（A/B），按 §四.6 清单标注
- **估时**：S
- **依赖**：P1-4 完成

### 7.4 修复路线图（推荐顺序）

```
第 1 周（P0 文档与阻塞性 bug）
  Day 1: P0-1 修评估文档声称 + P0-3 处理桩命令（删/补参数）
  Day 2: P0-4 修 add-provider env 硬编码
  Day 3: P0-2 apply-config 方案 B（先删字段止血）
  Day 4-5: P1-2 API 格式覆盖 + P1-4 评估文档分类调整

第 2~3 周（P1 架构重构）
  Day 1-3: P1-1 lib crate 分层
  Day 4-5: P1-3 service 去 Tauri 化
  Day 6-7: P0-2 apply-config 方案 A（补完整应用）

第 4 周（P2 体验完善）
  P2-1 文档补全 + P2-3 OPT 子优先级
  P2-2 daemon worker（如有需要）
```

### 7.5 修复后预期效果

| 维度 | 当前 | 修复后 |
|---|---|---|
| 评估文档声称准确性 | "全部完成 ✅"（不准确） | "Phase 2 完成、Phase 1 基本完成、Phase 3 部分完成"（准确） |
| CLI 构建依赖 | 需 webkit2gtk / GTK | 仅 Rust 标准库 + axum + reqwest |
| 核心标准 A 协议转换 | 部分覆盖 | 7 应用全覆盖 + 烟雾测试 |
| 核心标准 B 供应商配置 | env 硬编码 bug | 7 应用正确 env 名 |
| 核心标准 C API 格式 | 4 应用支持 | 7 应用支持（或明确不支持） |
| apply-config | 静默失败 | 完整应用或字段移除 |
| REQ 数量 | 19（含 3 误分类） | 21（精准对齐核心标准） |
| OPT 数量 | 60（无优先级） | 12 OPT-A + 16 OPT-B（清晰） |

### 7.6 关键结论

1. **P0 共 4 项**：文档声称、apply-config、桩命令、env 硬编码 —— 1 周内可完成
2. **P1 共 4 项**：lib 分层、API 格式、service 去 Tauri、分类调整 —— 2~3 周
3. **P2 共 3 项**：文档补全、daemon worker、OPT 子优先级 —— 1 周
4. **关键路径**：P1-1（lib 分层）是后续多项修复的基础，应优先排期

---

## 八、附录

### 8.1 评审方法

- 阅读两份目标文档全文
- 抽样核对 `src-tauri/src/bin/cc-switch-cli.rs`、`src-tauri/src/lib.rs`、`src-tauri/src/commands/mod.rs`、`src-tauri/src/core/decl_config.rs`、`src-tauri/Cargo.toml`
- 按"代理能否跑、协议能否转、模型能否配"的判别标准逐项评估
- 与用户确认核心标准（A 协议转换 + B 供应商模型配置 + C API 格式设置 + D CLI/GUI 等价）

### 8.2 评审维度结论汇总

| 维度 | 评级 | 核心问题 |
|------|------|----------|
| 分类合理性 | 良好但有改进空间 | REQ-007/008/013 偏激进；OpenClaw/Hermes 模型配置应升 REQ |
| 优先级与范围 | 中等 | REQ 优先级基本合理；60 项 OPT 严重 YAGNI |
| 实现与文档一致性 | **不及格** | "全部完成 ✅" 与 4 处桩实现/部分实现冲突 |
| 架构与依赖 | **不及格** | lib crate 未分层，4 个架构问题被包装为"已知限制" |

### 8.3 修订后 REQ 完整清单（23 项）

| 编号 | 功能 | 阶段 | 来源 |
|---|---|---|---|
| REQ-001 | 供应商排序 | Phase 1 | 原 REQ |
| REQ-002 | 从 Live 配置导入 | Phase 1 | 原 REQ（含 OPT-055 合并） |
| REQ-003 | 读取 Live 配置 | Phase 1 | 原 REQ |
| REQ-004 | 模型列表获取 | Phase 1 | 原 REQ |
| REQ-005 | 同步到 Live | Phase 1 | 原 REQ |
| REQ-006 | 代理配置读写（含 6a 基础 listen/port/retry + 6b 应用级 enabled/auto_failover） | Phase 1 (6a) / Phase 2 (6b) | 原 REQ-006 拆分 |
| REQ-009 | Live 接管检测 | Phase 1（上移） | 原 REQ |
| REQ-010 | 熔断器统计 | Phase 2 | 原 REQ |
| REQ-011 | 供应商健康状态 | Phase 2 | 原 REQ |
| REQ-012 | 可用故障转移列表 | Phase 2 | 原 REQ |
| REQ-014 | 按应用统计用量 | Phase 2 | 原 REQ（澄清"仅 summary"） |
| REQ-015 | 请求日志查看 | Phase 2 | 原 REQ |
| REQ-016 | 供应商限额检查 | Phase 2 | 原 REQ（澄清"仅检查不阻断"） |
| REQ-017 | 备份删除 | Phase 2 | 原 REQ |
| REQ-018 | 备份重命名 | Phase 2 | 原 REQ |
| REQ-019 | 自定义端点管理 | Phase 2 | 原 REQ |
| REQ-020 | API 格式设置覆盖 7 应用 | Phase 1 | 新增 |
| REQ-021 | 协议转换烟雾测试 | Phase 1 | 新增 |
| REQ-022 | 代理热重载 | Phase 1 | §九 M-1 新增 |
| REQ-023 | 代理访问控制（auth-token + acl） | Phase 1 | §九 M-2 新增 |
| REQ-OC-01 | OpenClaw 默认模型 | Phase 2 | OPT-048 升级 |
| REQ-OC-02 | OpenClaw 模型目录 | Phase 2 | OPT-049 升级 |
| REQ-HE-01 | Hermes 模型配置查看 | Phase 2 | OPT-056 升级 |

### 8.4 修订后 N/A 新增清单（14 项）

| 编号 | 功能 | 原 OPT | 理由 |
|---|---|---|---|
| N/A-021 | 工具版本检测 | OPT-028 | `which` 命令替代 |
| N/A-022 | 供应商复制 | OPT-041 | GUI 内部实现 |
| N/A-023 | Codex 历史迁移 | OPT-046 | 一次性操作 |
| N/A-024 | Skills 仓库浏览 | OPT-017 | GUI 商店功能 |
| N/A-025 | Skills 更新检查 | OPT-018 | 同上 |
| N/A-026 | Skills 仓库源管理 | OPT-019 | 同上 |
| N/A-027 | Skills ZIP 批量安装 | OPT-020 | 同上 |
| N/A-028 | Skills 备份恢复 | OPT-021 | 同上 |
| N/A-029 | OpenClaw 工作区文件 | OPT-053 | 应用自身功能 |
| N/A-030 | OpenClaw 每日记忆 | OPT-054 | 应用自身功能 |
| N/A-031 | Hermes 记忆管理 | OPT-057 | 应用自身功能 |
| N/A-032 | OMO 配置读取 | OPT-058 | 非核心场景 |
| N/A-033 | OMO 停用 | OPT-059 | 同上 |
| N/A-034 | OMO Slim 管理 | OPT-060 | 同上 |

---

## 九、软件功能缺失项评审

### 9.1 评审标尺

按核心标准 A/B/C/D 与"代理软件应有的能力"对比当前 CLI 命令清单（[cc-switch-cli.rs:4020~4112](file:///f:/workspace/trae/cc-switch/src-tauri/src/bin/cc-switch-cli.rs#L4020)），找出尚未覆盖的功能。

**核心场景确认**：用户"调任意模型"= 任意 APP 通过代理调任意后端 API 格式（openai_chat / openai_responses / anthropic / gemini_native / bedrock）。此能力已由 REQ-020（API 格式设置覆盖 7 应用）覆盖，**模型路由/别名不作为缺失项**。

### 9.2 缺失项清单

#### 🔴 P0 严重缺失（无头部署刚需）

##### M-1：代理热重载

- **缺失命令**：`reload`
- **现状**：`switch-proxy` 是热切换供应商，但变更 failover-queue、circuit-breaker、api-format 等配置后必须 `stop` + `start`，会中断活跃连接
- **严重性**：无头服务器场景下配置变更频繁，每次重启中断连接体验差，与 CLI 无头定位冲突
- **关联核心标准**：D（CLI/GUI 等价 — CLI 应至少与 GUI 重启持平）
- **建议归类**：新增 REQ-022

##### M-2：代理访问控制

- **缺失命令**：`auth-token <set|clear>`、`acl <list|add|remove> --cidr C`
- **现状**：代理监听 `0.0.0.0` 时（远程部署场景），任何能访问端口的人都能用
- **严重性**：无头服务器部署的安全刚需。当前代理完全开放，任何人扫到端口即可白嫖
- **关联核心标准**：无头部署安全刚需（非 A/B/C，但属 CLI 核心场景）
- **建议归类**：新增 REQ-023

#### 🟡 P1 中度缺失（核心能力补强）

##### M-3：协议转换可观测性

- **缺失命令**：
  - `proxy-trace <APP> --model M` — 跟踪一次请求的完整转换过程（请求 → 协议转换 → 转发 → 响应 → 反转换）
  - `replay-request <REQUEST_ID>` — 重放历史请求用于排障（已有 request-logs 但不能重放）
  - `preview-conversion --from F --to F --payload JSON` — 预览转换后的请求体（不入网）
- **现状**：只有 `verify-key`（验证 key）和 `speedtest`（测延迟），代理转换链路是黑盒
- **严重性**：协议转换出问题时用户无法定位是哪一步出错，是代理软件的核心排障能力
- **关联核心标准**：A（协议转换的可观测）
- **建议归类**：OPT-A

##### M-4：实时可观测性

- **缺失命令**：
  - `connections` — 查看当前活跃连接
  - `stats --live` — 实时统计（QPS / 延迟 p50/p99 / 错误率）
  - `logs --tail` — 实时查看代理日志（tail -f 模式）
- **现状**：`status` 只显示运行/停止，`request-logs` 是历史日志
- **严重性**：无头部署无 GUI 可看，CLI 是唯一观察窗口
- **关联核心标准**：D（CLI/GUI 等价）
- **建议归类**：OPT-A

##### M-5：配置 diff/rollback

- **缺失命令**：
  - `diff <PATH>` — 对比 YAML 与当前数据库配置的差异（apply 前预览变更）
  - `rollback` — 回滚到上一个 apply 前的状态（当前需手动 backup + restore）
- **现状**：`apply-config` 是单向不可逆，无预览
- **严重性**：声明式配置是 CLI 自动化部署核心场景，缺 diff/rollback 等于裸奔
- **关联核心标准**：D
- **建议归类**：OPT-A（与 P0-2 apply-config 重构协同）

##### M-6：导出为 YAML

- **缺失命令**：`export-yaml <PATH>`
- **现状**：`export-config` 只导出 SQL，不能导出为声明式 YAML
- **严重性**：配置即代码工作流的关键缺失。用户想版本控制配置时，SQL 不可读
- **关联核心标准**：D
- **建议归类**：OPT-A（apply-config 的逆操作）

##### M-7：供应商启用/禁用

- **缺失命令**：`toggle-provider <APP> <ID> <on|off>`
- **现状**：只能 `remove-provider` 删除，无法临时禁用
- **严重性**：供应商暂时不可用（限流/维护）时，想保留配置但暂停使用
- **关联核心标准**：B（供应商配置）
- **建议归类**：OPT-A

##### M-8：格式自动检测

- **缺失命令**：`detect-format --base-url U --api-key K`
- **现状**：用户配置 `--api-format` 时需要知道目标 API 支持什么格式，但这个信息往往不明确
- **严重性**：核心标准 C 的可用性补强
- **关联核心标准**：C
- **建议归类**：OPT-B

#### 🟢 P2 低优先级缺失

##### M-9：优雅停机

- **缺失命令**：`stop --grace SECONDS`
- **现状**：`stop` 是直接停，可能中断活跃请求
- **建议归类**：OPT-B

##### M-10：预算告警

- **缺失命令**：`budget <set|get|clear> <APP> [--amount V]`
- **现状**：`check-limits` 只查供应商限额，不查用户预算
- **建议归类**：OPT-B

##### M-11：多 profile 管理

- **缺失命令**：`profile <list|switch|create|delete> <NAME>`
- **现状**：只能用 `CC_SWITCH_HOME` 环境变量切换实例，无 CLI 管理
- **建议归类**：OPT-B

### 9.3 缺失项汇总表

| 编号 | 缺失功能 | 严重性 | 建议归类 | 关联核心标准 |
|---|---|---|---|---|
| M-1 | 代理热重载 | 🔴 P0 | REQ-022 | D |
| M-2 | 代理访问控制（auth-token + acl） | 🔴 P0 | REQ-023 | 安全刚需 |
| M-3 | 协议转换可观测性（trace/replay/preview） | 🟡 P1 | OPT-A | A |
| M-4 | 实时可观测性（connections/stats/logs） | 🟡 P1 | OPT-A | D |
| M-5 | 配置 diff/rollback | 🟡 P1 | OPT-A | D |
| M-6 | 导出为 YAML | 🟡 P1 | OPT-A | D |
| M-7 | 供应商启用/禁用 | 🟡 P1 | OPT-A | B |
| M-8 | 格式自动检测 | 🟡 P1 | OPT-B | C |
| M-9 | 优雅停机 | 🟢 P2 | OPT-B | D |
| M-10 | 预算告警 | 🟢 P2 | OPT-B | 用量管理 |
| M-11 | 多 profile | 🟢 P2 | OPT-B | 多实例 |

### 9.4 修订后 REQ 总数

| 来源 | 数量 |
|---|---|
| §三.6 原 REQ | 21 |
| M-1 升入 REQ | +1（REQ-022） |
| M-2 升入 REQ | +1（REQ-023） |
| **修订后 REQ 总数** | **23** |

### 9.5 关键判断

**三个最严重缺失**：

1. **代理热重载（M-1）**：无头服务器场景下配置变更频繁，每次重启中断连接，与 CLI 无头定位冲突
2. **代理访问控制（M-2）**：监听 0.0.0.0 时完全开放，无头服务器部署有被盗用风险
3. **协议转换可观测性（M-3）**：协议转换链路黑盒，出问题无法定位

**已撤回的项**：

- ~~模型路由~~：核心场景是协议转换的格式覆盖，已由 REQ-020 体现，不是按模型路由
- ~~模型别名~~：同上
- ~~模型清单管理~~：核心是格式转换，模型级配置非核心

### 9.6 对修复路线图的影响

新增 P0 项需补入 §七.4 修复路线图：

```
第 1 周（P0 文档与阻塞性 bug）— 不变
第 2~3 周（P1 架构重构）— 增加 M-1/M-2 的设计与实现
  Day 1-3: P1-1 lib crate 分层
  Day 4-5: P1-3 service 去 Tauri 化
  Day 6-7: P0-2 apply-config 方案 A
  Day 8-10: M-1 热重载 + M-2 访问控制（新增）
第 4 周（P2 体验完善）— 增加 M-3~M-8 的 OPT-A 实现
  P2-1 文档补全 + P2-3 OPT 子优先级
  M-3~M-7 OPT-A 实现（新增）
  M-8/M-9/M-10/M-11 OPT-B 视情况
```
