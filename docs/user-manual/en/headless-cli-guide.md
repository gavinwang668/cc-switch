# CC Switch CLI Headless Mode Guide

> For headless Linux server environments, fully manage the CC Switch proxy server and configuration via the command line.

## Quick Start

### 1. Build the CLI

```bash
cd src-tauri
cargo build --release --bin cc-switch-cli
# Binary is at target/release/cc-switch-cli
```

### 2. Deploy with systemd

```bash
# Copy binary and service file
sudo cp target/release/cc-switch-cli /usr/bin/cc-switch-cli
sudo cp deploy/cc-switch.service /etc/systemd/system/

# Create dedicated user
sudo useradd -r -s /sbin/nologin ccswitch

# Start the service
sudo systemctl daemon-reload
sudo systemctl enable cc-switch
sudo systemctl start cc-switch

# Check status
sudo systemctl status cc-switch
sudo journalctl -u cc-switch -f
```

### 3. Use Declarative Config File

```bash
# Copy the example config
cp deploy/config.example.yaml /etc/cc-switch/config.yaml

# Edit config
vim /etc/cc-switch/config.yaml

# Validate config
cc-switch-cli validate /etc/cc-switch/config.yaml

# Apply config
cc-switch-cli apply-config /etc/cc-switch/config.yaml
```

## Command Reference

### Proxy Management

| Command | Description |
|---|---|
| `start` | Start proxy in foreground (for debugging) |
| `daemon` | Start proxy in background (for production) |
| `stop` | Stop the background proxy |
| `status` | View proxy status and current providers |

### Provider Management

| Command | Description |
|---|---|
| `list-providers [APP]` | List providers (all 7 apps supported) |
| `add-provider APP ID NAME [--api-key K] [--base-url U]` | Add a provider |
| `update-provider APP ID [--name N] [--api-key K] [--base-url U]` | Update a provider |
| `remove-provider APP ID` | Remove a provider |
| `switch-provider APP ID` | Switch the current provider |

### Proxy Configuration

| Command | Description |
|---|---|
| `takeover APP [on\|off]` | View / set proxy takeover |
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
| `apply-config PATH` | Apply a declarative YAML config |

### Backup & Restore

| Command | Description |
|---|---|
| `backup-create` | Create a database backup |
| `backup-list` | List backups |
| `backup-restore NAME` | Restore from a backup |

### Others

| Command | Description |
|---|---|
| `list-mcp` | List MCP servers |
| `list-prompts [APP]` | List prompts |
| `usage-summary [--days N]` | View usage statistics |
| `speedtest URL [--timeout S]` | Test API endpoint latency |
| `verify-key --base-url U --api-key K` | Verify an API key |
| `help` | Show help |

## Environment Variables

| Variable | Default | Description |
|---|---|---|
| `CC_SWITCH_LISTEN` | `127.0.0.1` | Proxy listen address |
| `CC_SWITCH_PORT` | `9090` | Proxy listen port |

## Global Options

- `--log-level <LEVEL>`: Log level (error/warn/info/debug/trace, default info)

## Signal Handling

The following signals are supported in daemon mode:

| Signal | Behavior |
|---|---|
| `SIGTERM` / `SIGINT` | Graceful shutdown: restore live config, delete PID file, exit |
| `SIGHUP` | Log only (hot config reload not yet implemented) |

## Declarative Config File Format

See `deploy/config.example.yaml`. Supports configuring:
- Proxy server (listen address, port, takeover toggles)
- Provider list (with env vars, current provider flag)
- Failover (auto toggle, queue config)
- Global outbound proxy
- Device-level settings (language, backup policy, config directories)

## Known Limitations

1. **WebDAV/S3 Auto-Sync**: The auto-sync worker is not started in daemon mode (requires AppHandle). You can manually use `export-config` / `import-config` with external sync tools.

2. **OAuth Authentication**: Copilot/Codex OAuth device flow is not yet implemented in the CLI. Providers using API Key are not affected.

3. **Usage Statistics**: The session usage sync worker is started in daemon mode, but the usage query feature is a simplified version.

4. **webkit2gtk Build Dependency**: Building the CLI on Linux still requires webkit2gtk development libraries (because the lib module references it). Not needed at runtime.

5. **GUI-Only Features**: The following features are only available in the GUI and not supported by the CLI:
   - System tray, desktop notifications, clipboard
   - File selection dialogs (CLI uses path arguments instead)
   - Auto-launch (use systemd instead)
   - Deep link import, auto-update
   - Keychain (disabled on Linux; API keys are stored in config files)
