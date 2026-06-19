# CC Switch CLI Deployment & Usage Guide (Windows / Linux)

> This document describes how to deploy and use CC Switch CLI on different platforms, covering both Windows desktop environments and headless Linux servers.

---

## Table of Contents

- [1. Overview](#1-overview)
- [2. Obtaining Binaries](#2-obtaining-binaries)
- [3. Windows Guide](#3-windows-guide)
- [4. Linux Guide](#4-linux-guide)
- [5. Command Reference](#5-command-reference)
- [6. Declarative Configuration File](#6-declarative-configuration-file)
- [7. FAQ](#7-faq)

---

## 1. Overview

CC Switch CLI is a command-line program that serves a dual role as both a **proxy server** and a **management tool**:

- **Proxy Server**: Start a local HTTP proxy server via the `start` or `daemon` command. It intercepts API requests from tools like Claude Code / Codex / Gemini CLI, providing multi-provider routing, failover, usage tracking, and more.
- **Management Tool**: Manage configuration via commands like `list-providers`, `switch-provider`, `settings`, etc. These commands execute and exit immediately, without depending on whether the proxy server is running.

Both modes operate on the same local data (the `cc-switch.db` database and `settings.json` config file under the `~/.cc-switch/` directory). The proxy server picks up configuration changes made by management commands in real time.

### Relationship with the GUI Version

CC Switch provides both a GUI desktop app (`cc-switch`) and a CLI tool (`cc-switch-cli`). They share the same backend logic and database and can be used interchangeably. The CLI is especially suited for:

- Headless Linux servers without a GUI
- Scripted / automated management scenarios
- Running as a system service in the background

---

## 2. Obtaining Binaries

### Option 1: Use Pre-built Binaries

Get the pre-compiled binaries from the project's `release/` directory:

| File | Platform | Description |
|---|---|---|
| `cc-switch.exe` | Windows | GUI desktop app |
| `cc-switch-cli.exe` | Windows | CLI tool |
| `cc-switch-cli-linux-x86_64` | Linux x86_64 | CLI tool |

### Option 2: Build from Source

#### Building on Windows

Prerequisites: Node.js 22+, pnpm 8+, Rust 1.85+, Visual Studio C++ Build Tools.

```powershell
# 1. Install frontend dependencies
cd f:\workspace\trae\cc-switch
pnpm install

# 2. Build the frontend
pnpm exec vite build

# 3. Build the CLI
cd src-tauri
cargo build --release --bin cc-switch-cli

# 4. Build the GUI (optional)
cargo build --release --bin cc-switch
```

Output path: `src-tauri\target\release\cc-switch-cli.exe` and `cc-switch.exe`

#### Building on Linux (WSL or native Linux)

Prerequisites: Rust 1.85+, `libwebkit2gtk-4.1-dev`, `libgtk-3-dev`, `libayatana-appindicator3-dev`.

```bash
# Install system dependencies (Ubuntu/Debian)
sudo apt install -y libwebkit2gtk-4.1-dev libgtk-3-dev libayatana-appindicator3-dev

# Load Rust environment
source ~/.cargo/env

# Navigate to the project directory
cd /path/to/cc-switch/src-tauri

# Ensure the frontend is built (dist directory exists)
# If using a Windows-mounted directory, skip if dist already exists

# Build the CLI
cargo build --release --bin cc-switch-cli
```

Output path: `src-tauri/target/release/cc-switch-cli`

> **Note**: Even when building only the CLI, webkit2gtk development libraries are currently required because the CLI and GUI share the same lib crate. webkit2gtk is not needed at runtime.

---

## 3. Windows Guide

### 3.1 Quick Start

Place `cc-switch-cli.exe` in any directory (e.g., `C:\Tools\cc-switch\`), then add it to your PATH or use the full path directly.

```powershell
# Show help
cc-switch-cli.exe help

# Check current status
cc-switch-cli.exe status

# List providers
cc-switch-cli.exe list-providers
```

### 3.2 Run Proxy in Foreground

```powershell
# Start the proxy (foreground, Ctrl+C to stop)
cc-switch-cli.exe start

# Customize listen address and port
$env:CC_SWITCH_LISTEN = "0.0.0.0"
$env:CC_SWITCH_PORT = "8080"
cc-switch-cli.exe start
```

### 3.3 Run Proxy in Background

```powershell
# Start as a daemon (background process)
cc-switch-cli.exe daemon
# Output: Proxy server started in background (PID: 12345)
# Output: Log file: C:\Users\YourName\.cc-switch\cc-switch-daemon.log

# Check status
cc-switch-cli.exe status

# Stop the background proxy
cc-switch-cli.exe stop
```

### 3.4 Daily Management

```powershell
# Add a provider
cc-switch-cli.exe add-provider claude my-provider "My Provider" --api-key sk-ant-xxx --base-url https://api.anthropic.com

# Switch the current provider
cc-switch-cli.exe switch-provider claude my-provider

# Enable proxy takeover for Claude
cc-switch-cli.exe takeover claude on

# Hot-switch provider in proxy mode
cc-switch-cli.exe switch-proxy claude backup-provider
```

### 3.5 Using Alongside the GUI

On Windows, you can install both the GUI app and the CLI tool. They share the same `~/.cc-switch/` config directory, and operations are fully interoperable:

- Providers added in the GUI are visible to the CLI
- Provider switches made via CLI are reflected in the GUI after restart
- You can do initial setup in the GUI, then automate management with the CLI

### 3.6 Auto-start on Boot (Optional)

Use Task Scheduler to auto-start the daemon on login:

```powershell
# Create an auto-start task
$action = New-ScheduledTaskAction -Execute "C:\Tools\cc-switch\cc-switch-cli.exe" -Argument "daemon"
$trigger = New-ScheduledTaskTrigger -AtLogon
Register-ScheduledTask -TaskName "CC Switch" -Action $action -Trigger $trigger
```

---

## 4. Linux Guide

### 4.1 Installation

```bash
# Copy the binary to a system path
sudo cp cc-switch-cli-linux-x86_64 /usr/local/bin/cc-switch-cli
sudo chmod +x /usr/local/bin/cc-switch-cli

# Create a dedicated user (recommended, avoid running as root)
sudo useradd -r -s /sbin/nologin ccswitch

# Create the config directory
sudo mkdir -p /home/ccswitch/.cc-switch
sudo chown ccswitch:ccswitch /home/ccswitch/.cc-switch
```

### 4.2 Managing the Service with systemd

The project provides a systemd unit template (`deploy/cc-switch.service`). Deploy it as follows:

```bash
# Copy the service file
sudo cp deploy/cc-switch.service /etc/systemd/system/

# Edit if needed
sudo vim /etc/systemd/system/cc-switch.service
```

Reference service file content:

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

Enable and manage:

```bash
# Reload systemd configuration
sudo systemctl daemon-reload

# Enable auto-start on boot
sudo systemctl enable cc-switch

# Start the service
sudo systemctl start cc-switch

# Check status
sudo systemctl status cc-switch

# View logs
sudo journalctl -u cc-switch -f

# Restart the service
sudo systemctl restart cc-switch

# Stop the service
sudo systemctl stop cc-switch
```

### 4.3 Daily Management

Once the service is running, use CLI commands to manage configuration. These commands do not require root privileges (as long as the config directory is accessible):

```bash
# Check proxy status
cc-switch-cli status

# List all providers
cc-switch-cli list-providers

# Add a provider
cc-switch-cli add-provider claude my-provider "My Provider" \
  --api-key sk-ant-xxx \
  --base-url https://api.anthropic.com

# Switch provider
cc-switch-cli switch-provider claude my-provider

# Enable Claude proxy takeover
cc-switch-cli takeover claude on

# Set up automatic failover
cc-switch-cli auto-failover claude on
cc-switch-cli failover-queue add claude backup-provider
```

> **Permission Note**: If the service runs as the `ccswitch` user while management commands are run as another user, ensure both can access the `~/.cc-switch/` directory. It is recommended to use the same user for both.

### 4.4 Pointing the CLI to the Correct Config

By default, the CLI reads and writes to the `~/.cc-switch/` directory. If the service runs as the `ccswitch` user, the config directory is `/home/ccswitch/.cc-switch/`. Management commands must also point to the same directory:

```bash
# Option 1: Run management commands as the ccswitch user
sudo -u ccswitch cc-switch-cli list-providers

# Option 2: Override via environment variable (not yet supported)
# The current version does not support env var override; use Option 1
```

### 4.5 Viewing Logs

In daemon mode, logs are written to two locations:

```bash
# Option 1: journalctl (when managed by systemd)
sudo journalctl -u cc-switch -f

# Option 2: Log file
tail -f /home/ccswitch/.cc-switch/cc-switch-daemon.log
```

### 4.6 Using with Claude Code / Codex / Gemini CLI

After the proxy server starts and takes over, requests from each CLI tool are automatically routed through the proxy. The proxy listens on `127.0.0.1:9090` by default. Upon takeover, it automatically modifies each CLI tool's live config file (e.g., `~/.claude/settings.json`) to point the API endpoint to the local proxy. No manual configuration of CLI tools is needed.

To manually verify:

```bash
# Claude Code: check if settings.json has been taken over
cat ~/.claude/settings.json
# After takeover, base_url should point to http://127.0.0.1:9090

# Codex: check auth.json and config.toml
cat ~/.codex/auth.json
cat ~/.codex/config.toml

# Gemini: check settings.json
cat ~/.gemini/settings.json
```

---

## 5. Command Reference

### Proxy Management

| Command | Description |
|---|---|
| `start` | Start proxy in foreground (for debugging, Ctrl+C to stop) |
| `daemon` | Start proxy in background (for production) |
| `stop` | Stop the background proxy (sends HTTP POST /stop) |
| `status` | View proxy status and current providers |

### Provider Management

| Command | Description |
|---|---|
| `list-providers [APP]` | List providers (all 7 apps if APP is omitted) |
| `add-provider APP ID NAME [--api-key K] [--base-url U]` | Add a provider |
| `update-provider APP ID [--name N] [--api-key K] [--base-url U]` | Update a provider |
| `remove-provider APP ID` | Remove a provider |
| `switch-provider APP ID` | Switch the current provider |

APP values: `claude`, `claude-desktop`, `codex`, `gemini`, `opencode`, `openclaw`, `hermes`

### Proxy Configuration

| Command | Description |
|---|---|
| `takeover APP [on\|off]` | View / set proxy takeover (claude/codex/gemini only) |
| `switch-proxy APP ID` | Hot-switch provider in proxy mode |
| `failover-queue list\|add\|remove APP [ID]` | Manage failover queue |
| `auto-failover [APP] [on\|off]` | View / set automatic failover |
| `circuit-breaker get\|set\|reset [APP] [--config JSON] [ID]` | Circuit breaker management |
| `rectifier get\|set [--config JSON]` | Request rectifier config |
| `optimizer get\|set [--config JSON]` | Optimizer config |
| `global-proxy get\|set\|clear\|test [URL]` | Global outbound proxy |

### Configuration & Settings

| Command | Description |
|---|---|
| `settings [KEY] [VALUE]` | Device-level settings (`~/.cc-switch/settings.json`) |
| `config [--key K] [--value V]` | Database config (settings table) |
| `export-config PATH` | Export config to an SQL file |
| `import-config PATH` | Import config from an SQL file |
| `validate PATH` | Validate a declarative YAML config |
| `apply-config PATH` | Apply a declarative YAML config to the database |

### Backup & Restore

| Command | Description |
|---|---|
| `backup-create` | Create a database backup |
| `backup-list` | List all backups |
| `backup-restore NAME` | Restore from a backup |

### Others

| Command | Description |
|---|---|
| `list-mcp` | List MCP servers |
| `list-prompts [APP]` | List prompts |
| `usage-summary [--days N]` | View usage statistics |
| `speedtest URL [--timeout S]` | Test API endpoint latency |
| `verify-key --base-url U --api-key K` | Verify an API key |
| `help` | Show full help |

### Environment Variables

| Variable | Default | Description |
|---|---|---|
| `CC_SWITCH_LISTEN` | `127.0.0.1` | Proxy listen address |
| `CC_SWITCH_PORT` | `9090` | Proxy listen port |

### Global Options

- `--log-level <LEVEL>`: Log level (`error` / `warn` / `info` / `debug` / `trace`, default `info`)

---

## 6. Declarative Configuration File

For scenarios requiring bulk configuration or version-controlled config, you can use a YAML declarative configuration file.

### 6.1 Config File Format

```yaml
# Proxy server
proxy:
  listen: "127.0.0.1"
  port: 9090
  takeover:
    claude: true
    codex: true
    gemini: false

# Provider list
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

# Failover
failover:
  auto: true
  queue:
    claude:
      - my-anthropic
      - backup-provider

# Global outbound proxy (optional)
# global_proxy:
#   url: "socks5://127.0.0.1:1080"

# Device-level settings
settings:
  language: "en"
  backup_interval_hours: 24
  backup_retain_count: 10
```

### 6.2 Usage

```bash
# Validate the config file
cc-switch-cli validate config.yaml

# Apply the config to the database
cc-switch-cli apply-config config.yaml
```

See `deploy/config.example.yaml` for a complete example.

---

## 7. FAQ

### Q: Claude Code can't connect after the proxy starts?

Check the following:

1. Confirm proxy takeover is enabled: `cc-switch-cli takeover claude on`
2. Confirm Claude Code's `~/.claude/settings.json` has been modified to point to the local proxy
3. Confirm the provider config is correct: `cc-switch-cli list-providers claude`
4. Check the proxy logs for errors

### Q: The daemon exits immediately after starting?

1. Check if the port is in use: `netstat -tlnp | grep 9090` (Linux) or `netstat -ano | findstr 9090` (Windows)
2. Check for a stale PID file: delete `~/.cc-switch/cc-switch-daemon.pid`
3. Check the log file: `~/.cc-switch/cc-switch-daemon.log`

### Q: Management commands report "database initialization failed"?

Ensure the current user has read/write permissions on the `~/.cc-switch/` directory. If the service runs as the `ccswitch` user, management commands must also run as that user:

```bash
sudo -u ccswitch cc-switch-cli list-providers
```

### Q: Build fails on Linux with "webkit2gtk not found"?

Install webkit2gtk development libraries:

```bash
sudo apt install -y libwebkit2gtk-4.1-dev libgtk-3-dev
```

### Q: The `stop` command shows success but the process is still running?

`stop` sends a shutdown signal and waits up to 5 seconds. If the proxy is handling many requests, graceful shutdown may take longer. Wait a few seconds and verify with `status`, or force-kill:

```bash
# Linux
sudo kill $(cat /home/ccswitch/.cc-switch/cc-switch-daemon.pid)

# Windows
taskkill /PID <PID> /F
```

### Q: Can I use the CLI and GUI at the same time?

Yes. They share the same `~/.cc-switch/` config directory and database. Changes made in the GUI are visible to the CLI and vice versa. However, avoid running the GUI proxy and the CLI daemon simultaneously to prevent port conflicts.

### Q: How do I migrate config to another server?

```bash
# Source server: export config
cc-switch-cli export-config /tmp/cc-switch-backup.sql

# Target server: import config
cc-switch-cli import-config /path/to/cc-switch-backup.sql

# Or simply copy the entire config directory
scp -r ~/.cc-switch/ user@new-server:~/.cc-switch/
```

### Q: Signal Handling

In daemon mode, the following signals are supported (Linux only):

| Signal | Behavior |
|---|---|
| `SIGTERM` / `SIGINT` | Graceful shutdown: restore live config → delete PID file → exit |
| `SIGHUP` | Log only (hot config reload not yet implemented) |

On Windows, only `Ctrl+C` (equivalent to SIGINT) is supported.
