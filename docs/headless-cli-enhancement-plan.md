# CC Switch 无 GUI 后台运行与 CLI 改造计划

> 文档版本：v1.1  ｜  日期：2026-06-26  ｜  状态：Phase 1-2 全部完成，Phase 3-4 部分完成
>
> 目标：评估当前项目在无 GUI 的 Linux 服务器上后台运行的可行性，并扩展 CLI 使其功能覆盖 GUI 中的所有配置能力（GUI 保留不删除，CLI 作为无头环境的等价替代方案），给出分阶段改造计划。

---

## 一、执行摘要

当前项目**已具备一个基础 CLI**（`cc-switch-cli`，8 个子命令）和一个**可独立启动的本地代理服务器**（`ProxyServer`，`app_handle` 可为 `None`）。配置数据本身已文件化/数据库化（`~/.cc-switch/settings.json` + `cc-switch.db`），这意味着大部分 GUI 功能的后端逻辑并不强依赖窗口，理论上可以被 CLI 复用。

但现状距离"CLI 功能覆盖 GUI"仍有较大差距：

- **CLI 仅覆盖约 4% 的 GUI 功能**（8 命令 vs GUI 暴露的 200+ Tauri 命令）。
- **CLI 存在功能性缺陷**：`stop` 子命令调用的 `/stop` HTTP 路由在代理服务器中并不存在；`config` 子命令操作的是数据库 `settings` 表，而非设备级 `settings.json`。
- **无守护进程/后台服务能力**：CLI 的 `start` 是前台阻塞运行，没有 PID 文件、没有 `daemon` 模式、没有 systemd unit。
- **约 15% 的命令强依赖 GUI 运行时**（托盘、系统对话框、窗口主题、剪贴板、deep link、自动启动等），无法直接在纯 CLI 中复用。
- **lightweight 模式不是真正的无头模式**，它仍在 Tauri 应用进程内运行，只是销毁主窗口保留托盘，强依赖 `tauri::AppHandle`。

**结论**：CLI 功能覆盖 GUI（GUI 保留不删除）在技术上可行但工作量中等偏大，建议分 4 个阶段渐进改造。约 70% 的功能可低成本复用现有服务层，15% 需中等改造，15% 属于"GUI 专属"不建议或无必要在 CLI 中实现。

---

## 二、现状评估

### 2.1 现有 CLI 能力（`src-tauri/src/bin/cc-switch-cli.rs`）

| 子命令 | 功能 | 对应 GUI 能力 | 完备性 |
|---|---|---|---|
| `start` | 前台启动代理服务器（阻塞，Ctrl+C 退出） | `start_proxy_server` | 部分：仅启动，不接管 Live 配置、不恢复上次状态 |
| `stop` | 通过 HTTP GET `/stop` 停止后台代理 | `stop_proxy_server` | **有缺陷**：服务器未注册 `/stop` 路由，实际返回 404 但 CLI 误报成功 |
| `status` | 查看代理状态 + 当前供应商（仅 claude/codex/gemini） | `get_proxy_status` + `get_current_provider` | 部分：缺少 OpenCode/OpenClaw/Hermes/ClaudeDesktop |
| `config` | 读写数据库 `settings` 表的键值 | `get_settings`/`save_settings` | **错位**：操作的是 DB settings 表，而非设备级 `settings.json`（AppSettings），无法设置 UI/目录/同步等关键配置 |
| `list-providers` | 列出供应商（默认仅 3 种 app） | `get_providers` | 部分：不支持全部 7 种 app 类型 |
| `add-provider` | 添加供应商（仅 api_key + base_url） | `add_provider` | 极度简化：无法设置模型映射、环境变量、meta、category 等完整字段 |
| `remove-provider` | 删除供应商 | `delete_provider` | 基本可用 |
| `switch-provider` | 切换当前供应商 | `switch_provider` | 基本可用 |

**环境变量支持**：`CC_SWITCH_LISTEN`（监听地址）、`CC_SWITCH_PORT`（端口）。

### 2.2 GUI 功能全景

GUI 设置对话框包含 **6 个 Tab**（General / Proxy / Auth / Advanced / Usage / About），主界面还有供应商列表、MCP 管理、Skills 管理、Prompts 管理、会话管理等功能区。后端通过 `invoke_handler` 注册了 **200+ 个 Tauri 命令**，按功能模块归类如下：

| 功能域 | GUI 命令数 | CLI 已覆盖 | 主要缺口 |
|---|---|---|---|
| 供应商管理（含 7 种 app、通用供应商、Live 导入） | ~30 | 4 | 更新、排序、通用供应商、各 app 专属导入、Live 读取 |
| 代理管理（启停、接管、热切换、配置） | ~19 | 1 | 接管开关、热切换、配置读写、成本倍率、计费来源 |
| 故障转移（队列、熔断器配置/统计/重置） | ~11 | 0 | 全部缺失 |
| 请求修正器/优化器 | ~6 | 0 | 全部缺失 |
| 全局出站代理 | ~5 | 0 | 全部缺失 |
| MCP 管理（Claude/Codex/Gemini/统一） | ~14 | 0 | 全部缺失 |
| Skills 管理（统一 + 遗留） | ~23 | 0 | 全部缺失 |
| Prompts 管理 | ~6 | 0 | 全部缺失 |
| 设备级设置（AppSettings） | ~6 | 0 | 全部缺失（config 命令操作的是 DB 不是 settings.json） |
| 导入导出/备份恢复 | ~9 | 0 | 全部缺失 |
| 云同步（WebDAV / S3） | ~10 | 0 | 全部缺失 |
| 用量统计（概览/趋势/日志/统计/定价） | ~20 | 0 | 全部缺失 |
| 流检查 | ~3 | 0 | 全部缺失 |
| 会话管理 | ~8 | 0 | 全部缺失 |
| OAuth 认证（Copilot / Codex / 通用） | ~22 | 0 | 全部缺失 |
| Keychain（API Key 安全存储） | 3 | 0 | Linux 下已被 `cfg` 掉，需替代方案 |
| Hermes 专属 | ~7 | 0 | 全部缺失 |
| OpenClaw 专属 | ~9 | 0 | 全部缺失 |
| OMO | ~6 | 0 | 全部缺失 |
| 工作区/文件（OpenClaw） | ~9 | 0 | 全部缺失 |
| 模型获取 / 端点测速 | ~6 | 0 | 全部缺失 |
| 环境变量管理 | ~3 | 0 | 全部缺失 |
| Claude 插件 | ~6 | 0 | 全部缺失 |
| Deep link 导入 | ~4 | 0 | 全部缺失（GUI 专属，CLI 可选支持） |
| 自动启动 | ~2 | 0 | GUI/桌面环境专属 |
| 轻量模式 | ~3 | 0 | 非真正无头，不建议 CLI 实现 |
| 杂项/UI（打开文件夹、剪贴板、窗口主题、更新检查等） | ~25 | 0 | 多数 GUI 专属 |
| **合计** | **~200+** | **~8** | — |

### 2.3 代理服务器独立性评估

`ProxyServer`（`src-tauri/src/proxy/server.rs`）的核心设计**支持无 GUI 运行**：

- 构造函数 `ProxyServer::new(config, db, app_handle: Option<tauri::AppHandle>)` — 第三个参数可为 `None`，CLI 已利用此点。
- 服务器基于 Axum + 手动 hyper accept loop，不依赖 Tauri 事件循环。
- `ProxyState.app_handle` 仅用于向前端 emit 事件和刷新托盘菜单，为 `None` 时这些副作用静默跳过（需验证所有 emit 调用做了 `Option` 判空）。

**但存在以下问题**：

1. **`stop` 路由缺失**：`build_router()` 注册了 `/health`、`/status` 和各 API 路由，但没有 `/stop`。CLI 的 `stop` 命令实际无效。
2. **无信号处理集成**：CLI `start` 仅处理 Ctrl+C，没有 SIGTERM/SIGHUP 处理，不适合被进程管理器（systemd/supervisor）优雅管理。
3. **无 Live 配置接管**：CLI `start` 直接启动代理但不执行 `set_takeover_for_app`，意味着代理虽运行但不会接管 Claude/Codex/Gemini 的 Live 配置（GUI 启动时由 `restore_proxy_state_on_startup` 完成）。
4. **无启动时恢复逻辑**：GUI 启动时会恢复上次代理状态、清理异常退出的接管残留、自动导入 Live 配置/官方预设供应商/MCP/Prompts/Skills。CLI `start` 完全跳过这些初始化，可能导致代理运行但数据库/配置不完整。
5. **无用量统计同步**：GUI 启动时和每 60 秒同步会话用量，CLI 模式下不会运行，导致用量统计缺失。
6. **无周期性维护**：GUI 启动每日定时备份，CLI 模式下不运行。
7. **WebDAV/S3 自动同步 worker 不启动**：GUI 启动时 `start_worker`，CLI 不会。

### 2.4 配置存储架构

配置分为两层，均以文件形式存在，**CLI 可直接读写**：

| 存储 | 路径 | 内容 | CLI 现状 |
|---|---|---|---|
| 设备级设置 | `~/.cc-switch/settings.json` | AppSettings：UI 行为、目录覆盖、当前供应商、Skill 同步方式、WebDAV/S3 凭证、备份策略、终端、语言等 | CLI 的 `config` 命令**未操作此文件**，而是操作 DB settings 表 |
| 应用数据库 | `~/.cc-switch/cc-switch.db` (SQLite) | 供应商、MCP、Skills、Prompts、代理配置、熔断器配置、用量记录、请求日志、定价、会话、备份记录等 | CLI 直接通过 `Database::init()` 访问，可行 |
| 各 CLI 工具的 Live 配置 | `~/.claude/`、`~/.codex/`、`~/.gemini/` 等 | 被 CC Switch 接管/切换时改写的实际 CLI 工具配置 | CLI 可通过现有服务层访问 |

### 2.5 lightweight 模式评估

`src-tauri/src/lightweight.rs` 提供的"轻量模式"**不是无头模式**：

- 它在 Tauri 应用进程内运行，通过 `window.destroy()` 销毁主窗口，但保留 Tauri 运行时和托盘。
- 强依赖 `tauri::AppHandle`，无法脱离 Tauri 框架使用。
- 仅适用于桌面环境下降内存/资源占用，不适用于无 GUI 服务器。

---

## 三、差距分析

### 3.1 CLI vs GUI 功能覆盖矩阵（关键模块）

> 下表按"对服务器后台运行的重要性"排序，标注每项的 CLI 覆盖状态与改造难度。

| 优先级 | 功能模块 | GUI 命令示例 | CLI 覆盖 | 改造难度 | 说明 |
|---|---|---|---|---|---|
| P0 | 代理启停与管理 | start/stop/status_proxy/takeover/switch | 部分（有bug） | 低 | 修复 stop 路由 + 补齐接管/热切换命令 |
| P0 | 设备级设置读写 | get_settings/save_settings | 缺失 | 低 | 新增 `settings get/set` 子命令直接操作 settings.json |
| P0 | 供应商完整管理 | add/update/delete/switch/sort | 部分 | 低-中 | 补齐 update、完整字段、全部 app 类型 |
| P0 | 守护进程化 | — | 缺失 | 中 | 新增 daemon 模式 + PID 文件 + 信号处理 |
| P1 | 故障转移配置 | failover queue / circuit breaker | 缺失 | 低 | 复用服务层，新增子命令 |
| P1 | 请求修正器/优化器 | get/set_rectifier/optimizer | 缺失 | 低 | 复用服务层 |
| P1 | 全局出站代理 | get/set_global_proxy_url | 缺失 | 低 | 复用服务层 |
| P1 | MCP 管理 | upsert/delete/toggle/test_mcp | 缺失 | 中 | 复用服务层，需设计 CLI 交互 |
| P1 | 代理 Live 接管恢复 | restore_proxy_state | 缺失 | 中 | CLI 启动时需执行与 GUI 相同的恢复逻辑 |
| P2 | Skills 管理 | install/uninstall/toggle/update | 缺失 | 中 | 复用服务层 |
| P2 | Prompts 管理 | upsert/delete/enable | 缺失 | 低 | 复用服务层 |
| P2 | 用量统计查询 | get_usage_summary/stats/logs | 缺失 | 低 | 复用 DAO 层，格式化输出 |
| P2 | 导入导出/备份恢复 | export/import/backup | 缺失 | 低 | 复用服务层 |
| P2 | 云同步 | webdav/s3 upload/download | 缺失 | 低-中 | 复用服务层，需处理 worker 启动 |
| P3 | OAuth 认证 | copilot/codex device flow | 缺失 | 高 | 设备码流程可在 CLI 实现（打印 URL + 轮询），但交互复杂 |
| P3 | Hermes/OpenClaw 专属 | 各种 get/set | 缺失 | 中 | 复用服务层，命令较多 |
| P3 | 会话管理 | list/delete sessions | 缺失 | 低 | 复用服务层 |
| P3 | 流检查 | stream_check | 缺失 | 低 | 复用服务层 |
| P3 | 模型获取/端点测速 | fetch_models/test_endpoints | 缺失 | 低 | 复用服务层 |
| P3 | 环境变量管理 | check/delete/restore_env | 缺失 | 低 | 复用服务层 |
| — | 系统托盘 | tray menu | N/A | — | GUI 专属，CLI 不需要 |
| — | 文件选择对话框 | save/open_file_dialog | N/A | — | GUI 专属，CLI 用路径参数替代 |
| — | 窗口主题 | set_window_theme | N/A | — | GUI 专属 |
| — | 剪贴板 | copy_text_to_clipboard | N/A | — | GUI 专属，CLI 直接 stdout |
| — | Deep link 导入 | parse/merge/import_deeplink | N/A | — | GUI 专属，CLI 可选支持 URL 参数 |
| — | 自动启动 | set/get_auto_launch | N/A | — | 桌面环境专属，服务器用 systemd 替代 |
| — | 轻量模式 | enter/exit_lightweight | N/A | — | 非真正无头，CLI 不需要 |
| — | 更新检查/安装 | check_for_updates | N/A | — | GUI 专属，服务器用包管理器 |
| — | 打开文件夹/终端 | open_config_folder/terminal | N/A | — | GUI 专属 |
| — | Keychain | set/get/delete_api_key | N/A | — | Linux 已 cfg 掉，CLI 用配置文件/环境变量存 API Key |

### 3.2 强依赖 GUI 的模块识别

以下模块**本质上必须有窗口/桌面环境**，无法也不必在纯 CLI 中实现：

1. **系统托盘**（`tray.rs`）— 托盘菜单、图标、最小化到托盘。
2. **系统对话框**（`tauri-plugin-dialog`）— 文件选择、保存对话框。Linux 下已被 `cfg(not(target_os = "linux"))` 排除。CLI 用路径参数替代。
3. **窗口管理**（`set_window_theme`、`window-state` 插件）— 窗口主题、位置、大小。
4. **剪贴板**（`arboard`）— 复制到剪贴板。CLI 直接输出到 stdout。
5. **系统通知**（`useSystemNotifications`）— 桌面通知。CLI 用日志替代。
6. **自动启动**（`auto-launch`）— 开机自启依赖桌面环境的 autostart 机制。服务器用 systemd unit 替代。
7. **Deep link**（`tauri-plugin-deep-link`）— `ccswitch://` 协议注册依赖桌面环境。CLI 可选支持直接传入 URL 参数。
8. **轻量模式**（`lightweight.rs`）— 依赖 Tauri 运行时。
9. **更新检查/安装**（`tauri-plugin-updater`）— 桌面应用自更新。服务器用包管理器。
10. **打开外部应用**（`open_external`、`open_provider_terminal`、`launch_session_terminal`、`open_hermes_web_ui`）— 依赖桌面环境。
11. **Keychain**（`keyring`）— Linux 下已 `cfg` 掉。CLI 模式 API Key 直接存在供应商配置或环境变量中。
12. **WebKitGTK 相关**（`linux_fix`、硬件加速禁用）— 仅 GUI 需要。

**重要**：这些 GUI 专属模块**不影响**后端服务层的核心逻辑。供应商管理、代理、MCP、Skills、Prompts、用量统计等业务逻辑均在 `services/` 层实现，不依赖 Tauri 运行时，CLI 可直接调用。

### 3.3 守护进程 / 后台服务能力评估

当前 CLI **不具备**守护进程能力：

| 能力 | 现状 | 需求 |
|---|---|---|
| 后台运行（detach） | 无，`start` 前台阻塞 | 需 `--daemon` 或 `start --background` |
| PID 文件 | 无 | 需写入 `~/.cc-switch/cc-switch.pid` |
| 信号处理 | 仅 Ctrl+C | 需 SIGTERM 优雅停止 + SIGHUP 重载配置 |
| 进程管理 | 无 | 需 systemd unit / supervisor 配置示例 |
| 日志重定向 | stdout | 需日志文件 + 轮转（已有 tauri-plugin-log，但 CLI 未集成） |
| 单实例保证 | 无 | 需 PID 文件锁或 Unix socket |
| 崩溃恢复 | 无 | 需 systemd Restart=on-failure |

---

## 四、改造可行性评估

### 4.1 可直接改造（低成本，复用现有服务层）

这些功能的后端逻辑完全在 `services/` 层，不依赖 `tauri::AppHandle`，CLI 只需调用 `Database` + Service 即可：

- 供应商 CRUD（补齐 update、完整字段、全部 app 类型）
- 设备级设置读写（直接操作 `settings.json`，已有 `settings::get_settings()` / `update_settings()`）
- 代理配置读写、接管开关、热切换（`ProxyService` 方法）
- 故障转移队列与熔断器配置（DAO 层 + `ProxyService`）
- 请求修正器/优化器配置（DAO 层）
- 全局出站代理配置（DAO 层 + `http_client::init`）
- MCP 管理（`McpService`）
- Prompts 管理（`PromptService`）
- 用量统计查询（DAO 层）
- 导入导出/备份恢复（`ConfigService` + DAO）
- 端点测速（`SpeedtestService`）
- 模型获取（`ModelFetchService`）
- 会话管理查询（DAO 层）
- 环境变量管理（`EnvManager`）

### 4.2 需中等改造

- **守护进程化**：新增 daemon 模式、PID 文件、信号处理、日志集成。
- **CLI 启动时初始化逻辑**：复用 GUI `setup` 中的数据库迁移、Live 导入、官方预设、MCP/Prompts/Skills 导入、代理状态恢复、用量同步 worker、WebDAV/S3 worker、周期性备份 timer。需将 `lib.rs` setup 逻辑抽取为可复用函数。
- **OAuth 认证**：设备码流程可在 CLI 实现（打印验证 URL，轮询状态），但需处理多账号管理交互。
- **云同步触发**：WebDAV/S3 的同步操作本身是服务层方法，但自动同步 worker 需在 daemon 模式下启动。
- **MCP 连接测试**：`test_mcp_connection` 可能涉及进程启动，需验证是否依赖 GUI。
- **流检查**：`stream_check_provider` 发起真实 API 请求，可在 CLI 实现。
- **Hermes/OpenClaw 专属配置**：命令较多，需逐一封装 CLI 子命令。

### 4.3 改造困难 / 不建议改造

- **系统托盘 / 窗口 / 桌面通知 / 剪贴板 / 文件对话框**：GUI 专属，CLI 不需要也无法实现。
- **自动启动**：桌面环境专属，服务器用 systemd 替代。
- **Deep link**：桌面协议注册，CLI 可选支持直接传入 URL 参数解析，但不注册系统协议。
- **更新检查/自更新**：桌面应用机制，服务器用包管理器。
- **Keychain**：Linux 已禁用，CLI 用配置文件存 API Key（注意权限 0600，`settings.json` 已有此保护）。

---

## 五、分阶段改造计划

### Phase 1：修复 CLI 缺陷 + 守护进程化（基础可用）

**目标**：让 CLI 能在服务器上稳定后台运行代理服务器，修复现有 bug。

| 任务 | 说明 | 涉及文件 |
|---|---|---|
| 1.1 修复 `stop` 路由 | 在 `proxy/server.rs` 的 `build_router()` 添加 `POST /stop` 路由，触发 shutdown 通道 | `proxy/server.rs`, `proxy/handlers.rs` |
| 1.2 CLI `stop` 改用 POST | `cmd_stop` 改为 `reqwest::Client.post()` | `bin/cc-switch-cli.rs` |
| 1.3 新增 `daemon` 模式 | `start --daemon` 或独立 `daemon` 子命令：fork 后台、写 PID 文件、重定向日志、捕获 SIGTERM/SIGHUP | `bin/cc-switch-cli.rs` |
| 1.4 PID 文件管理 | 写入/检查/删除 `~/.cc-switch/cc-switch.pid`，单实例保证 | `bin/cc-switch-cli.rs` |
| 1.5 信号处理 | SIGTERM → 优雅停止代理 + 恢复 Live 配置；SIGHUP → 重载配置 | `bin/cc-switch-cli.rs` |
| 1.6 日志集成 | CLI 模式集成 `env_logger` 或复用日志文件（`~/.cc-switch/logs/cc-switch.log`），支持 `--log-level` | `bin/cc-switch-cli.rs` |
| 1.7 systemd unit 模板 | 提供 `cc-switch.service` 模板，含 Restart 策略、环境变量 | `docs/` 或 `deploy/` |
| 1.8 CLI 启动时初始化 | 抽取 `lib.rs` setup 中的数据库初始化、迁移、Live 导入、代理状态恢复为可复用函数，CLI `start/daemon` 调用 | `lib.rs`, `bin/cc-switch-cli.rs` |

**验收标准**：`cc-switch-cli daemon` 可后台运行，`systemctl start cc-switch` 可管理，`cc-switch-cli stop` 可正常停止，代理能接管 Live 配置并恢复上次状态。

### Phase 2：补齐核心配置命令（P0-P1）

**目标**：CLI 能完成日常供应商与代理配置管理。

| 任务 | 说明 |
|---|---|
| 2.1 `settings` 子命令 | `settings get [key]` / `settings set <key> <value>` / `settings list`，直接操作 `settings.json`（AppSettings），支持目录覆盖、语言、同步开关等 |
| 2.2 供应商管理补齐 | `update-provider`（完整字段）、`list-providers` 支持全部 7 种 app、`add-provider` 支持模型映射/env/meta/category |
| 2.3 代理接管命令 | `takeover <app> on/off`、`proxy-config get/set`、`switch-proxy-provider <app> <id>` |
| 2.4 故障转移命令 | `failover queue list/add/remove`、`failover auto on/off`、`circuit-breaker get/set/reset <provider>` |
| 2.5 修正器/优化器命令 | `rectifier get/set`、`optimizer get/set` |
| 2.6 全局出站代理命令 | `global-proxy get/set/test/scan` |
| 2.7 用量同步 worker | daemon 模式下启动会话用量定期同步（复用 GUI 的 spawn 逻辑） |
| 2.8 WebDAV/S3 worker | daemon 模式下启动自动同步 worker |

**验收标准**：无需 GUI 即可完成供应商增删改查切换、代理接管与热切换、故障转移与熔断器配置、全局代理配置。

### Phase 3：补齐高级功能命令（P2-P3）

**目标**：CLI 覆盖剩余可改造功能。

| 任务 | 说明 |
|---|---|
| 3.1 MCP 管理命令 | `mcp list/upsert/delete/toggle/test/import` |
| 3.2 Skills 管理命令 | `skill list/install/uninstall/toggle/update/search` |
| 3.3 Prompts 管理命令 | `prompt list/upsert/delete/enable/import` |
| 3.4 用量统计命令 | `usage summary/trends/logs/stats` |
| 3.5 导入导出/备份命令 | `config export/import`、`backup create/list/restore/rename/delete` |
| 3.6 云同步命令 | `sync webdav test/upload/download`、`sync s3 test/upload/download` |
| 3.7 端点测速命令 | `speedtest`、`verify-key`、`fetch-models` |
| 3.8 会话管理命令 | `session list/messages/delete` |
| 3.9 流检查命令 | `stream-check <provider/all>` |
| 3.10 环境变量命令 | `env check/delete/restore` |
| 3.11 Hermes/OpenClaw 专属命令 | 按需封装各 get/set 子命令 |
| 3.12 OAuth 认证命令（可选） | `auth copilot login/status/list/remove`、`auth codex login/...`，设备码流程打印 URL |

**验收标准**：除 GUI 专属功能外，所有配置与管理操作均有 CLI 等价命令。

### Phase 4：配置文件驱动 + 运维完善

**目标**：支持纯配置文件驱动的部署，完善运维能力。

| 任务 | 说明 |
|---|---|
| 4.1 声明式配置文件 | 支持 `--config <file.yaml>` 启动时读取声明式配置，批量设置供应商/代理/熔断器等（类似 docker-compose） |
| 4.2 配置校验命令 | `cc-switch-cli validate <config.yaml>` 校验配置文件合法性 |
| 4.3 配置 diff | `cc-switch-cli config diff` 对比当前运行配置与文件配置的差异 |
| 4.4 热重载 | SIGHUP 触发配置重载，无需重启 |
| 4.5 健康检查端点 | 完善 `/health` 返回 JSON 状态（运行时间、当前供应商、熔断器状态） |
| 4.6 Prometheus 指标（可选） | `/metrics` 暴露请求数/延迟/错误率/熔断器状态 |
| 4.7 完整文档 | CLI 帮助、man page、服务器部署指南、systemd/supervisor 配置示例 |
| 4.8 集成测试 | 针对无 GUI Linux 环境的端到端测试 |

**验收标准**：可通过单一 YAML 配置文件完成全部部署，支持热重载，有完整运维文档。

---

## 六、技术方案要点

### 6.1 守护进程方案

推荐**双模式**设计：

- **前台模式**（`start`）：保持现有行为，前台阻塞，适合调试和容器前台进程。
- **守护模式**（`daemon` / `start --daemon`）：fork 子进程脱离终端，写 PID 文件，捕获信号，日志写文件。

信号处理：
- `SIGTERM` / `SIGINT`：优雅停止 — 停止代理、恢复 Live 配置（`stop_with_restore_keep_state`）、停止 worker、删除 PID 文件、退出。
- `SIGHUP`：重载配置（Phase 4）。

systemd unit 示例方向：
```ini
[Unit]
Description=CC Switch Headless Proxy
After=network.target

[Service]
Type=simple
ExecStart=/usr/bin/cc-switch-cli daemon
Restart=on-failure
RestartSec=5
User=ccswitch
Environment=CC_SWITCH_LISTEN=0.0.0.0
Environment=CC_SWITCH_PORT=9090

[Install]
WantedBy=multi-user.target
```

### 6.2 CLI 启动初始化复用

将 `lib.rs` 中 `setup` 闭包的初始化逻辑抽取为独立函数（如 `core::bootstrap::initialize_headless(db) -> AppState`），包含：

1. 数据库初始化与 schema 迁移
2. config.json → SQLite 迁移（如有）
3. 默认 Skills 仓库初始化
4. Skills SSOT 迁移
5. Live 配置导入 + 官方预设供应商 seed
6. OpenCode/OpenClaw/Hermes Live 导入
7. OMO 配置导入
8. MCP / Prompts 导入（表空时）
9. 通用配置片段提取
10. 全局出站代理 HTTP 客户端初始化
11. 代理状态恢复（`restore_proxy_state_on_startup`）
12. 异常退出恢复（清理接管残留）
13. 用量同步 worker 启动
14. WebDAV/S3 自动同步 worker 启动
15. 周期性备份 timer 启动

CLI `start`/`daemon` 调用此函数后，再启动 `ProxyServer`。

### 6.3 命令补齐策略

- **优先复用 service 层**：所有业务逻辑通过 `ProviderService`、`McpService`、`PromptService`、`ProxyService`、`ConfigService`、`SkillService`、`SpeedtestService` 等调用，避免重复实现。
- **输出格式**：默认人类可读表格，`--json` 输出 JSON 供脚本消费，`--quiet` 仅输出 ID/状态码。
- **API Key 处理**：CLI 模式下 API Key 通过命令参数（`--api-key`）或环境变量（`CC_SWITCH_API_KEY`）传入，存储在供应商配置中（DB），不依赖 Keychain。
- **交互式 vs 非交互式**：默认非交互式（适合脚本/自动化），可选 `--interactive` 进入向导模式。

### 6.4 配置文件方案（Phase 4）

声明式 YAML 配置示例方向：

```yaml
# cc-switch-config.yaml
proxy:
  listen: 0.0.0.0
  port: 9090
  takeover:
    claude: true
    codex: true
    gemini: false

providers:
  - app: claude
    id: my-anthropic
    name: My Anthropic
    env:
      ANTHROPIC_API_KEY: ${ANTHROPIC_API_KEY}
      ANTHROPIC_BASE_URL: https://api.anthropic.com
    current: true

failover:
  auto: true
  queue:
    claude: [my-anthropic, backup-provider]
  circuit_breaker:
    max_retries: 3
    failure_threshold: 5

global_proxy:
  url: socks5://127.0.0.1:1080

settings:
  language: zh
  backup_interval_hours: 24
```

---

## 七、风险与注意事项

1. **代理服务器 emit 事件的安全性**：`ProxyState.app_handle = None` 时，所有 `app.emit(...)` 调用必须判空。需全局审计 `proxy/` 目录中的 emit 调用，确保不会 panic。当前代码可能存在未判空的 emit 调用。

2. **Live 配置接管的副作用**：代理接管会改写 `~/.claude/settings.json` 等真实 CLI 工具配置文件。CLI 模式下若异常退出未恢复，会导致用户的 Claude Code/Codex/Gemini 配置处于损坏状态（含占位符）。daemon 模式必须有可靠的信号处理和崩溃恢复。

3. **数据库并发**：daemon 进程与 CLI 管理命令同时访问同一 SQLite 数据库。SQLite 支持并发读但写需串行。当前 GUI 模式下也是单进程访问，需确认 WAL 模式是否开启，以及 CLI 管理命令与 daemon 的锁竞争。

4. **OAuth Token 持久化**：Copilot/Codex OAuth token 存储在 `app_config_dir` 下的文件中。CLI 模式下需确保这些文件的读写路径与 GUI 一致，且权限正确。

5. **Linux WebKitGTK 依赖**：当前 `Cargo.toml` 在 Linux target 依赖 `webkit2gtk`。纯 CLI 二进制若要避免该依赖，可能需要条件编译或拆分 crate。否则即使在无 GUI 服务器上运行 CLI，编译时仍需 webkit2gtk 开发库。**这是一个需要重点评估的构建依赖问题**。

6. **Tauri 运行时依赖**：`cc_switch_lib` crate 类型为 `staticlib/cdylib/rlib`，CLI binary 依赖 lib。lib 中大量模块（tray、lightweight、deeplink 等）引用 Tauri 类型。虽然 CLI 只调用部分函数，但编译时整个 lib 都会参与链接。可能需要通过 feature flag 隔离 GUI 专属模块。

7. **测试覆盖**：无 GUI 环境的端到端测试需要单独的 CI 流程（无显示器的 Linux 容器）。

---

## 八、工作量估算

| 阶段 | 预估工作量 | 优先级 |
|---|---|---|
| Phase 1：修复缺陷 + 守护进程化 | 中（3-5 天） | 必须 |
| Phase 2：核心配置命令 | 中-大（5-8 天） | 高 |
| Phase 3：高级功能命令 | 大（8-12 天） | 中 |
| Phase 4：配置文件 + 运维 | 中（3-5 天） | 低（可按需） |
| **合计** | **约 19-30 天** | — |

> 估算基于熟悉代码库的开发者，不含测试与文档的额外时间。实际工作量取决于对 emit 判空审计、构建依赖梳理等前置调研的结果。

---

## 九、下一步建议

1. **请用户审阅本计划**，确认改造范围与优先级。
2. 若批准，建议先执行一项**前置调研**：审计 `proxy/` 目录中所有 `app_handle.emit()` 调用的判空情况，以及评估 webkit2gtk 构建依赖是否可条件化，这两项直接影响 Phase 1 的可行性与工作量。
3. 确认后从 Phase 1 开始实施，每个阶段完成后验收再进入下一阶段。

---

*本文档由代码全面检查生成，基于 `src-tauri/src/` 后端代码、`src/` 前端代码、`Cargo.toml` 依赖配置的静态分析。*
