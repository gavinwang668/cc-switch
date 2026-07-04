# CC Switch CLI 完整命令参考手册

> 文档版本：v2.1 ｜ 更新日期：2026-07-04
>
> 本文档是 `cc-switch-cli` 命令行工具的完整参考手册，涵盖所有命令、参数、选项、使用示例及故障排查指南。

---

## 目录

- [编译与安装](#编译与安装)
- [CLI 与 GUI 的关系](#cli-与-gui-的关系)
- [Windows 使用指南](#windows-使用指南)
- [Linux 使用指南](#linux-使用指南)
- [全局选项](#全局选项)
- [环境变量](#环境变量)
- [信号处理](#信号处理)
- [命令总览](#命令总览)
- [代理管理](#代理管理)
- [供应商管理](#供应商管理)
- [代理配置](#代理配置)
- [故障转移与熔断器](#故障转移与熔断器)
- [请求处理配置](#请求处理配置)
- [全局出站代理](#全局出站代理)
- [代理核心能力](#代理核心能力)
- [配置与设置](#配置与设置)
- [声明式配置](#声明式配置)
- [备份与恢复](#备份与恢复)
- [用量统计与监控](#用量统计与监控)
- [测试与诊断](#测试与诊断)
- [MCP 与 Prompts](#mcp-与-prompts)
- [常见问题与故障排查](#常见问题与故障排查)
- [已知限制](#已知限制)
- [附录](#附录)

---

## 编译与安装

### 获取二进制文件

#### 方式一：使用预编译二进制

从项目 `release/` 目录获取已编译好的二进制：

| 文件 | 平台 | 说明 |
|---|---|---|
| `cc-switch.exe` | Windows | GUI 桌面应用 |
| `cc-switch-cli.exe` | Windows | CLI 工具 |
| `cc-switch-cli-linux-x86_64` | Linux x86_64 | CLI 工具 |

#### 方式二：从源码编译

**Windows 编译**

前置要求：Node.js 22+、pnpm 8+、Rust 1.85+、Visual Studio C++ Build Tools。

```powershell
# 1. 安装前端依赖
cd f:\workspace\trae\cc-switch
pnpm install

# 2. 构建前端
pnpm exec vite build

# 3. 编译 CLI
cd src-tauri
cargo build --release --bin cc-switch-cli

# 4. 编译 GUI（可选）
cargo build --release --bin cc-switch
```

产物路径：`src-tauri\target\release\cc-switch-cli.exe` 和 `cc-switch.exe`

**Linux 编译（WSL 或原生 Linux）**

前置要求：Rust 1.85+、`libwebkit2gtk-4.1-dev`、`libgtk-3-dev`、`libayatana-appindicator3-dev`。

```bash
# 安装系统依赖（Ubuntu/Debian）
sudo apt install -y libwebkit2gtk-4.1-dev libgtk-3-dev libayatana-appindicator3-dev

# 加载 Rust 环境
source ~/.cargo/env

# 进入项目目录
cd /path/to/cc-switch/src-tauri

# 确保前端已构建（dist 目录存在）
# 如果从 Windows 挂载的目录，dist 已存在则跳过

# 编译 CLI
cargo build --release --bin cc-switch-cli
```

产物路径：`src-tauri/target/release/cc-switch-cli`

> **注意**：即使只编译 CLI，当前仍需要 webkit2gtk 开发库，因为 CLI 和 GUI 共享同一个 lib crate。运行时不需要 webkit2gtk。

### systemd 部署（推荐生产环境）

```bash
# 复制二进制
sudo cp target/release/cc-switch-cli /usr/bin/cc-switch-cli

# 创建配置目录
sudo mkdir -p /etc/cc-switch
sudo cp deploy/config.example.yaml /etc/cc-switch/config.yaml

# 创建专用用户
sudo useradd -r -s /sbin/nologin ccswitch

# 安装 systemd 服务
sudo cp cc-switch.service /etc/systemd/system/
sudo systemctl daemon-reload
sudo systemctl enable cc-switch
sudo systemctl start cc-switch

# 查看状态和日志
sudo systemctl status cc-switch
sudo journalctl -u cc-switch -f
```

> **注意**：`deploy/cc-switch.service` 文件需提前存在。如果项目根目录的 `cc-switch.service` 被移动，请相应调整路径。

---

## CLI 与 GUI 的关系

CC Switch 同时提供 GUI 桌面应用（`cc-switch`）和 CLI 工具（`cc-switch-cli`）。两者共享相同的后端逻辑和数据库（`~/.cc-switch/cc-switch.db`），可以交替使用：

- 在 GUI 中添加的供应商，CLI 也能看到。
- CLI 切换的供应商，GUI 下次启动时也会反映。
- 可以用 GUI 完成初始配置，再用 CLI 做脚本化管理。

> **注意**：不建议同时运行 GUI 的代理和 CLI 的 `daemon`，否则会出现端口冲突。选择一个入口启动代理即可。

---

## Windows 使用指南

### 快速开始

将 `cc-switch-cli.exe` 放到任意目录（如 `C:\Tools\cc-switch\`），然后将其加入 PATH 或直接使用完整路径。

```powershell
# 查看帮助
cc-switch-cli.exe help

# 查看当前状态
cc-switch-cli.exe status

# 列出供应商
cc-switch-cli.exe list-providers
```

### 前台运行代理

```powershell
# 启动代理（前台运行，Ctrl+C 停止）
cc-switch-cli.exe start

# 自定义监听地址和端口
$env:CC_SWITCH_LISTEN = "0.0.0.0"
$env:CC_SWITCH_PORT = "8080"
cc-switch-cli.exe start
```

### 后台运行代理

```powershell
# 以守护进程方式后台运行
cc-switch-cli.exe daemon
# 输出：代理服务器已在后台启动 (PID: 12345)
# 输出：日志文件: C:\Users\YourName\.cc-switch\cc-switch-daemon.log

# 查看状态
cc-switch-cli.exe status

# 停止后台代理
cc-switch-cli.exe stop
```

### 日常管理

```powershell
# 添加供应商
cc-switch-cli.exe add-provider claude my-provider "My Provider" `
  --api-key sk-ant-xxx `
  --base-url https://api.anthropic.com

# 切换当前供应商
cc-switch-cli.exe switch-provider claude my-provider

# 开启代理接管 Claude
cc-switch-cli.exe takeover claude on

# 代理模式下热切换供应商
cc-switch-cli.exe switch-proxy claude backup-provider
```

### 配合 GUI 使用

Windows 上可以同时安装 GUI 应用和 CLI 工具。两者共享同一个 `~/.cc-switch/` 配置目录，操作完全互通：

- 在 GUI 中添加的供应商，CLI 也能看到。
- CLI 切换的供应商，GUI 重启后也会反映。
- 可以用 GUI 做初始配置，然后用 CLI 做自动化管理。

### 开机自启（可选）

使用任务计划程序设置开机自动启动 daemon：

```powershell
# 创建开机自启任务
$action = New-ScheduledTaskAction -Execute "C:\Tools\cc-switch\cc-switch-cli.exe" -Argument "daemon"
$trigger = New-ScheduledTaskTrigger -AtLogon
Register-ScheduledTask -TaskName "CC Switch" -Action $action -Trigger $trigger
```

---

## Linux 使用指南

### 安装

```bash
# 复制二进制到系统路径
sudo cp cc-switch-cli-linux-x86_64 /usr/local/bin/cc-switch-cli
sudo chmod +x /usr/local/bin/cc-switch-cli

# 创建专用用户（推荐，不使用 root 运行）
sudo useradd -r -s /sbin/nologin ccswitch

# 创建配置目录
sudo mkdir -p /home/ccswitch/.cc-switch
sudo chown ccswitch:ccswitch /home/ccswitch/.cc-switch
```

### 使用 systemd 管理服务

项目提供 systemd unit 模板（`cc-switch.service`），按以下步骤部署：

```bash
# 复制 service 文件
sudo cp cc-switch.service /etc/systemd/system/

# 根据需要编辑配置
sudo vim /etc/systemd/system/cc-switch.service
```

参考的 service 文件内容：

```ini
[Unit]
Description=CC Switch Headless Proxy Server
After=network-online.target
Wants=network-online.target

[Service]
Type=simple
ExecStart=/usr/local/bin/cc-switch-cli daemon
KillSignal=SIGTERM
TimeoutStopSec=10
Restart=on-failure
RestartSec=5
User=ccswitch
Group=ccswitch
WorkingDirectory=/home/ccswitch
StandardOutput=journal
StandardError=journal
SyslogIdentifier=cc-switch

[Install]
WantedBy=multi-user.target
```

启用和管理：

```bash
# 加载 service 配置
sudo systemctl daemon-reload

# 设置开机自启
sudo systemctl enable cc-switch

# 启动服务
sudo systemctl start cc-switch

# 查看状态
sudo systemctl status cc-switch

# 查看日志
sudo journalctl -u cc-switch -f

# 重启服务
sudo systemctl restart cc-switch

# 停止服务
sudo systemctl stop cc-switch
```

### 日常管理

服务运行后，使用 CLI 命令管理配置。这些命令不需要 root 权限（只要能访问配置目录）：

```bash
# 查看代理状态
cc-switch-cli status

# 列出所有供应商
cc-switch-cli list-providers

# 添加供应商
cc-switch-cli add-provider claude my-provider "My Provider" \
  --api-key sk-ant-xxx \
  --base-url https://api.anthropic.com

# 切换供应商
cc-switch-cli switch-provider claude my-provider

# 开启 Claude 代理接管
cc-switch-cli takeover claude on

# 设置自动故障转移
cc-switch-cli auto-failover claude on
cc-switch-cli failover-queue add claude backup-provider
```

### 让 CLI 指向正确的配置

默认情况下 CLI 读写 `~/.cc-switch/` 目录。如果 service 以 `ccswitch` 用户运行，配置目录为 `/home/ccswitch/.cc-switch/`。管理命令也需要指向同一目录：

```bash
# 方式一：以 ccswitch 用户执行管理命令
sudo -u ccswitch cc-switch-cli list-providers

# 方式二：通过环境变量覆盖配置目录（暂不支持）
# 当前版本不支持环境变量覆盖，建议使用方式一
```

### 查看日志

`daemon` 模式下日志输出到两个地方：

```bash
# 方式一：journalctl（systemd 管理时）
sudo journalctl -u cc-switch -f

# 方式二：日志文件
tail -f /home/ccswitch/.cc-switch/cc-switch-daemon.log
```

### 配合 Claude Code / Codex / Gemini CLI 使用

代理服务器启动并接管后，各 CLI 工具的请求会自动经过代理。默认监听 `127.0.0.1:9090`，接管后会自动修改各 CLI 工具的 Live 配置文件（如 `~/.claude/settings.json`），将其 API 端点指向本地代理。无需手动修改 CLI 工具配置。

如果需要手动验证：

```bash
# Claude Code: 查看 settings.json 是否被接管
cat ~/.claude/settings.json
# 接管后应显示 base_url 指向 http://127.0.0.1:9090

# Codex: 查看 auth.json 和 config.toml
cat ~/.codex/auth.json
cat ~/.codex/config.toml

# Gemini: 查看 settings.json
cat ~/.gemini/settings.json
```

---

## 全局选项

| 选项 | 说明 | 默认值 |
|------|------|--------|
| `--log-level <LEVEL>` | 日志级别，可选值：error / warn / info / debug / trace | `info` |

该选项为全局选项，可放在任何子命令之前或之后。

```bash
# 设置 debug 级别日志
cc-switch-cli --log-level debug status

# 设置 trace 级别日志（最详细）
cc-switch-cli status --log-level trace
```

---

## 环境变量

| 变量 | 默认值 | 说明 |
|------|--------|------|
| `CC_SWITCH_LISTEN` | `127.0.0.1` | 代理服务器监听地址 |
| `CC_SWITCH_PORT` | `9090` | 代理服务器监听端口 |

```bash
# 监听所有网络接口
CC_SWITCH_LISTEN=0.0.0.0 cc-switch-cli start

# 使用自定义端口
CC_SWITCH_PORT=8080 cc-switch-cli daemon
```

---

## 信号处理

`daemon` 和 `start` 模式下支持以下信号（仅 Unix）：

| 信号 | 行为 |
|------|------|
| `SIGTERM` | 优雅停止代理，恢复 Live 配置，删除 PID 文件，退出 |
| `SIGINT` (Ctrl+C) | 同 SIGTERM |
| `SIGHUP` | 重载设备级配置（settings.json） |

Windows 仅支持 `Ctrl+C`（SIGINT 等效）。

---

## 命令总览

```
cc-switch-cli [OPTIONS] <COMMAND>

代理管理:
    start / daemon / stop / status

供应商管理:
    list-providers / add-provider / update-provider / remove-provider / switch-provider
    sort-providers / import-live / read-live / fetch-models / sync-live

代理配置:
    takeover / switch-proxy / proxy-config / global-proxy-config / app-proxy-config
    cost-multiplier / pricing-source / takeover-status

故障转移:
    failover-queue / auto-failover / circuit-breaker

请求处理:
    rectifier / optimizer

网络:
    global-proxy

配置:
    settings / config / export-config / import-config

声明式配置:
    validate / apply-config

备份:
    backup-create / backup-list / backup-restore / backup-delete / backup-rename

用量与测试:
    usage-summary / usage-by-app / usage-trends / provider-stats / model-stats
    speedtest / verify-key / check-limits

请求日志:
    request-logs / request-detail

代理监控:
    circuit-breaker-stats / provider-health / failover-available
    config-snippet / endpoint

MCP 管理:
    list-mcp / add-mcp / remove-mcp / toggle-mcp / test-mcp

Prompt 管理:
    list-prompts / add-prompt / remove-prompt / enable-prompt

Skills 管理:
    list-skills / remove-skill / toggle-skill

其他:
    check-env / list-sessions / smoke-test / help

代理安全与热重载:
    reload / auth-token / acl

诊断与测试:
    preview-conversion / proxy-trace / replay-request

配置管理增强:
    export-yaml / diff / rollback / toggle-provider
```

---

## 代理管理

### start — 前台启动代理

```bash
cc-switch-cli start
```

前台阻塞运行代理服务器，按 `Ctrl+C` 停止。适用于调试。

**参数**：

| 参数 | 说明 |
|------|------|
| `--internal-daemon` | 内部参数，由 `daemon` 命令自动传入，用户无需指定 |

**示例**：

```bash
# 前台启动，监听 9090 端口
cc-switch-cli start

# 指定监听地址和端口
CC_SWITCH_LISTEN=0.0.0.0 CC_SWITCH_PORT=8080 cc-switch-cli start

# 调试模式
cc-switch-cli --log-level debug start
```

### daemon — 后台守护进程启动

```bash
cc-switch-cli daemon
```

以守护进程方式后台启动代理服务器。自动创建 PID 文件（`~/.cc-switch/cc-switch-daemon.pid`）和日志文件（`~/.cc-switch/cc-switch-daemon.log`）。

**特性**：
- 自动检测已有实例运行，避免重复启动
- 发现残留 PID 文件（进程已退出）时自动清理
- Windows 使用 `DETACHED_PROCESS` 标志脱离控制台

**示例**：

```bash
# 后台启动
cc-switch-cli daemon
# 输出: 代理服务器已在后台启动 (PID: 12345)
# 输出: 日志文件: /home/user/.cc-switch/cc-switch-daemon.log

# 查看日志
tail -f ~/.cc-switch/cc-switch-daemon.log
```

### stop — 停止后台代理

```bash
cc-switch-cli stop
```

通过 HTTP POST `/stop` 通知后台代理服务器停止。等待最多 5 秒确认进程退出。

**示例**：

```bash
cc-switch-cli stop
# 输出: 已发送停止信号到代理服务器 127.0.0.1:9090
# 输出: 代理服务器已停止
```

### status — 查看代理状态

```bash
cc-switch-cli status
```

显示守护进程状态、代理服务运行状态（通过 HTTP 查询）、运行时间，以及全部 7 种应用的当前供应商。

**示例**：

```bash
cc-switch-cli status
# 输出:
# 代理服务器状态:
#   守护进程: 运行中 (PID: 12345)
#   代理服务: 运行中
#   监听地址: 127.0.0.1:9090
#   运行时间: 3600秒
#
# 当前供应商:
#   claude           : my-provider
#   claude-desktop   : (无)
#   codex            : openai-direct
#   gemini           : (无)
#   opencode         : (无)
#   openclaw         : (无)
#   hermes           : (无)
```

---

## 供应商管理

### list-providers — 列出供应商

```bash
cc-switch-cli list-providers [APP]
```

列出指定应用或全部应用的供应商。当前供应商标记为 `*`。

**参数**：

| 参数 | 必填 | 说明 |
|------|------|------|
| `APP` | 否 | 应用类型，不指定则列出全部。可选值：claude, claude-desktop, codex, gemini, opencode, openclaw, hermes |

**示例**：

```bash
# 列出全部应用的供应商
cc-switch-cli list-providers

# 仅列出 Claude 供应商
cc-switch-cli list-providers claude

# 仅列出 Codex 供应商
cc-switch-cli list-providers codex
```

**输出格式**：

```
── claude ─────────────────────────────────
  * my-provider         My Provider       anthropic          https://api.anthropic.com
    backup-provider     Backup            openai_chat        https://api.openai.com
```

### add-provider — 添加供应商

```bash
cc-switch-cli add-provider <APP> <ID> <NAME> [--api-key KEY] [--base-url URL] [--api-format FORMAT]
```

**参数**：

| 参数 | 必填 | 说明 |
|------|------|------|
| `APP` | 是 | 应用类型 |
| `ID` | 是 | 供应商唯一标识（如 `my-provider`） |
| `NAME` | 是 | 供应商显示名称 |
| `--api-key` | 否 | API Key |
| `--base-url` | 否 | Base URL |
| `--api-format` | 否 | API 格式（见下表） |

**API 格式说明**：

| 应用类型 | 支持的格式 |
|----------|-----------|
| claude | `anthropic` / `openai_chat` / `openai_responses` |
| codex | `openai_responses` / `openai_chat` |
| gemini | `gemini_native` / `openai_chat` / `openai_responses` / `anthropic` |
| claude-desktop | `anthropic` / `openai_chat` / `openai_responses` / `gemini_native` / `bedrock` |

**示例**：

```bash
# 添加 Anthropic 官方供应商
cc-switch-cli add-provider claude anthropic-direct "Anthropic Direct" \
  --api-key sk-ant-xxx \
  --base-url https://api.anthropic.com \
  --api-format anthropic

# 添加 OpenAI 兼容供应商
cc-switch-cli add-provider claude openai-proxy "OpenAI Proxy" \
  --api-key sk-xxx \
  --base-url https://api.openai.com \
  --api-format openai_chat

# 添加不带 API Key 的供应商（稍后通过 update-provider 设置）
cc-switch-cli add-provider codex my-codex "My Codex Provider"
```

### update-provider — 更新供应商

```bash
cc-switch-cli update-provider <APP> <ID> [--name NAME] [--api-key KEY] [--base-url URL] [--api-format FORMAT] [--clear-api-format]
```

**参数**：

| 参数 | 必填 | 说明 |
|------|------|------|
| `APP` | 是 | 应用类型 |
| `ID` | 是 | 供应商 ID |
| `--name` | 否 | 新名称 |
| `--api-key` | 否 | 新 API Key |
| `--base-url` | 否 | 新 Base URL |
| `--api-format` | 否 | 新 API 格式 |
| `--clear-api-format` | 否 | 清除 API 格式设置 |

**示例**：

```bash
# 更新 API Key
cc-switch-cli update-provider claude my-provider --api-key sk-new-xxx

# 更新名称和 Base URL
cc-switch-cli update-provider claude my-provider --name "New Name" --base-url https://new-api.example.com

# 更改 API 格式
cc-switch-cli update-provider claude my-provider --api-format openai_chat

# 清除 API 格式（恢复默认）
cc-switch-cli update-provider claude my-provider --clear-api-format
```

### remove-provider — 删除供应商

```bash
cc-switch-cli remove-provider <APP> <ID>
```

从数据库中永久删除供应商。

**示例**：

```bash
cc-switch-cli remove-provider claude my-provider
# 输出: 供应商 'my-provider' 已从 claude 删除
```

### switch-provider — 切换当前供应商

```bash
cc-switch-cli switch-provider <APP> <ID>
```

切换指定应用的当前活跃供应商。此操作会写入 Live 配置文件。

**示例**：

```bash
cc-switch-cli switch-provider claude backup-provider
# 输出: 已切换到供应商 'backup-provider' (claude)
```

### sort-providers — 调整供应商排序

```bash
cc-switch-cli sort-providers <APP> --order '<JSON_ARRAY>'
```

调整供应商的排序索引，影响列表显示顺序和故障转移优先级。

**参数**：

| 参数 | 必填 | 说明 |
|------|------|------|
| `APP` | 是 | 应用类型 |
| `--order` | 是 | JSON 数组，每项包含 `id` 和 `sortIndex` |

**示例**：

```bash
# 设置排序：provider-a 排第一，provider-b 排第二
cc-switch-cli sort-providers claude --order '[{"id":"provider-a","sortIndex":0},{"id":"provider-b","sortIndex":1}]'

# 使用文件传入（通过 shell 命令替换）
cc-switch-cli sort-providers claude --order "$(cat order.json)"
```

### import-live — 从 Live 配置导入

```bash
cc-switch-cli import-live <APP>
```

从应用的 Live 配置文件（如 `~/.claude/settings.json`、`~/.codex/config.toml` 等）导入供应商到数据库。仅在数据库中无该应用供应商时执行导入。

**示例**：

```bash
# 从 Claude live 配置导入
cc-switch-cli import-live claude

# 从 Codex live 配置导入
cc-switch-cli import-live codex

# 从 Gemini live 配置导入
cc-switch-cli import-live gemini
```

### read-live — 读取 Live 配置

```bash
cc-switch-cli read-live <APP>
```

读取并显示应用当前的 Live 配置文件内容（JSON 格式）。用于排查配置问题。

**示例**：

```bash
cc-switch-cli read-live claude
# 输出 Live 配置文件的完整 JSON 内容

cc-switch-cli read-live codex
```

### fetch-models — 获取模型列表

```bash
cc-switch-cli fetch-models --base-url <URL> --api-key <KEY> [--full-url] [--models-path <PATH>]
```

通过 OpenAI 兼容的 `GET /v1/models` 端点获取供应商支持的模型列表。

**参数**：

| 参数 | 必填 | 说明 |
|------|------|------|
| `--base-url` | 是 | API Base URL |
| `--api-key` | 是 | API Key |
| `--full-url` | 否 | 指定 base-url 为完整 URL，不自动拼接 `/v1/models`（默认 false） |
| `--models-path` | 否 | 自定义 models 端点路径 |

**示例**：

```bash
# 获取 OpenAI 模型列表
cc-switch-cli fetch-models --base-url https://api.openai.com --api-key sk-xxx

# 获取自定义端点模型列表
cc-switch-cli fetch-models --base-url https://my-api.example.com --api-key sk-xxx

# 使用完整 URL
cc-switch-cli fetch-models --base-url https://my-api.example.com/v1/models --api-key sk-xxx --full-url

# 使用自定义路径
cc-switch-cli fetch-models --base-url https://my-api.example.com --api-key sk-xxx --models-path /api/models
```

### sync-live — 同步到 Live 配置

```bash
cc-switch-cli sync-live
```

将数据库中所有应用的当前供应商配置同步写入到各自的 Live 配置文件。适用于数据库修改后需要更新 Live 配置的场景。

**示例**：

```bash
cc-switch-cli sync-live
# 输出: 已将数据库供应商同步到 Live 配置
```

---

## 代理配置

### takeover — 查看/设置代理接管

```bash
cc-switch-cli takeover <APP> [on|off]
```

查看或设置指定应用的代理接管状态。接管后，应用的 Live 配置会被修改为指向本地代理服务器。

**参数**：

| 参数 | 必填 | 说明 |
|------|------|------|
| `APP` | 是 | 应用类型（claude / codex / gemini） |
| `on` / `off` | 否 | 不指定则查看当前状态 |

**示例**：

```bash
# 查看所有应用接管状态
cc-switch-cli takeover claude

# 开启 Claude 接管
cc-switch-cli takeover claude on

# 关闭 Codex 接管
cc-switch-cli takeover codex off
```

### switch-proxy — 代理热切换

```bash
cc-switch-cli switch-proxy <APP> <ID>
```

在代理模式下热切换供应商，无需重启代理。仅修改代理的内部目标，不修改 Live 配置文件。

**示例**：

```bash
cc-switch-cli switch-proxy claude backup-provider
# 输出: 已热切换 claude 到供应商 'backup-provider' (逻辑目标变更: true)
```

### proxy-config — 查看/设置代理配置

```bash
cc-switch-cli proxy-config <get|set> [--config <JSON>]
```

查看或设置代理服务器配置（监听地址、端口、最大重试次数等）。

**参数**：

| 参数 | 必填 | 说明 |
|------|------|------|
| `action` | 是 | `get` 或 `set` |
| `--config` | set 时必填 | 配置 JSON |

**ProxyConfig 结构**：

```json
{
  "listen_address": "127.0.0.1",
  "listen_port": 9090,
  "max_retries": 3,
  "request_timeout": 300
}
```

**示例**：

```bash
# 查看当前代理配置
cc-switch-cli proxy-config get

# 设置代理配置
cc-switch-cli proxy-config set --config '{"listen_address":"0.0.0.0","listen_port":8080,"max_retries":3,"request_timeout":300}'
```

### global-proxy-config — 查看/设置全局代理配置

```bash
cc-switch-cli global-proxy-config <get|set> [--config <JSON>]
```

查看或设置全局代理配置（代理总开关、监听、日志等）。

**GlobalProxyConfig 结构**：

```json
{
  "proxy_enabled": true,
  "listen_address": "127.0.0.1",
  "listen_port": 9090,
  "enable_logging": true
}
```

**示例**：

```bash
cc-switch-cli global-proxy-config get
cc-switch-cli global-proxy-config set --config '{"proxy_enabled":true,"listen_address":"0.0.0.0","listen_port":9090,"enable_logging":true}'
```

### app-proxy-config — 查看/设置应用级代理配置

```bash
cc-switch-cli app-proxy-config <get|set> <APP> [--config <JSON>]
```

查看或设置指定应用的代理配置（启用开关、自动故障转移、最大重试等）。

**AppProxyConfig 结构**：

```json
{
  "app_type": "claude",
  "enabled": true,
  "auto_failover_enabled": false,
  "max_retries": 3
}
```

**示例**：

```bash
# 查看 Claude 代理配置
cc-switch-cli app-proxy-config get claude

# 设置 Codex 代理配置
cc-switch-cli app-proxy-config set codex --config '{"app_type":"codex","enabled":true,"auto_failover_enabled":true,"max_retries":3}'
```

### cost-multiplier — 查看/设置成本倍率

```bash
cc-switch-cli cost-multiplier <get|set> <APP> [--value <VALUE>]
```

查看或设置指定应用的默认成本倍率。成本倍率影响用量统计中的成本计算（实际成本 = 原始成本 × 倍率）。

**参数**：

| 参数 | 必填 | 说明 |
|------|------|------|
| `action` | 是 | `get` 或 `set` |
| `APP` | 是 | 应用类型 |
| `--value` | set 时必填 | 倍率值，如 `1.0`、`0.5`、`2.0` |

**示例**：

```bash
# 查看 Claude 成本倍率
cc-switch-cli cost-multiplier get claude
# 输出: claude 成本倍率: 1.0 (默认)

# 设置为 0.5（半价）
cc-switch-cli cost-multiplier set claude --value 0.5

# 设置为 2.0（双倍）
cc-switch-cli cost-multiplier set claude --value 2.0
```

### pricing-source — 查看/设置计费模型来源

```bash
cc-switch-cli pricing-source <get|set> <APP> [--value <VALUE>]
```

查看或设置计费模型来源。`official` 使用内置官方定价表，`custom` 使用用户自定义定价。

**示例**：

```bash
# 查看
cc-switch-cli pricing-source get claude

# 设置为官方定价
cc-switch-cli pricing-source set claude --value official

# 设置为自定义定价
cc-switch-cli pricing-source set claude --value custom
```

### takeover-status — 检测 Live 接管状态

```bash
cc-switch-cli takeover-status
```

检测 Live 配置文件是否当前被代理接管。用于排查代理状态问题。

**示例**：

```bash
cc-switch-cli takeover-status
# 输出: Live 配置已被代理接管
# 或
# 输出: Live 配置未被代理接管
```

---

## 故障转移与熔断器

### failover-queue — 管理故障转移队列

```bash
cc-switch-cli failover-queue <list|add|remove> <APP> [ID]
```

**参数**：

| 参数 | 必填 | 说明 |
|------|------|------|
| `action` | 是 | `list`（查看队列）、`add`（添加到队列）、`remove`（从队列移除） |
| `APP` | 是 | 应用类型 |
| `ID` | add/remove 时必填 | 供应商 ID |

**示例**：

```bash
# 查看 Claude 故障转移队列
cc-switch-cli failover-queue list claude

# 添加供应商到队列
cc-switch-cli failover-queue add claude backup-provider

# 从队列移除
cc-switch-cli failover-queue remove claude backup-provider
```

### auto-failover — 查看/设置自动故障转移

```bash
cc-switch-cli auto-failover [APP] [on|off]
```

不指定参数则查看全部应用的自动故障转移状态。指定 `APP` 和 `on/off` 则设置该应用。

**示例**：

```bash
# 查看全部
cc-switch-cli auto-failover

# 开启 Claude 自动故障转移
cc-switch-cli auto-failover claude on

# 关闭 Codex 自动故障转移
cc-switch-cli auto-failover codex off
```

### circuit-breaker — 熔断器管理

```bash
cc-switch-cli circuit-breaker <get|set|reset> [APP] [--config <JSON>] [ID]
```

**参数**：

| 参数 | 必填 | 说明 |
|------|------|------|
| `action` | 是 | `get`（查看配置）、`set`（设置配置）、`reset`（重置供应商熔断状态） |
| `APP` | get 时可选，set/reset 时必填 | 应用类型 |
| `--config` | set 时必填 | 配置 JSON |
| `ID` | reset 时必填 | 供应商 ID |

**CircuitBreakerConfig 结构**：

```json
{
  "failure_threshold": 5,
  "recovery_timeout": 60,
  "half_open_max_calls": 3
}
```

**示例**：

```bash
# 查看 Claude 熔断器配置
cc-switch-cli circuit-breaker get claude

# 设置熔断器配置
cc-switch-cli circuit-breaker set claude --config '{"failure_threshold":5,"recovery_timeout":60,"half_open_max_calls":3}'

# 重置供应商熔断状态
cc-switch-cli circuit-breaker reset claude my-provider
```

---

## 请求处理配置

### rectifier — 请求修正器

```bash
cc-switch-cli rectifier <get|set> [--config <JSON>]
```

查看或设置请求修正器配置。修正器用于在请求发送前修改请求体（如添加/删除字段、转换格式等）。

**示例**：

```bash
cc-switch-cli rectifier get
cc-switch-cli rectifier set --config '{"enabled":true,"rules":[]}'
```

### optimizer — 优化器

```bash
cc-switch-cli optimizer <get|set> [--config <JSON>]
```

查看或设置优化器配置。优化器用于优化请求（如缓存控制、Token 优化等）。

**示例**：

```bash
cc-switch-cli optimizer get
cc-switch-cli optimizer set --config '{"enabled":true,"rules":[]}'
```

---

## 全局出站代理

### global-proxy — 全局出站代理管理

```bash
cc-switch-cli global-proxy <get|set|clear|test> [URL]
```

管理全局出站代理（所有 HTTP 请求通过此代理发出）。适用于网络受限环境。

**参数**：

| 参数 | 必填 | 说明 |
|------|------|------|
| `action` | 是 | `get`（查看）、`set`（设置）、`clear`（清除）、`test`（测试连接） |
| `URL` | set 时必填，test 时可选 | 代理 URL（如 `http://127.0.0.1:7890`、`socks5://127.0.0.1:1080`） |

**示例**：

```bash
# 查看当前全局代理
cc-switch-cli global-proxy get

# 设置 HTTP 代理
cc-switch-cli global-proxy set http://127.0.0.1:7890

# 设置 SOCKS5 代理
cc-switch-cli global-proxy set socks5://127.0.0.1:1080

# 测试代理连接
cc-switch-cli global-proxy test http://127.0.0.1:7890
# 输出: 代理连接测试成功: http://127.0.0.1:7890 (HTTP 200, 350ms)

# 清除代理
cc-switch-cli global-proxy clear
```

---

## 配置与设置

### settings — 设备级设置

```bash
cc-switch-cli settings [KEY] [VALUE]
```

查看或修改设备级设置（`~/.cc-switch/settings.json`）。不指定参数则列出全部设置。

**常用设置项**：

| 键 | 类型 | 说明 |
|----|------|------|
| `silent_startup` | bool | 静默启动（不显示窗口） |
| `minimize_to_tray_on_close` | bool | 关闭时最小化到托盘 |
| `enable_local_proxy` | bool | 启用本地代理 |
| `enable_failover_toggle` | bool | 启用故障转移切换 |
| `language` | string | 界面语言（en/zh/ja） |
| `use_app_window_controls` | bool | 使用应用窗口控制按钮（Linux） |

**示例**：

```bash
# 列出全部设置
cc-switch-cli settings

# 查看单个设置
cc-switch-cli settings language

# 修改设置
cc-switch-cli settings silent_startup true
cc-switch-cli settings language zh
```

### config — 数据库配置

```bash
cc-switch-cli config [--key <KEY>] [--value <VALUE>]
```

查看或修改数据库 settings 表中的键值对。

**示例**：

```bash
# 列出全部数据库配置
cc-switch-cli config

# 查看单个配置
cc-switch-cli config --key cost_multiplier_claude

# 修改配置
cc-switch-cli config --key my_setting --value '{"enabled":true}'
```

### export-config — 导出配置

```bash
cc-switch-cli export-config <PATH>
```

将数据库配置导出为 SQL 文件。

**示例**：

```bash
cc-switch-cli export-config /tmp/cc-switch-backup.sql
```

### import-config — 导入配置

```bash
cc-switch-cli import-config <PATH>
```

从 SQL 文件导入配置到数据库。

**示例**：

```bash
cc-switch-cli import-config /tmp/cc-switch-backup.sql
```

---

## 声明式配置

### validate — 校验声明式配置

```bash
cc-switch-cli validate <PATH>
```

校验 YAML 声明式配置文件的语法和结构，不执行实际应用。

**示例**：

```bash
cc-switch-cli validate /etc/cc-switch/config.yaml
# 输出:
# ✓ 配置文件校验通过
#   供应商数量: 5
#   故障转移队列: 3 个应用
#   全局代理: 已配置
#   代理接管: 2 个应用
```

### apply-config — 应用声明式配置

```bash
cc-switch-cli apply-config <PATH>
```

先校验再应用 YAML 声明式配置文件到数据库。适用于自动化部署场景。

**示例**：

```bash
cc-switch-cli apply-config /etc/cc-switch/config.yaml
```

**YAML 配置示例**：

```yaml
proxy:
  takeover:
    claude: true
    codex: false

# 供应商列表
providers:
  - app: claude
    id: my-anthropic
    name: "My Anthropic"
    env:
      ANTHROPIC_API_KEY: "sk-ant-xxxxx"
      ANTHROPIC_BASE_URL: "https://api.anthropic.com"
    current: true

  - app: claude
    id: backup-provider
    name: "Backup Provider"
    env:
      ANTHROPIC_API_KEY: "sk-ant-yyyyy"
      ANTHROPIC_BASE_URL: "https://backup.example.com"

# 故障转移
failover:
  auto: true
  queue:
    claude:
      - my-anthropic
      - backup-provider

# 全局出站代理（可选）
# global_proxy:
#   url: "socks5://127.0.0.1:1080"

# 设备级设置
settings:
  language: "zh"
  backup_interval_hours: 24
  backup_retain_count: 10
```

> **说明**（2026-07-04 更新）：
> - `providers` 中通过 `env` 设置环境变量，`current: true` 表示切换为当前供应商。
> - **已移除字段**：`proxy.listen` 与 `proxy.port` 已从 YAML schema 删除。请使用 `CC_SWITCH_LISTEN` / `CC_SWITCH_PORT` 环境变量，或 `proxy-config` 命令设置监听地址与端口。`proxy.takeover` 仍可在 YAML 中配置并通过 `apply-config` 应用。
> - 如需设置 `api_format`，请在 `add-provider` / `update-provider` 中使用 `--api-format`。

---

## 备份与恢复

### backup-create — 创建备份

```bash
cc-switch-cli backup-create
```

创建数据库备份（SQL 格式），保存到 `~/.cc-switch/backups/` 目录，文件名包含时间戳。

**示例**：

```bash
cc-switch-cli backup-create
# 输出: 备份已创建: /home/user/.cc-switch/backups/cc-switch-backup-20260627_080000.sql
```

### backup-list — 列出备份

```bash
cc-switch-cli backup-list
```

列出所有可用备份，显示文件名、大小和创建时间。

**示例**：

```bash
cc-switch-cli backup-list
# 输出:
# 文件名                                     大小           创建时间
# cc-switch-backup-20260627_080000.sql      12 KB         2026-06-27 08:00:00
# cc-switch-backup-20260626_120000.sql      11 KB         2026-06-26 12:00:00
```

### backup-restore — 从备份恢复

```bash
cc-switch-cli backup-restore <NAME>
```

从指定备份文件恢复数据库。`NAME` 为备份文件名（不含完整路径）。

**示例**：

```bash
cc-switch-cli backup-restore cc-switch-backup-20260627_080000.sql
# 输出: 已从备份 'cc-switch-backup-20260627_080000.sql' 恢复
```

> **注意**：恢复后需重启代理服务使配置生效：`cc-switch-cli stop && cc-switch-cli daemon`

---

## 用量统计与监控

### usage-summary — 用量统计摘要

```bash
cc-switch-cli usage-summary [--days <N>]
```

查看最近 N 天的用量统计摘要，包括请求数、成本、Token 数、成功率、缓存命中率。

**参数**：

| 参数 | 说明 | 默认值 |
|------|------|--------|
| `--days` | 统计天数 | 7 |

**示例**：

```bash
# 查看最近 7 天
cc-switch-cli usage-summary

# 查看最近 30 天
cc-switch-cli usage-summary --days 30
```

**输出示例**：

```
用量统计 (最近 7 天):
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
  总请求数:       1234
  总成本:         $12.34
  输入 Token:     1.23M
  输出 Token:     456.78K
  缓存写入 Token: 100.00K
  缓存读取 Token: 500.00K
  实际总 Token:   1.79M
  成功率:         98.5%
  缓存命中率:     45.2%
```

---

## 测试与诊断

### speedtest — API 端点测速

> ℹ️ **不依赖代理运行**：此命令直接向目标 URL 发送 HTTP 请求，不需要代理服务器在运行。适合首次配置时测试供应商连通性。

```bash
cc-switch-cli speedtest <URL> [--timeout <SECONDS>]
```

测试 API 端点的响应延迟。

**参数**：

| 参数 | 说明 | 默认值 |
|------|------|--------|
| `URL` | 测试 URL | — |
| `--timeout` | 超时秒数 | 10 |

**示例**：

```bash
cc-switch-cli speedtest https://api.anthropic.com
# 输出: https://api.anthropic.com 延迟: 150ms (HTTP 200)

cc-switch-cli speedtest https://api.openai.com --timeout 5
```

### verify-key — 验证 API Key

> ℹ️ **不依赖代理运行**：此命令直接向供应商 API 发送认证请求，不需要代理服务器在运行。

```bash
cc-switch-cli verify-key --base-url <URL> --api-key <KEY>
```

通过请求 `/v1/models` 端点验证 API Key 是否有效。

**示例**：

```bash
cc-switch-cli verify-key --base-url https://api.anthropic.com --api-key sk-ant-xxx
# 输出: API Key 验证成功: https://api.anthropic.com (HTTP 200, 200ms)
# 或
# 输出: API Key 验证失败: https://api.anthropic.com (HTTP 401)
```

---

## MCP 与 Prompts

### list-mcp — 列出 MCP 服务器

```bash
cc-switch-cli list-mcp
```

列出数据库中所有 MCP 服务器配置。

**示例**：

```bash
cc-switch-cli list-mcp
# 输出:
# #   名称                    ID
# 1   Filesystem             filesystem
# 2   GitHub                 github
```

### add-mcp — 添加/更新 MCP 服务器

```bash
cc-switch-cli add-mcp <ID> <NAME> --command <CMD> [--args <JSON>] [--env <JSON>]
```

添加或更新 MCP 服务器配置。默认启用 Claude 应用。

**参数**：

| 参数 | 必填 | 说明 |
|------|------|------|
| `ID` | 是 | MCP 服务器唯一标识 |
| `NAME` | 是 | 显示名称 |
| `--command` | 是 | 启动命令（如 `npx`、`node`、`python`） |
| `--args` | 否 | 命令参数，JSON 数组格式 |
| `--env` | 否 | 环境变量，JSON 对象格式 |

**示例**：

```bash
# 添加 Filesystem MCP 服务器
cc-switch-cli add-mcp filesystem "Filesystem" \
  --command npx \
  --args '["@modelcontextprotocol/server-filesystem","/tmp"]'

# 添加带环境变量的 MCP 服务器
cc-switch-cli add-mcp github "GitHub MCP" \
  --command npx \
  --args '["@modelcontextprotocol/server-github"]' \
  --env '{"GITHUB_TOKEN":"ghp_xxx"}'

# 添加 Python MCP 服务器
cc-switch-cli add-mcp my-tool "My Tool" \
  --command python \
  --args '["/path/to/server.py","--port","8080"]'
```

### remove-mcp — 删除 MCP 服务器

```bash
cc-switch-cli remove-mcp <ID>
```

从数据库中删除指定的 MCP 服务器。

**示例**：

```bash
cc-switch-cli remove-mcp filesystem
# 输出: MCP 服务器 'filesystem' 已删除
```

### toggle-mcp — 启用/禁用 MCP

```bash
cc-switch-cli toggle-mcp <ID> <APP> <on|off>
```

切换 MCP 服务器在指定应用中的启用状态。

**示例**：

```bash
# 在 Claude 中启用
cc-switch-cli toggle-mcp filesystem claude on

# 在 Codex 中禁用
cc-switch-cli toggle-mcp filesystem codex off
```

### test-mcp — 测试 MCP 连接

```bash
cc-switch-cli test-mcp <ID>
```

测试 MCP 服务器命令是否可执行。此为基本连通性测试，完整连接测试请在 GUI 中进行。

**示例**：

```bash
cc-switch-cli test-mcp filesystem
# 输出:
# MCP 服务器: Filesystem (filesystem)
# 命令: npx @modelcontextprotocol/server-filesystem /tmp
# 启动测试: 成功 (150ms)
```

### list-prompts — 列出 Prompts

```bash
cc-switch-cli list-prompts [APP]
```

列出指定应用或全部应用的提示词。启用的提示词标记为 `*`。

**示例**：

```bash
# 列出全部
cc-switch-cli list-prompts

# 仅列出 Claude 提示词
cc-switch-cli list-prompts claude
```

### add-prompt — 添加/更新 Prompt

```bash
cc-switch-cli add-prompt <APP> <ID> <NAME> [--content <TEXT>] [--file <PATH>]
```

添加或更新提示词。可以使用 `--content` 直接指定内容，或使用 `--file` 从文件读取。

**参数**：

| 参数 | 必填 | 说明 |
|------|------|------|
| `APP` | 是 | 应用类型 |
| `ID` | 是 | Prompt 唯一标识 |
| `NAME` | 是 | 显示名称 |
| `--content` | 否 | 提示词内容（与 --file 二选一） |
| `--file` | 否 | 从文件读取内容 |

**示例**：

```bash
# 直接指定内容
cc-switch-cli add-prompt claude my-prompt "My Prompt" --content "You are a helpful assistant."

# 从文件读取
cc-switch-cli add-prompt claude my-prompt "My Prompt" --file /path/to/prompt.md

# 使用 heredoc
cc-switch-cli add-prompt claude my-prompt "My Prompt" --content "$(cat prompt.txt)"
```

### remove-prompt — 删除 Prompt

```bash
cc-switch-cli remove-prompt <APP> <ID>
```

删除指定提示词。已启用的提示词无法删除，需先禁用。

**示例**：

```bash
cc-switch-cli remove-prompt claude my-prompt
```

### enable-prompt — 启用/禁用 Prompt

```bash
cc-switch-cli enable-prompt <APP> <ID> <on|off>
```

启用或禁用提示词。启用时会将内容写入 Live 配置文件。

**示例**：

```bash
# 启用
cc-switch-cli enable-prompt claude my-prompt on

# 禁用
cc-switch-cli enable-prompt claude my-prompt off
```

---

## Skills 管理

### list-skills — 列出已安装 Skills

```bash
cc-switch-cli list-skills [APP]
```

列出已安装的 Skills。可按应用过滤。

**示例**：

```bash
# 列出全部
cc-switch-cli list-skills

# 仅列出 Claude 的 Skills
cc-switch-cli list-skills claude
```

### remove-skill — 卸载 Skill

```bash
cc-switch-cli remove-skill <ID> [APP]
```

卸载指定的 Skill。

**示例**：

```bash
cc-switch-cli remove-skill owner/repo:directory
# 输出: Skill 'owner/repo:directory' 已卸载
#       备份路径: /home/user/.cc-switch/skill-backups/...
```

### toggle-skill — 启用/禁用 Skill

```bash
cc-switch-cli toggle-skill <ID> <APP> <on|off>
```

切换 Skill 在指定应用中的启用状态。

**示例**：

```bash
cc-switch-cli toggle-skill owner/repo:directory claude on
cc-switch-cli toggle-skill owner/repo:directory codex off
```

---

## 环境变量检查

### check-env — 检查环境变量冲突

```bash
cc-switch-cli check-env
```

检查所有应用的环境变量是否与配置文件中的设置冲突（如系统环境变量与 `settings.json` 中的 `ANTHROPIC_API_KEY` 冲突）。

**示例**：

```bash
cc-switch-cli check-env
# 输出:
# ── claude ──
#   ANTHROPIC_API_KEY = sk-xxx (来源: system - HKLM\SYSTEM\...)
#
# ── codex ──
#   (无冲突)
#
# 或: 未检测到环境变量冲突
```

---

## 会话管理

### list-sessions — 列出会话

```bash
cc-switch-cli list-sessions [APP] [--limit <N>]
```

列出历史会话。

**参数**：

| 参数 | 说明 | 默认值 |
|------|------|--------|
| `APP` | 按供应商 ID 过滤 | — |
| `--limit` | 限制数量 | 20 |

**示例**：

```bash
# 列出最近 20 条会话
cc-switch-cli list-sessions

# 列出 50 条
cc-switch-cli list-sessions --limit 50
```

### remove-session — 删除会话

> ⚠️ **GUI 专属命令**：此命令需要 `provider_id` 与 `source_path` 完整参数，CLI 已移除此命令。CLI 用户请手动删除会话文件，或使用 GUI。

---

## 详细用量统计

### usage-trends — 查看用量趋势

```bash
cc-switch-cli usage-trends [--days <N>]
```

查看每日用量趋势。

**示例**：

```bash
cc-switch-cli usage-trends --days 30
# 输出:
# 用量趋势 (最近 30 天):
# ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
# 日期          请求数     成本($)       输入Token     输出Token
# 2026-06-27    150        1.50         500K          100K
# 2026-06-26    120        1.20         400K          80K
```

### provider-stats — 查看供应商统计

```bash
cc-switch-cli provider-stats [--days <N>]
```

按供应商查看用量统计。

**示例**：

```bash
cc-switch-cli provider-stats --days 7
# 输出:
# 供应商统计 (最近 7 天):
# ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
# 供应商                    请求数     成本($)       成功率
# Anthropic Direct          1000       10.00        99.5%
# OpenAI Proxy              500        5.00         98.0%
```

### model-stats — 查看模型统计

```bash
cc-switch-cli model-stats [--days <N>]
```

按模型查看用量统计。

**示例**：

```bash
cc-switch-cli model-stats --days 7
# 输出:
# 模型统计 (最近 7 天):
# ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
# 模型                       请求数     成本($)       平均成本/请求
# claude-sonnet-4-20250514   800        8.00         0.010
# gpt-4o                     200        2.00         0.010
```

---

## 流式健康检查

> ⚠️ **GUI 专属命令**：`stream-check` 与 `stream-check-all` 依赖代理运行时的 `CopilotAuthState`，CLI 架构上不可行。CLI 用户请使用 `speedtest` 或 `verify-key` 进行基本连通性测试。

---

## 代理运维与监控

### circuit-breaker-stats — 查看熔断器/供应商状态

```bash
cc-switch-cli circuit-breaker-stats <APP> <ID>
```

查看指定供应商的熔断器和健康状态，包括连续失败次数、最后成功/失败时间等。

**示例**：

```bash
cc-switch-cli circuit-breaker-stats claude my-provider
# 输出:
# 熔断器/供应商状态 (claude/my-provider):
#   健康状态:     不健康（可能已熔断）
#   连续失败次数: 5
#   最后成功:     2026-06-27T07:00:00Z
#   最后失败:     2026-06-27T07:30:00Z
#   最后错误:     HTTP 502 Bad Gateway
#   更新时间:     2026-06-27T07:30:00Z
```

### provider-health — 查看供应商健康状态

```bash
cc-switch-cli provider-health <APP> <ID>
```

查看指定供应商的详细健康状态。

**示例**：

```bash
cc-switch-cli provider-health claude my-provider
```

### failover-available — 列出可用故障转移供应商

```bash
cc-switch-cli failover-available <APP>
```

列出指定应用中可以加入故障转移队列但尚未加入的供应商。

**示例**：

```bash
cc-switch-cli failover-available claude
# 输出:
# 可加入 claude 故障转移队列的供应商:
#   backup-provider (Backup Provider)
#   openai-proxy (OpenAI Proxy)
```

### config-snippet — 通用配置片段管理

```bash
cc-switch-cli config-snippet <get|set|extract> <APP> [--snippet <JSON>]
```

管理多供应商共享的通用配置片段（如公共环境变量）。

**参数**：

| 参数 | 必填 | 说明 |
|------|------|------|
| `action` | 是 | `get`（查看）、`set`（设置）、`extract`（从 Live 配置提取） |
| `APP` | 是 | 应用类型 |
| `--snippet` | set 时必填 | 配置片段 JSON |

**示例**：

```bash
# 查看 Claude 通用配置片段
cc-switch-cli config-snippet get claude

# 设置通用配置片段
cc-switch-cli config-snippet set claude --snippet '{"env":{"CLAUDE_CODE_USE_BEDROCK":"0"}}'

# 从当前 Live 配置自动提取通用配置片段
cc-switch-cli config-snippet extract claude
```

### usage-by-app — 按应用查看用量

```bash
cc-switch-cli usage-by-app [--days <N>]
```

按应用类型分别查看用量统计。

**参数**：

| 参数 | 说明 | 默认值 |
|------|------|--------|
| `--days` | 统计天数 | 7 |

**示例**：

```bash
cc-switch-cli usage-by-app
cc-switch-cli usage-by-app --days 30
```

### request-logs — 查看请求日志

```bash
cc-switch-cli request-logs [--page <N>] [--page-size <N>] [--app <APP>] [--provider <NAME>] [--model <MODEL>] [--status <CODE>]
```

查看代理转发的请求日志，支持分页和多维过滤。

**参数**：

| 参数 | 说明 | 默认值 |
|------|------|--------|
| `--page` | 页码（从 1 开始） | 1 |
| `--page-size` | 每页条数 | 20 |
| `--app` | 按应用过滤 | — |
| `--provider` | 按供应商名称过滤 | — |
| `--model` | 按模型过滤 | — |
| `--status` | 按 HTTP 状态码过滤 | — |

**示例**：

```bash
# 查看最近 20 条请求
cc-switch-cli request-logs

# 查看 Claude 应用的请求
cc-switch-cli request-logs --app claude

# 查看失败请求（4xx/5xx）
cc-switch-cli request-logs --status 500

# 查看第二页
cc-switch-cli request-logs --page 2 --page-size 50

# 按供应商和模型过滤
cc-switch-cli request-logs --provider "OpenAI Proxy" --model gpt-4
```

### request-detail — 查看请求详情

```bash
cc-switch-cli request-detail <REQUEST_ID>
```

查看指定请求的详细信息（完整 JSON），包括 Token 用量、成本、延迟等。

**示例**：

```bash
cc-switch-cli request-detail abc123def456
# 输出完整的请求详情 JSON
```

### check-limits — 检查供应商用量限额

```bash
cc-switch-cli check-limits <APP> <ID>
```

检查指定供应商的日/月用量是否超过设定限额。

**示例**：

```bash
cc-switch-cli check-limits claude my-provider
# 输出:
# 供应商限额状态:
#   供应商 ID:  my-provider
#   日用量:     $5.23
#   日限额:     $10.00
#   日超限:     否
#   月用量:     $150.00
#   月限额:     $500.00
#   月超限:     否
```

### backup-delete — 删除备份

```bash
cc-switch-cli backup-delete <NAME>
```

删除指定的数据库备份文件。

**示例**：

```bash
cc-switch-cli backup-delete cc-switch-backup-20260620_120000.sql
# 输出: 备份 'cc-switch-backup-20260620_120000.sql' 已删除
```

### backup-rename — 重命名备份

```bash
cc-switch-cli backup-rename <OLD_NAME> <NEW_NAME>
```

重命名数据库备份文件，便于标记重要备份点。

**示例**：

```bash
cc-switch-cli backup-rename cc-switch-backup-20260627_080000.sql pre-migration-backup
# 输出: 备份已重命名
```

### endpoint — 管理自定义测速端点

```bash
cc-switch-cli endpoint <list|add|remove> <APP> <ID> [--url <URL>]
```

管理供应商的自定义测速端点，供 `speedtest` 命令使用。

**参数**：

| 参数 | 必填 | 说明 |
|------|------|------|
| `action` | 是 | `list`（列出）、`add`（添加）、`remove`（移除） |
| `APP` | 是 | 应用类型 |
| `ID` | 是 | 供应商 ID |
| `--url` | add/remove 时必填 | 端点 URL |

**示例**：

```bash
# 列出自定义端点
cc-switch-cli endpoint list claude my-provider

# 添加自定义测速端点
cc-switch-cli endpoint add claude my-provider --url https://api.anthropic.com/v1/messages

# 移除端点
cc-switch-cli endpoint remove claude my-provider --url https://api.anthropic.com/v1/messages
```

---

## 常见问题与故障排查

### 1. 代理服务器无法启动

**症状**：`cc-switch-cli start` 报错 "启动代理服务器失败"

**排查步骤**：

```bash
# 1. 检查端口是否被占用
ss -tlnp | grep 9090

# 2. 使用其他端口启动
CC_SWITCH_PORT=9091 cc-switch-cli start

# 3. 检查数据库是否正常
cc-switch-cli config

# 4. 使用 debug 日志查看详细错误
cc-switch-cli --log-level debug start
```

### 2. daemon 模式启动后立即退出

**症状**：`cc-switch-cli daemon` 显示启动成功，但 `status` 显示未运行

**排查步骤**：

```bash
# 1. 查看日志文件
cat ~/.cc-switch/cc-switch-daemon.log

# 2. 检查 PID 文件是否存在残留
cat ~/.cc-switch/cc-switch-daemon.pid

# 3. 检查对应进程是否存活
ps aux | grep cc-switch-cli

# 4. 清理残留 PID 文件后重试
rm ~/.cc-switch/cc-switch-daemon.pid
cc-switch-cli daemon
```

### 3. stop 命令无响应

**症状**：`cc-switch-cli stop` 卡住或报错 "无法连接"

**排查步骤**：

```bash
# 1. 确认代理正在运行
cc-switch-cli status

# 2. 检查监听地址和端口
echo $CC_SWITCH_LISTEN
echo $CC_SWITCH_PORT

# 3. 手动查找并终止进程
ps aux | grep cc-switch-cli
kill <PID>

# 4. 清理 PID 文件
rm ~/.cc-switch/cc-switch-daemon.pid
```

### 4. 数据库初始化失败

**症状**：任何命令报错 "数据库初始化失败"

**排查步骤**：

```bash
# 1. 检查数据库文件
ls -la ~/.cc-switch/cc-switch.db

# 2. 检查磁盘空间
df -h ~

# 3. 检查文件权限
ls -la ~/.cc-switch/

# 4. 从备份恢复
cc-switch-cli backup-list
cc-switch-cli backup-restore <backup-name>
```

### 5. 供应商切换后 Claude Code 仍使用旧配置

**症状**：`switch-provider` 成功，但 Claude Code 仍连接旧供应商

**原因**：Claude Code 可能缓存了配置，需要重启

**解决**：

```bash
# 1. 确认 Live 配置已更新
cc-switch-cli read-live claude

# 2. 如果使用代理模式，确认接管状态
cc-switch-cli takeover-status

# 3. 重启 Claude Code
```

### 6. 全局出站代理设置后代理无法连接

**症状**：设置 `global-proxy` 后，代理服务器请求全部失败

**排查步骤**：

```bash
# 1. 测试代理连通性
cc-switch-cli global-proxy test

# 2. 检查代理地址是否正确
cc-switch-cli global-proxy get

# 3. 如果代理不可用，清除设置
cc-switch-cli global-proxy clear

# 4. 重启代理服务
cc-switch-cli stop && cc-switch-cli daemon
```

### 7. 声明式配置校验失败

**症状**：`validate` 或 `apply-config` 报错

**排查步骤**：

```bash
# 1. 检查 YAML 语法
python3 -c "import yaml; yaml.safe_load(open('config.yaml'))"

# 2. 查看详细错误信息
cc-switch-cli --log-level debug validate /path/to/config.yaml

# 3. 参考示例配置
cat deploy/config.example.yaml
```

### 8. Windows 上 daemon 模式异常

**症状**：Windows 上 `daemon` 后台进程不工作

**说明**：Windows 使用 `DETACHED_PROCESS` 标志，子进程在独立进程组中运行。如果异常退出，PID 文件可能残留。

**解决**：

```powershell
# 查找进程
tasklist /FI "IMAGENAME eq cc-switch-cli.exe"

# 终止进程
taskkill /PID <PID> /F

# 清理 PID 文件
del %USERPROFILE%\.cc-switch\cc-switch-daemon.pid
```

### 9. 代理启动后，Claude Code 连接不上

**排查步骤**：

1. 确认代理已开启接管：`cc-switch-cli takeover claude on`
2. 确认 Claude Code 的 `~/.claude/settings.json` 已被修改指向本地代理
3. 确认供应商配置正确：`cc-switch-cli list-providers claude`
4. 查看代理日志排查错误

```bash
cc-switch-cli read-live claude
cc-switch-cli takeover-status
```

### 10. `stop` 命令显示成功但进程还在

`stop` 发送停止信号后等待最多 5 秒。如果代理正在处理大量请求，优雅关闭可能需要更长时间。等待几秒后用 `status` 确认，或强制终止：

```bash
# Linux
sudo kill $(cat /home/ccswitch/.cc-switch/cc-switch-daemon.pid)

# Windows
taskkill /PID <PID> /F
```

### 11. 如何迁移配置到另一台服务器

```bash
# 源服务器：导出配置
cc-switch-cli export-config /tmp/cc-switch-backup.sql

# 目标服务器：导入配置
cc-switch-cli import-config /path/to/cc-switch-backup.sql

# 或者直接复制整个配置目录
scp -r ~/.cc-switch/ user@new-server:~/.cc-switch/
```

### 12. Linux 编译报 "webkit2gtk not found"

安装 webkit2gtk 开发库：

```bash
sudo apt install -y libwebkit2gtk-4.1-dev libgtk-3-dev
```

### 13. CLI 和 GUI 可以同时使用吗

可以。两者共享 `~/.cc-switch/` 配置目录和数据库。但不建议同时运行 GUI 的代理和 CLI 的 `daemon`，避免端口冲突。

---

## 已知限制

1. **WebDAV/S3 自动同步**：`daemon` 模式下不启动自动同步 worker（需要 GUI 的 AppHandle）。可手动使用 `export-config` / `import-config` 配合外部工具同步。

2. **OAuth 设备码认证**：Copilot / Codex OAuth 设备码流程暂未在 CLI 中实现。使用 API Key 的供应商不受影响。

3. **用量统计**：`daemon` 模式会启动会话用量同步 worker，但部分查询功能为简化版。

4. **webkit2gtk 构建依赖**：在 Linux 上编译 CLI 仍需安装 webkit2gtk 开发库（因为 lib 模块引用了它）。运行时不需要。

5. **GUI 专属功能**：以下功能仅在 GUI 中可用：
   - 系统托盘、桌面通知、剪贴板
   - 文件选择对话框（CLI 使用路径参数替代）
   - 自动启动（使用 systemd / Task Scheduler 替代）
   - Deep link 导入、自动更新
   - Keychain（Linux 上已禁用，API Key 存在配置文件中）

6. **声明式配置中的代理字段**：`proxy.listen`、`proxy.port`、`proxy.takeover` 当前只参与解析和校验，不会被 `apply-config` 实际应用。请使用对应命令或环境变量设置。

---

## 附录

### 应用类型说明

| 应用类型 | 对应 CLI 工具 | 配置文件位置 |
|----------|-------------|-------------|
| `claude` | Claude Code | `~/.claude/settings.json` |
| `claude-desktop` | Claude Desktop | `~/Library/Application Support/Claude/claude_desktop_config.json` |
| `codex` | Codex CLI | `~/.codex/config.toml` |
| `gemini` | Gemini CLI | `~/.gemini/settings.json` |
| `opencode` | OpenCode | `~/.opencode/config.json` |
| `openclaw` | OpenClaw | `~/.openclaw/config.json` |
| `hermes` | Hermes | `~/.hermes/config.json` |

### 配置文件目录

| 文件 | 路径 | 说明 |
|------|------|------|
| 数据库 | `~/.cc-switch/cc-switch.db` | 主数据库 |
| 设备设置 | `~/.cc-switch/settings.json` | 设备级设置 |
| PID 文件 | `~/.cc-switch/cc-switch-daemon.pid` | daemon 进程 PID |
| 日志文件 | `~/.cc-switch/cc-switch-daemon.log` | daemon 日志 |
| 崩溃日志 | `~/.cc-switch/crash.log` | panic 崩溃记录 |
| 备份目录 | `~/.cc-switch/backups/` | 数据库备份 |
| 日志目录 | `~/.cc-switch/logs/` | GUI 模式日志 |

### 命令速查表

| 场景 | 命令 |
|------|------|
| 首次部署 | `daemon` → `add-provider` → `switch-provider` |
| 从现有配置迁移 | `import-live` → `takeover on` |
| 自动化部署 | `apply-config` → `daemon` |
| 切换供应商 | `switch-provider` 或 `switch-proxy`（代理模式） |
| 查看状态 | `status` |
| 查看用量 | `usage-summary` |
| 测试供应商 | `verify-key` / `speedtest` |
| 备份恢复 | `backup-create` / `backup-restore` |
| 配置导出迁移 | `export-config` / `import-config` |

---

## 代理安全与热重载（v3.16.6 新增）

### reload — 热重载代理配置

```bash
cc-switch-cli reload
```

不中断活跃连接，重新加载代理运行时配置（供应商列表、熔断器、故障转移队列等）。需要代理服务器正在运行。

**示例**：

```bash
cc-switch-cli reload
# 输出: ✓ 代理配置已热重载
```

### auth-token — 代理访问令牌

```bash
cc-switch-cli auth-token [set|clear] [--token <TOKEN>]
```

不指定参数则查看当前状态。设置后，所有代理请求必须携带 `Authorization: Bearer <token>` 头。清除后代理回到开放状态。

**示例**：

```bash
# 查看状态
cc-switch-cli auth-token
# 输出: 令牌未设置（代理完全开放）

# 设置令牌
cc-switch-cli auth-token set --token "my-secret-token-123"

# 清除令牌
cc-switch-cli auth-token clear
```

### acl — IP 白名单管理

```bash
cc-switch-cli acl <list|add|remove> [--cidr <CIDR>]
```

管理代理的 IP CIDR 白名单。设置后，只有白名单内的 IP 可以访问代理。

**示例**：

```bash
# 列出白名单
cc-switch-cli acl list

# 添加 CIDR
cc-switch-cli acl add --cidr "10.0.0.0/8"
cc-switch-cli acl add --cidr "172.16.0.0/12"

# 移除 CIDR
cc-switch-cli acl remove --cidr "10.0.0.0/8"
```

---

## 诊断与测试（v3.16.6 新增）

### smoke-test — 协议转换烟雾测试

```bash
cc-switch-cli smoke-test [APP]
```

不走网络，直接调用内部转换模块验证协议转换链路。不指定 APP 则测试全部应用。

**示例**：

```bash
# 全部测试
cc-switch-cli smoke-test

# 仅测试 Claude
cc-switch-cli smoke-test claude
```

### preview-conversion — 预览协议转换

```bash
cc-switch-cli preview-conversion --from <FMT> --to <FMT> --payload <JSON> [--base-url <URL>]
```

将请求体 JSON 从一种 API 格式转换为另一种，预览输出。不发送实际请求。

**示例**：

```bash
cc-switch-cli preview-conversion \
  --from anthropic \
  --to openai_chat \
  --payload '{"model":"claude-sonnet-5","messages":[{"role":"user","content":"Hello"}],"max_tokens":10}'
```

### proxy-trace — 代理链路跟踪指南

```bash
cc-switch-cli proxy-trace <APP> --model <MODEL>
```

生成 curl 命令和日志查看指南，帮助手动跟踪代理链路。

**示例**：

```bash
cc-switch-cli proxy-trace claude --model claude-sonnet-5
```

### replay-request — 重放历史请求

```bash
cc-switch-cli replay-request <REQUEST_ID>
```

查看历史请求详情并生成 curl 重放命令。需 `CC_SWITCH_LOG_BODIES=1` 环境变量启用请求体记录。

**示例**：

```bash
cc-switch-cli replay-request req_abc123
```

---

## 配置管理增强（v3.16.6 新增）

### export-yaml — 导出配置为 YAML

```bash
cc-switch-cli export-yaml <PATH>
```

将数据库配置导出为声明式 YAML 格式（`apply-config` 的逆操作）。

**示例**：

```bash
cc-switch-cli export-yaml /etc/cc-switch/config.yaml
```

### diff — 对比配置差异

```bash
cc-switch-cli diff <PATH>
```

对比 YAML 文件与当前数据库配置，显示新增/变更的供应商数量。

**示例**：

```bash
cc-switch-cli diff /etc/cc-switch/config.yaml
```

### rollback — 回滚配置

```bash
cc-switch-cli rollback
```

回滚到最近一次 `apply-config` 执行前的备份。

**示例**：

```bash
cc-switch-cli rollback
```

### toggle-provider — 启用/禁用供应商

```bash
cc-switch-cli toggle-provider <APP> <ID> <on|off>
```

临时禁用供应商而不删除配置。禁用的供应商不会参与代理路由和故障转移。

**示例**：

```bash
# 禁用
cc-switch-cli toggle-provider claude backup-provider off

# 重新启用
cc-switch-cli toggle-provider claude backup-provider on
```
| 停止服务 | `stop` |
