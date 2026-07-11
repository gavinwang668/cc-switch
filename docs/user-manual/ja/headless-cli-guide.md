# CC Switch CLI ヘッドレスモードガイド

> GUIのないLinuxサーバー環境向け。コマンドラインからCC Switchプロキシサーバーと設定を完全に管理します。

## クイックスタート

### 1. CLIをビルド

```bash
# ワークスペースのルートで実行（src-tauri 内ではありません）
cargo build --release -p cc-switch-cli
# バイナリは target/release/cc-switch-cli にあります
```

### 2. systemd でデプロイ

```bash
# バイナリとserviceファイルをコピー
sudo cp target/release/cc-switch-cli /usr/bin/cc-switch-cli
sudo cp deploy/cc-switch.service /etc/systemd/system/

# 専用ユーザーを作成
sudo useradd -r -s /sbin/nologin ccswitch

# サービスを開始
sudo systemctl daemon-reload
sudo systemctl enable cc-switch
sudo systemctl start cc-switch

# ステータスを確認
sudo systemctl status cc-switch
sudo journalctl -u cc-switch -f
```

### 3. 宣言型設定ファイルを使用

```bash
# サンプル設定をコピー
cp deploy/config.example.yaml /etc/cc-switch/config.yaml

# 設定を編集
vim /etc/cc-switch/config.yaml

# 設定を検証
cc-switch-cli validate /etc/cc-switch/config.yaml

# 設定を適用
cc-switch-cli apply-config /etc/cc-switch/config.yaml
```

## コマンドリファレンス

### プロキシ管理

| コマンド | 説明 |
|---|---|
| `start` | フォアグラウンドでプロキシを起動（デバッグ用） |
| `daemon` | バックグラウンドでプロキシを起動（本番用） |
| `stop` | バックグラウンドプロキシを停止 |
| `status` | プロキシステータスと現在のプロバイダーを表示 |

### プロバイダー管理

| コマンド | 説明 |
|---|---|
| `list-providers [APP]` | プロバイダー一覧を表示（全7種サポート） |
| `add-provider APP ID NAME [--api-key K] [--base-url U]` | プロバイダーを追加 |
| `update-provider APP ID [--name N] [--api-key K] [--base-url U]` | プロバイダーを更新 |
| `remove-provider APP ID` | プロバイダーを削除 |
| `switch-provider APP ID` | 現在のプロバイダーを切り替え |

### プロキシ設定

| コマンド | 説明 |
|---|---|
| `takeover APP [on\|off]` | プロキシテイクオーバーの表示 / 設定 |
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
| `apply-config PATH` | 宣言型YAML設定を適用 |

### バックアップと復元

| コマンド | 説明 |
|---|---|
| `backup-create` | データベースバックアップを作成 |
| `backup-list` | バックアップ一覧を表示 |
| `backup-restore NAME` | バックアップから復元 |

### その他

| コマンド | 説明 |
|---|---|
| `list-mcp` | MCPサーバー一覧を表示 |
| `list-prompts [APP]` | プロンプト一覧を表示 |
| `usage-summary [--days N]` | 使用量統計を表示 |
| `speedtest URL [--timeout S]` | APIエンドポイントのレイテンシをテスト |
| `verify-key --base-url U --api-key K` | APIキーを検証 |
| `help` | ヘルプを表示 |

## 環境変数

| 変数 | デフォルト | 説明 |
|---|---|---|
| `CC_SWITCH_LISTEN` | `127.0.0.1` | プロキシのリッスンアドレス |
| `CC_SWITCH_PORT` | `9090` | プロキシのリッスンポート |

## グローバルオプション

- `--log-level <LEVEL>`：ログレベル（error/warn/info/debug/trace、デフォルト info）

## シグナル処理

デーモンモードでは以下のシグナルをサポートしています：

| シグナル | 動作 |
|---|---|
| `SIGTERM` / `SIGINT` | グレースフルシャットダウン：ライブ設定を復元、PIDファイルを削除、終了 |
| `SIGHUP` | ログ記録のみ（ホット設定リロードは未実装） |

## 宣言型設定ファイルフォーマット

`deploy/config.example.yaml` を参照してください。以下の設定が可能です：
- プロキシサーバー（リッスンアドレス、ポート、テイクオーバートグル）
- プロバイダーリスト（環境変数、現在のプロバイダーフラグ付き）
- フェイルオーバー（自動トグル、キュー設定）
- グローバル送信プロキシ
- デバイスレベル設定（言語、バックアップポリシー、設定ディレクトリ）

## 既知の制限

1. **WebDAV/S3 自動同期**：デーモンモードでは自動同期ワーカーは起動しません（AppHandleが必要）。`export-config` / `import-config` を外部同期ツールと組み合わせて手動で使用できます。

2. **OAuth 認証**：Copilot/Codex OAuthデバイスフローはCLIにまだ実装されていません。APIキーを使用するプロバイダーには影響しません。

3. **使用量統計**：デーモンモードではセッション使用量同期ワーカーが起動しますが、使用量クエリ機能は簡易版です。

4. **webkit2gtk ビルド依存**：LinuxでCLIをビルドするにはwebkit2gtk開発ライブラリが必要です（libモジュールが参照しているため）。実行時には不要です。

5. **GUI専用機能**：以下の機能はGUIでのみ利用可能で、CLIではサポートされていません：
   - システムトレイ、デスクトップ通知、クリップボード
   - ファイル選択ダイアログ（CLIはパス引数を使用）
   - 自動起動（systemdを使用）
   - Deep linkインポート、自動更新
   - Keychain（Linuxでは無効化、APIキーは設定ファイルに保存）
