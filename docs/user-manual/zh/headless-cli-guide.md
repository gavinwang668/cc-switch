# CC Switch CLI 无头模式使用指南

> 适用于无 GUI 的 Linux 服务器环境，通过命令行完全管理 CC Switch 代理服务器和配置。

## 快速开始

### 1. 编译 CLI

```bash
cd src-tauri
cargo build --release --bin cc-switch-cli
# 二进制位于 target/release/cc-switch-cli
```

### 2. 使用 systemd 部署

```bash
# 复制二进制和 service 文件
sudo cp target/release/cc-switch-cli /usr/bin/cc-switch-cli
sudo cp deploy/cc-switch.service /etc/systemd/system/

# 创建专用用户
sudo useradd -r -s /sbin/nologin ccswitch

# 启动服务
sudo systemctl daemon-reload
sudo systemctl enable cc-switch
sudo systemctl start cc-switch

# 查看状态
sudo systemctl status cc-switch
sudo journalctl -u cc-switch -f
```

### 3. 使用声明式配置文件

```bash
# 复制示例配置
cp deploy/config.example.yaml /etc/cc-switch/config.yaml

# 编辑配置
vim /etc/cc-switch/config.yaml

# 校验配置
cc-switch-cli validate /etc/cc-switch/config.yaml

# 应用配置
cc-switch-cli apply-config /etc/cc-switch/config.yaml
```

## 命令一览

### 代理管理

| 命令 | 说明 |
|---|---|
| `start` | 前台启动代理（调试用） |
| `daemon` | 后台启动代理（生产用） |
| `stop` | 停止后台代理 |
| `status` | 查看代理状态和当前供应商 |

### 供应商管理

| 命令 | 说明 |
|---|---|
| `list-providers [APP]` | 列出供应商（支持全部 7 种 app） |
| `add-provider APP ID NAME [--api-key K] [--base-url U]` | 添加供应商 |
| `update-provider APP ID [--name N] [--api-key K] [--base-url U]` | 更新供应商 |
| `remove-provider APP ID` | 删除供应商 |
| `switch-provider APP ID` | 切换当前供应商 |

### 代理配置

| 命令 | 说明 |
|---|---|
| `takeover APP [on\|off]` | 查看/设置代理接管 |
| `switch-proxy APP ID` | 代理模式下热切换供应商 |
| `failover-queue list\|add\|remove APP [ID]` | 管理故障转移队列 |
| `auto-failover [APP] [on\|off]` | 查看/设置自动故障转移 |
| `circuit-breaker get\|set\|reset [APP] [--config JSON] [ID]` | 熔断器管理 |
| `rectifier get\|set [--config JSON]` | 请求修正器配置 |
| `optimizer get\|set [--config JSON]` | 优化器配置 |
| `global-proxy get\|set\|clear\|test [URL]` | 全局出站代理 |

### 配置与设置

| 命令 | 说明 |
|---|---|
| `settings [KEY] [VALUE]` | 设备级设置 (`~/.cc-switch/settings.json`) |
| `config [--key K] [--value V]` | 数据库配置 (settings 表) |
| `export-config PATH` | 导出配置到 SQL 文件 |
| `import-config PATH` | 从 SQL 文件导入配置 |
| `validate PATH` | 校验声明式 YAML 配置 |
| `apply-config PATH` | 应用声明式 YAML 配置 |

### 备份与恢复

| 命令 | 说明 |
|---|---|
| `backup-create` | 创建数据库备份 |
| `backup-list` | 列出备份 |
| `backup-restore NAME` | 从备份恢复 |

### 其他

| 命令 | 说明 |
|---|---|
| `list-mcp` | 列出 MCP 服务器 |
| `list-prompts [APP]` | 列出 Prompts |
| `usage-summary [--days N]` | 查看用量统计 |
| `speedtest URL [--timeout S]` | 测试 API 端点延迟 |
| `verify-key --base-url U --api-key K` | 验证 API Key |
| `help` | 显示帮助信息 |

## 环境变量

| 变量 | 默认值 | 说明 |
|---|---|---|
| `CC_SWITCH_LISTEN` | `127.0.0.1` | 代理监听地址 |
| `CC_SWITCH_PORT` | `9090` | 代理监听端口 |

## 全局选项

- `--log-level <LEVEL>`: 日志级别 (error/warn/info/debug/trace，默认 info)

## 信号处理

daemon 模式下支持以下信号：

| 信号 | 行为 |
|---|---|
| `SIGTERM` / `SIGINT` | 优雅停止代理，恢复 Live 配置，删除 PID 文件，退出 |
| `SIGHUP` | 记录日志（配置热重载暂未实现） |

## 声明式配置文件格式

参见 `deploy/config.example.yaml`。支持配置：
- 代理服务器（监听地址、端口、接管开关）
- 供应商列表（含环境变量、当前供应商标记）
- 故障转移（自动开关、队列配置）
- 全局出站代理
- 设备级设置（语言、备份策略、配置目录）

## 已知限制

1. **WebDAV/S3 自动同步**: daemon 模式下暂不启动自动同步 worker（需要 AppHandle）。可手动使用 `export-config` / `import-config` 配合外部工具同步。

2. **OAuth 认证**: Copilot/Codex OAuth 设备码流程暂未在 CLI 中实现。使用 API Key 方式的供应商不受影响。

3. **用量统计**: daemon 模式下会启动会话用量同步 worker，但用量统计查询功能为简化版。

4. **webkit2gtk 构建依赖**: 在 Linux 上编译 CLI 仍需安装 webkit2gtk 开发库（因为 lib 模块引用了它）。运行时不需要。

5. **GUI 专属功能**: 以下功能仅在 GUI 中可用，CLI 不支持：
   - 系统托盘、桌面通知、剪贴板
   - 文件选择对话框（CLI 使用路径参数替代）
   - 自动启动（使用 systemd 替代）
   - Deep link 导入、自动更新
   - Keychain（Linux 上已禁用，API Key 存在配置文件中）
