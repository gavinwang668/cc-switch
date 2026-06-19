# CC Switch CLI デプロイメント＆使用ガイド（Windows / Linux）

> このドキュメントでは、異なるプラットフォームでCC Switch CLIをデプロイおよび使用する方法を説明します。Windowsデスクトップ環境とヘッドレスLinuxサーバーの両方をカバーします。

---

## 目次

- [1. 概要](#1-概要)
- [2. バイナリの取得](#2-バイナリの取得)
- [3. Windowsガイド](#3-windowsガイド)
- [4. Linuxガイド](#4-linuxガイド)
- [5. コマンドリファレンス](#5-コマンドリファレンス)
- [6. 宣言型設定ファイル](#6-宣言型設定ファイル)
- [7. FAQ](#7-faq)

---

## 1. 概要

CC Switch CLIは、**プロキシサーバー**と**管理ツール**の両方の役割を兼ね備えたコマンドラインプログラムです：

- **プロキシサーバー**：`start`または`daemon`コマンドでローカルHTTPプロキシサーバーを起動し、Claude Code / Codex / Gemini CLIなどのツールからのAPIリクエストを傍受して、マルチプロバイダールーティング、フェイルオーバー、使用量追跡などを実現します。
- **管理ツール**：`list-providers`、`switch-provider`、`settings`などのコマンドで設定を管理します。これらのコマンドは実行後に即座に終了し、プロキシサーバーの実行状態に依存しません。

両モードは同じローカルデータ（`~/.cc-switch/`ディレクトリ内の`cc-switch.db`データベースと`settings.json`設定ファイル）を操作します。プロキシサーバーは管理コマンドによる変更をリアルタイムに反映します。

### GUI版との関係

CC SwitchはGUIデスクトップアプリ（`cc-switch`）とCLIツール（`cc-switch-cli`）の両方を提供します。両者は同じバックエンドロジックとデータベースを共有し、交互に使用できます。CLIは以下の用途に特に適しています：

- GUIのないヘッドレスLinuxサーバー
- スクリプト化・自動化管理が必要なシナリオ
- システムサービスとしてバックグラウンド実行する必要があるシナリオ

---

## 2. バイナリの取得

### 方法1：プリビルドバイナリを使用

プロジェクトの`release/`ディレクトリからコンパイル済みバイナリを取得します：

| ファイル | プラットフォーム | 説明 |
|---|---|---|
| `cc-switch.exe` | Windows | GUIデスクトップアプリ |
| `cc-switch-cli.exe` | Windows | CLIツール |
| `cc-switch-cli-linux-x86_64` | Linux x86_64 | CLIツール |

### 方法2：ソースからビルド

#### Windows でビルド

前提条件：Node.js 22+、pnpm 8+、Rust 1.85+、Visual Studio C++ Build Tools。

```powershell
# 1. フロントエンドの依存関係をインストール
cd f:\workspace\trae\cc-switch
pnpm install

# 2. フロントエンドをビルド
pnpm exec vite build

# 3. CLIをビルド
cd src-tauri
cargo build --release --bin cc-switch-cli

# 4. GUIをビルド（オプション）
cargo build --release --bin cc-switch
```

出力パス：`src-tauri\target\release\cc-switch-cli.exe` および `cc-switch.exe`

#### Linux でビルド（WSLまたはネイティブLinux）

前提条件：Rust 1.85+、`libwebkit2gtk-4.1-dev`、`libgtk-3-dev`、`libayatana-appindicator3-dev`。

```bash
# システム依存関係をインストール（Ubuntu/Debian）
sudo apt install -y libwebkit2gtk-4.1-dev libgtk-3-dev libayatana-appindicator3-dev

# Rust環境を読み込み
source ~/.cargo/env

# プロジェクトディレクトリに移動
cd /path/to/cc-switch/src-tauri

# フロントエンドがビルド済みであることを確認（distディレクトリが存在）
# Windowsマウントディレクトリを使用している場合、distが既に存在すればスキップ

# CLIをビルド
cargo build --release --bin cc-switch-cli
```

出力パス：`src-tauri/target/release/cc-switch-cli`

> **注意**：CLIのみをビルドする場合でも、CLIとGUIが同じlibクレートを共有しているため、webkit2gtk開発ライブラリが必要です。実行時にはwebkit2gtkは不要です。

---

## 3. Windowsガイド

### 3.1 クイックスタート

`cc-switch-cli.exe`を任意のディレクトリ（例：`C:\Tools\cc-switch\`）に配置し、PATHに追加するか完全パスで使用します。

```powershell
# ヘルプを表示
cc-switch-cli.exe help

# 現在のステータスを確認
cc-switch-cli.exe status

# プロバイダー一覧を表示
cc-switch-cli.exe list-providers
```

### 3.2 フォアグラウンドでプロキシを実行

```powershell
# プロキシを起動（フォアグラウンド、Ctrl+Cで停止）
cc-switch-cli.exe start

# リッスンアドレスとポートをカスタマイズ
$env:CC_SWITCH_LISTEN = "0.0.0.0"
$env:CC_SWITCH_PORT = "8080"
cc-switch-cli.exe start
```

### 3.3 バックグラウンドでプロキシを実行

```powershell
# デーモンとしてバックグラウンド起動
cc-switch-cli.exe daemon
# 出力：プロキシサーバーがバックグラウンドで起動しました (PID: 12345)
# 出力：ログファイル: C:\Users\YourName\.cc-switch\cc-switch-daemon.log

# ステータスを確認
cc-switch-cli.exe status

# バックグラウンドプロキシを停止
cc-switch-cli.exe stop
```

### 3.4 日常管理

```powershell
# プロバイダーを追加
cc-switch-cli.exe add-provider claude my-provider "My Provider" --api-key sk-ant-xxx --base-url https://api.anthropic.com

# 現在のプロバイダーを切り替え
cc-switch-cli.exe switch-provider claude my-provider

# Claudeのプロキシテイクオーバーを有効化
cc-switch-cli.exe takeover claude on

# プロキシモードでプロバイダーをホットスイッチ
cc-switch-cli.exe switch-proxy claude backup-provider
```

### 3.5 GUIと併用

Windowsでは、GUIアプリとCLIツールの両方をインストールできます。両者は同じ`~/.cc-switch/`設定ディレクトリを共有し、操作は完全に相互運用可能です：

- GUIで追加したプロバイダーはCLIからも確認可能
- CLIで切り替えたプロバイダーはGUIの再起動後に反映
- GUIで初期設定を行い、CLIで自動化管理が可能

### 3.6 起動時の自動実行（オプション）

タスクスケジューラを使用してデーモンの自動起動を設定します：

```powershell
# 自動起動タスクを作成
$action = New-ScheduledTaskAction -Execute "C:\Tools\cc-switch\cc-switch-cli.exe" -Argument "daemon"
$trigger = New-ScheduledTaskTrigger -AtLogon
Register-ScheduledTask -TaskName "CC Switch" -Action $action -Trigger $trigger
```

---

## 4. Linuxガイド

### 4.1 インストール

```bash
# バイナリをシステムパスにコピー
sudo cp cc-switch-cli-linux-x86_64 /usr/local/bin/cc-switch-cli
sudo chmod +x /usr/local/bin/cc-switch-cli

# 専用ユーザーを作成（推奨、rootでの実行を避ける）
sudo useradd -r -s /sbin/nologin ccswitch

# 設定ディレクトリを作成
sudo mkdir -p /home/ccswitch/.cc-switch
sudo chown ccswitch:ccswitch /home/ccswitch/.cc-switch
```

### 4.2 systemd でサービスを管理

プロジェクトはsystemdユニットテンプレート（`deploy/cc-switch.service`）を提供しています。以下の手順でデプロイします：

```bash
# serviceファイルをコピー
sudo cp deploy/cc-switch.service /etc/systemd/system/

# 必要に応じて編集
sudo vim /etc/systemd/system/cc-switch.service
```

serviceファイルの参考内容：

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

有効化と管理：

```bash
# systemd設定をリロード
sudo systemctl daemon-reload

# 起動時の自動起動を有効化
sudo systemctl enable cc-switch

# サービスを開始
sudo systemctl start cc-switch

# ステータスを確認
sudo systemctl status cc-switch

# ログを表示
sudo journalctl -u cc-switch -f

# サービスを再起動
sudo systemctl restart cc-switch

# サービスを停止
sudo systemctl stop cc-switch
```

### 4.3 日常管理

サービス実行後、CLIコマンドで設定を管理します。これらのコマンドにroot権限は不要です（設定ディレクトリにアクセスできれば十分）：

```bash
# プロキシステータスを確認
cc-switch-cli status

# 全プロバイダー一覧を表示
cc-switch-cli list-providers

# プロバイダーを追加
cc-switch-cli add-provider claude my-provider "My Provider" \
  --api-key sk-ant-xxx \
  --base-url https://api.anthropic.com

# プロバイダーを切り替え
cc-switch-cli switch-provider claude my-provider

# Claudeのプロキシテイクオーバーを有効化
cc-switch-cli takeover claude on

# 自動フェイルオーバーを設定
cc-switch-cli auto-failover claude on
cc-switch-cli failover-queue add claude backup-provider
```

> **権限のヒント**：サービスが`ccswitch`ユーザーで実行されている場合、管理コマンドを別のユーザーで実行する際は、両者が`~/.cc-switch/`ディレクトリにアクセスできることを確認してください。同じユーザーを使用することを推奨します。

### 4.4 CLIの設定ディレクトリを正しく指定

デフォルトではCLIは`~/.cc-switch/`ディレクトリを読み書きします。サービスが`ccswitch`ユーザーで実行されている場合、設定ディレクトリは`/home/ccswitch/.cc-switch/`になります。管理コマンドも同じディレクトリを指す必要があります：

```bash
# 方法1：ccswitchユーザーとして管理コマンドを実行
sudo -u ccswitch cc-switch-cli list-providers

# 方法2：環境変数でオーバーライド（現在未サポート）
# 現在のバージョンでは環境変数オーバーライドをサポートしていません。方法1を使用してください
```

### 4.5 ログの確認

デーモンモードでは、ログは2箇所に出力されます：

```bash
# 方法1：journalctl（systemd管理時）
sudo journalctl -u cc-switch -f

# 方法2：ログファイル
tail -f /home/ccswitch/.cc-switch/cc-switch-daemon.log
```

### 4.6 Claude Code / Codex / Gemini CLI と併用

プロキシサーバーが起動してテイクオーバーすると、各CLIツールのリクエストは自動的にプロキシ経由になります。プロキシはデフォルトで`127.0.0.1:9090`でリッスンし、テイクオーバー時に各CLIツールのライブ設定ファイル（例：`~/.claude/settings.json`）を自動的に変更し、APIエンドポイントをローカルプロキシに向けます。CLIツールの手動設定は不要です。

手動で確認する場合：

```bash
# Claude Code: settings.jsonがテイクオーバーされているか確認
cat ~/.claude/settings.json
# テイクオーバー後、base_urlが http://127.0.0.1:9090 を指しているはず

# Codex: auth.jsonとconfig.tomlを確認
cat ~/.codex/auth.json
cat ~/.codex/config.toml

# Gemini: settings.jsonを確認
cat ~/.gemini/settings.json
```

---

## 5. コマンドリファレンス

### プロキシ管理

| コマンド | 説明 |
|---|---|
| `start` | フォアグラウンドでプロキシを起動（デバッグ用、Ctrl+Cで停止） |
| `daemon` | バックグラウンドでプロキシを起動（本番用） |
| `stop` | バックグラウンドプロキシを停止（HTTP POST /stopを送信） |
| `status` | プロキシの実行状態と現在のプロバイダーを表示 |

### プロバイダー管理

| コマンド | 説明 |
|---|---|
| `list-providers [APP]` | プロバイダー一覧を表示（APP省略時は全7種） |
| `add-provider APP ID NAME [--api-key K] [--base-url U]` | プロバイダーを追加 |
| `update-provider APP ID [--name N] [--api-key K] [--base-url U]` | プロバイダーを更新 |
| `remove-provider APP ID` | プロバイダーを削除 |
| `switch-provider APP ID` | 現在のプロバイダーを切り替え |

APPの値：`claude`、`claude-desktop`、`codex`、`gemini`、`opencode`、`openclaw`、`hermes`

### プロキシ設定

| コマンド | 説明 |
|---|---|
| `takeover APP [on\|off]` | プロキシテイクオーバーの表示 / 設定（claude/codex/geminiのみ） |
| `switch-proxy APP ID` | プロキシモードでプロバイダーをホットスイッチ |
| `failover-queue list\|add\|remove APP [ID]` | フェイルオーバーキューを管理 |
| `auto-failover [APP] [on\|off]` | 自動フェイルオーバーの表示 / 設定 |
| `circuit-breaker get\|set\|reset [APP] [--config JSON] [ID]` | サーキットブレーカー管理 |
| `rectifier get\|set [--config JSON]` | リクエストレクティファイア設定 |
| `optimizer get\|set [--config JSON]` | オプティマイザ設定 |
| `global-proxy get\|set\|clear\|test [URL]` | グローバル送信プロキシ |

### 設定

| コマンド | 説明 |
|---|---|
| `settings [KEY] [VALUE]` | デバイスレベル設定（`~/.cc-switch/settings.json`） |
| `config [--key K] [--value V]` | データベース設定（settingsテーブル） |
| `export-config PATH` | 設定をSQLファイルにエクスポート |
| `import-config PATH` | SQLファイルから設定をインポート |
| `validate PATH` | 宣言型YAML設定を検証 |
| `apply-config PATH` | 宣言型YAML設定をデータベースに適用 |

### バックアップと復元

| コマンド | 説明 |
|---|---|
| `backup-create` | データベースバックアップを作成 |
| `backup-list` | 全バックアップ一覧を表示 |
| `backup-restore NAME` | バックアップから復元 |

### その他

| コマンド | 説明 |
|---|---|
| `list-mcp` | MCPサーバー一覧を表示 |
| `list-prompts [APP]` | プロンプト一覧を表示 |
| `usage-summary [--days N]` | 使用量統計を表示 |
| `speedtest URL [--timeout S]` | APIエンドポイントのレイテンシをテスト |
| `verify-key --base-url U --api-key K` | APIキーを検証 |
| `help` | 完全なヘルプを表示 |

### 環境変数

| 変数 | デフォルト | 説明 |
|---|---|---|
| `CC_SWITCH_LISTEN` | `127.0.0.1` | プロキシのリッスンアドレス |
| `CC_SWITCH_PORT` | `9090` | プロキシのリッスンポート |

### グローバルオプション

- `--log-level <LEVEL>`：ログレベル（`error` / `warn` / `info` / `debug` / `trace`、デフォルト`info`）

---

## 6. 宣言型設定ファイル

一括設定やバージョン管理が必要なシナリオでは、YAML宣言型設定ファイルを使用できます。

### 6.1 設定ファイルフォーマット

```yaml
# プロキシサーバー
proxy:
  listen: "127.0.0.1"
  port: 9090
  takeover:
    claude: true
    codex: true
    gemini: false

# プロバイダーリスト
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

# フェイルオーバー
failover:
  auto: true
  queue:
    claude:
      - my-anthropic
      - backup-provider

# グローバル送信プロキシ（オプション）
# global_proxy:
#   url: "socks5://127.0.0.1:1080"

# デバイスレベル設定
settings:
  language: "ja"
  backup_interval_hours: 24
  backup_retain_count: 10
```

### 6.2 使用方法

```bash
# 設定ファイルを検証
cc-switch-cli validate config.yaml

# 設定をデータベースに適用
cc-switch-cli apply-config config.yaml
```

完全な例は `deploy/config.example.yaml` を参照してください。

---

## 7. FAQ

### Q: プロキシ起動後、Claude Code が接続できない？

以下を確認してください：

1. プロキシテイクオーバーが有効か確認：`cc-switch-cli takeover claude on`
2. Claude Codeの`~/.claude/settings.json`がローカルプロキシを指すように変更されているか確認
3. プロバイダー設定が正しいか確認：`cc-switch-cli list-providers claude`
4. プロキシログでエラーを確認

### Q: デーモンが起動直後に終了する？

1. ポートが使用中でないか確認：`netstat -tlnp | grep 9090`（Linux）または `netstat -ano | findstr 9090`（Windows）
2. PIDファイルが残留していないか確認：`~/.cc-switch/cc-switch-daemon.pid`を削除
3. ログファイルを確認：`~/.cc-switch/cc-switch-daemon.log`

### Q: 管理コマンドで「データベース初期化失敗」と表示される？

現在のユーザーが`~/.cc-switch/`ディレクトリへの読み書き権限を持っていることを確認してください。サービスが`ccswitch`ユーザーで実行されている場合、管理コマンドもそのユーザーで実行する必要があります：

```bash
sudo -u ccswitch cc-switch-cli list-providers
```

### Q: Linux でビルド時に「webkit2gtk not found」とエラーが出る？

webkit2gtk開発ライブラリをインストールしてください：

```bash
sudo apt install -y libwebkit2gtk-4.1-dev libgtk-3-dev
```

### Q: `stop` コマンドが成功と表示されるがプロセスが残っている？

`stop`は停止シグナルを送信した後、最大5秒間待機します。プロキシが多くのリクエストを処理中の場合、グレースフルシャットダウンに時間がかかることがあります。数秒待ってから`status`で確認するか、強制終了してください：

```bash
# Linux
sudo kill $(cat /home/ccswitch/.cc-switch/cc-switch-daemon.pid)

# Windows
taskkill /PID <PID> /F
```

### Q: CLI と GUI を同時に使用できますか？

はい。両者は同じ`~/.cc-switch/`設定ディレクトリとデータベースを共有します。GUIでの変更はCLIから確認でき、その逆も同様です。ただし、GUIのプロキシとCLIのデーモンを同時に実行するとポート競合が発生するため避けてください。

### Q: 設定を別のサーバーに移行するには？

```bash
# 移行元サーバー：設定をエクスポート
cc-switch-cli export-config /tmp/cc-switch-backup.sql

# 移行先サーバー：設定をインポート
cc-switch-cli import-config /path/to/cc-switch-backup.sql

# または設定ディレクトリ全体をコピー
scp -r ~/.cc-switch/ user@new-server:~/.cc-switch/
```

### Q: シグナル処理

デーモンモードでは以下のシグナルをサポートしています（Linuxのみ）：

| シグナル | 動作 |
|---|---|
| `SIGTERM` / `SIGINT` | グレースフルシャットダウン：ライブ設定を復元 → PIDファイルを削除 → 終了 |
| `SIGHUP` | ログ記録のみ（ホット設定リロードは未実装） |

Windowsでは`Ctrl+C`（SIGINT相当）のみサポートしています。
