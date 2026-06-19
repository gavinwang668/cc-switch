# CC Switch CLI 部署与使用指南（Windows / Linux）

> 本文档介绍如何在不同平台上部署和使用 CC Switch CLI，包括 Windows 桌面环境与 Linux 无头服务器的完整流程。

---

## 目录

- [一、概述](#一概述)
- [二、获取二进制文件](#二获取二进制文件)
- [三、Windows 使用指南](#三windows-使用指南)
- [四、Linux 使用指南](#四linux-使用指南)
- [五、命令速查表](#五命令速查表)
- [六、声明式配置文件](#六声明式配置文件)
- [七、常见问题](#七常见问题)

---

## 一、概述

CC Switch CLI 是一个兼具**代理服务**和**管理工具**双重角色的命令行程序：

- **代理服务**：通过 `start` 或 `daemon` 命令启动本地 HTTP 代理服务器，拦截 Claude Code / Codex / Gemini CLI 等工具的 API 请求，实现多供应商路由、故障转移、用量统计等功能。
- **管理工具**：通过 `list-providers`、`switch-provider`、`settings` 等命令管理配置，执行完即退出，不依赖代理服务是否运行。

两者操作同一份本地数据（`~/.cc-switch/` 目录下的 `cc-switch.db` 数据库和 `settings.json` 配置文件），代理服务会实时感知管理命令带来的变化。

### 与 GUI 版本的关系

CC Switch 同时提供 GUI 桌面应用（`cc-switch`）和 CLI 工具（`cc-switch-cli`）。两者共享相同的后端逻辑和数据库，可以交替使用。CLI 特别适合：

- 无 GUI 的 Linux 服务器
- 需要脚本化 / 自动化管理的场景
- 需要以系统服务方式后台运行的场景

---

## 二、获取二进制文件

### 方式一：使用预编译二进制

从项目 `release/` 目录获取已编译好的二进制：

| 文件 | 平台 | 说明 |
|---|---|---|
| `cc-switch.exe` | Windows | GUI 桌面应用 |
| `cc-switch-cli.exe` | Windows | CLI 工具 |
| `cc-switch-cli-linux-x86_64` | Linux x86_64 | CLI 工具 |

### 方式二：自行编译

#### Windows 编译

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

#### Linux 编译（WSL 或原生 Linux）

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

---

## 三、Windows 使用指南

### 3.1 快速开始

将 `cc-switch-cli.exe` 放到任意目录（如 `C:\Tools\cc-switch\`），然后将其加入 PATH 或直接使用完整路径。

```powershell
# 查看帮助
cc-switch-cli.exe help

# 查看当前状态
cc-switch-cli.exe status

# 列出供应商
cc-switch-cli.exe list-providers
```

### 3.2 前台运行代理

```powershell
# 启动代理（前台运行，Ctrl+C 停止）
cc-switch-cli.exe start

# 自定义监听地址和端口
$env:CC_SWITCH_LISTEN = "0.0.0.0"
$env:CC_SWITCH_PORT = "8080"
cc-switch-cli.exe start
```

### 3.3 后台运行代理

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

### 3.4 日常管理

```powershell
# 添加供应商
cc-switch-cli.exe add-provider claude my-provider "My Provider" --api-key sk-ant-xxx --base-url https://api.anthropic.com

# 切换当前供应商
cc-switch-cli.exe switch-provider claude my-provider

# 开启代理接管 Claude
cc-switch-cli.exe takeover claude on

# 代理模式下热切换供应商
cc-switch-cli.exe switch-proxy claude backup-provider
```

### 3.5 配合 GUI 使用

Windows 上可以同时安装 GUI 应用和 CLI 工具。两者共享同一个 `~/.cc-switch/` 配置目录，操作完全互通：

- 在 GUI 中添加的供应商，CLI 也能看到
- CLI 切换的供应商，GUI 重启后也会反映
- 可以用 GUI 做初始配置，然后用 CLI 做自动化管理

### 3.6 Windows 开机自启（可选）

使用任务计划程序设置开机自动启动 daemon：

```powershell
# 创建开机自启任务
$action = New-ScheduledTaskAction -Execute "C:\Tools\cc-switch\cc-switch-cli.exe" -Argument "daemon"
$trigger = New-ScheduledTaskTrigger -AtLogon
Register-ScheduledTask -TaskName "CC Switch" -Action $action -Trigger $trigger
```

---

## 四、Linux 使用指南

### 4.1 安装

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

### 4.2 使用 systemd 管理服务

项目已提供 systemd unit 模板（`deploy/cc-switch.service`），按以下步骤部署：

```bash
# 复制 service 文件
sudo cp deploy/cc-switch.service /etc/systemd/system/

# 根据需要编辑配置
sudo vim /etc/systemd/system/cc-switch.service
```

service 文件内容参考：

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

### 4.3 日常管理

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

> **权限提示**：如果 service 以 `ccswitch` 用户运行，而管理命令以其他用户运行，需要确保两者都能访问 `~/.cc-switch/` 目录。建议统一使用同一用户，或将配置目录设为共享。

### 4.4 让 CLI 工具指向正确的配置

默认情况下 CLI 读写 `~/.cc-switch/` 目录。如果 service 以 `ccswitch` 用户运行，配置目录为 `/home/ccswitch/.cc-switch/`。管理命令也需要指向同一目录：

```bash
# 方式一：以 ccswitch 用户执行管理命令
sudo -u ccswitch cc-switch-cli list-providers

# 方式二：通过环境变量覆盖配置目录（需确认代码支持）
# 当前版本不支持环境变量覆盖，建议使用方式一
```

### 4.5 查看日志

daemon 模式下日志输出到两个地方：

```bash
# 方式一：journalctl（systemd 管理时）
sudo journalctl -u cc-switch -f

# 方式二：日志文件
tail -f /home/ccswitch/.cc-switch/cc-switch-daemon.log
```

### 4.6 配合 Claude Code / Codex / Gemini CLI 使用

代理服务器启动并接管后，各 CLI 工具的请求会自动经过代理。确保各工具的配置指向本地代理：

代理默认监听 `127.0.0.1:9090`，接管后会自动修改各 CLI 工具的 Live 配置文件（如 `~/.claude/settings.json`），将其 API 端点指向本地代理。无需手动修改 CLI 工具配置。

如果需要手动配置或验证：

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

## 五、命令速查表

### 代理管理

| 命令 | 说明 |
|---|---|
| `start` | 前台启动代理（调试用，Ctrl+C 停止） |
| `daemon` | 后台启动代理（生产用） |
| `stop` | 停止后台代理（发送 HTTP POST /stop） |
| `status` | 查看代理运行状态和当前供应商 |

### 供应商管理

| 命令 | 说明 |
|---|---|
| `list-providers [APP]` | 列出供应商（不指定 APP 则列出全部 7 种） |
| `add-provider APP ID NAME [--api-key K] [--base-url U]` | 添加供应商 |
| `update-provider APP ID [--name N] [--api-key K] [--base-url U]` | 更新供应商 |
| `remove-provider APP ID` | 删除供应商 |
| `switch-provider APP ID` | 切换当前供应商 |

APP 取值：`claude`、`claude-desktop`、`codex`、`gemini`、`opencode`、`openclaw`、`hermes`

### 代理配置

| 命令 | 说明 |
|---|---|
| `takeover APP [on\|off]` | 查看 / 设置代理接管（仅 claude/codex/gemini） |
| `switch-proxy APP ID` | 代理模式下热切换供应商 |
| `failover-queue list\|add\|remove APP [ID]` | 管理故障转移队列 |
| `auto-failover [APP] [on\|off]` | 查看 / 设置自动故障转移 |
| `circuit-breaker get\|set\|reset [APP] [--config JSON] [ID]` | 熔断器管理 |
| `rectifier get\|set [--config JSON]` | 请求修正器配置 |
| `optimizer get\|set [--config JSON]` | 优化器配置 |
| `global-proxy get\|set\|clear\|test [URL]` | 全局出站代理 |

### 配置与设置

| 命令 | 说明 |
|---|---|
| `settings [KEY] [VALUE]` | 设备级设置（`~/.cc-switch/settings.json`） |
| `config [--key K] [--value V]` | 数据库配置（settings 表） |
| `export-config PATH` | 导出配置到 SQL 文件 |
| `import-config PATH` | 从 SQL 文件导入配置 |
| `validate PATH` | 校验声明式 YAML 配置 |
| `apply-config PATH` | 应用声明式 YAML 配置到数据库 |

### 备份与恢复

| 命令 | 说明 |
|---|---|
| `backup-create` | 创建数据库备份 |
| `backup-list` | 列出所有备份 |
| `backup-restore NAME` | 从备份恢复 |

### 其他

| 命令 | 说明 |
|---|---|
| `list-mcp` | 列出 MCP 服务器 |
| `list-prompts [APP]` | 列出 Prompts |
| `usage-summary [--days N]` | 查看用量统计 |
| `speedtest URL [--timeout S]` | 测试 API 端点延迟 |
| `verify-key --base-url U --api-key K` | 验证 API Key |
| `help` | 显示完整帮助信息 |

### 环境变量

| 变量 | 默认值 | 说明 |
|---|---|---|
| `CC_SWITCH_LISTEN` | `127.0.0.1` | 代理监听地址 |
| `CC_SWITCH_PORT` | `9090` | 代理监听端口 |

### 全局选项

- `--log-level <LEVEL>`：日志级别（`error` / `warn` / `info` / `debug` / `trace`，默认 `info`）

---

## 六、声明式配置文件

对于需要批量配置或版本管理配置的场景，可以使用 YAML 声明式配置文件。

### 6.1 配置文件格式

```yaml
# 代理服务器
proxy:
  listen: "127.0.0.1"
  port: 9090
  takeover:
    claude: true
    codex: true
    gemini: false

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

### 6.2 使用方式

```bash
# 校验配置文件合法性
cc-switch-cli validate config.yaml

# 应用配置到数据库
cc-switch-cli apply-config config.yaml
```

完整示例见 `deploy/config.example.yaml`。

---

## 七、常见问题

### Q: 代理启动后，Claude Code 连接不上？

检查以下几点：

1. 确认代理已开启接管：`cc-switch-cli takeover claude on`
2. 确认 Claude Code 的 `~/.claude/settings.json` 已被修改指向本地代理
3. 确认供应商配置正确：`cc-switch-cli list-providers claude`
4. 查看代理日志排查错误

### Q: daemon 启动后立即退出？

1. 检查端口是否被占用：`netstat -tlnp | grep 9090`（Linux）或 `netstat -ano | findstr 9090`（Windows）
2. 检查 PID 文件是否残留：删除 `~/.cc-switch/cc-switch-daemon.pid`
3. 查看日志文件：`~/.cc-switch/cc-switch-daemon.log`

### Q: 管理命令报"数据库初始化失败"？

确保当前用户有权限读写 `~/.cc-switch/` 目录。如果 service 以 `ccswitch` 用户运行，管理命令也需要以该用户执行：

```bash
sudo -u ccswitch cc-switch-cli list-providers
```

### Q: Linux 上编译报错"webkit2gtk not found"？

安装 webkit2gtk 开发库：

```bash
sudo apt install -y libwebkit2gtk-4.1-dev libgtk-3-dev
```

### Q: stop 命令显示成功但进程还在？

`stop` 发送停止信号后等待最多 5 秒。如果代理正在处理大量请求，优雅关闭可能需要更长时间。等待几秒后用 `status` 确认，或直接用 `kill` 终止：

```bash
# Linux
sudo kill $(cat /home/ccswitch/.cc-switch/cc-switch-daemon.pid)

# Windows
taskkill /PID <PID> /F
```

### Q: CLI 和 GUI 可以同时使用吗？

可以。两者共享同一个 `~/.cc-switch/` 配置目录和数据库。在 GUI 中做的修改 CLI 能看到，反之亦然。但不建议同时运行 GUI 的代理和 CLI 的 daemon，避免端口冲突。

### Q: 如何迁移配置到另一台服务器？

```bash
# 源服务器：导出配置
cc-switch-cli export-config /tmp/cc-switch-backup.sql

# 目标服务器：导入配置
cc-switch-cli import-config /path/to/cc-switch-backup.sql

# 或者直接复制整个配置目录
scp -r ~/.cc-switch/ user@new-server:~/.cc-switch/
```

### Q: 信号处理

daemon 模式下支持以下信号（仅 Linux）：

| 信号 | 行为 |
|---|---|
| `SIGTERM` / `SIGINT` | 优雅停止：恢复 Live 配置 → 删除 PID 文件 → 退出 |
| `SIGHUP` | 记录日志（配置热重载暂未实现） |

Windows 上仅支持 `Ctrl+C`（SIGINT 等效）。
