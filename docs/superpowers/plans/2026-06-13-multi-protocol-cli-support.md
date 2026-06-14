# CC Switch 多协议支持与 Linux CLI 模式实施计划

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 实现完整的 API 协议转换矩阵（支持任意客户端调用任意模型后端）和 Linux 无头服务器 CLI 模式

**Architecture:** 
- 后端：基于现有的 transform 模块扩展，为 Gemini/Claude Desktop 添加 API 格式选择支持
- CLI：创建独立的 `cc-switch-cli` 二进制，复用核心代理和数据库逻辑，提供命令行管理界面
- 前端：扩展现有表单组件，添加 API 格式选择器

**Tech Stack:** Rust (Tauri), TypeScript/React, SQLite, Axum, Clap (CLI)

---

## 当前状态分析

### 已实现的转换函数（后端）
✅ `transform.rs`: Anthropic ↔ OpenAI Chat Completions  
✅ `transform_responses.rs`: Anthropic ↔ OpenAI Responses API  
✅ `transform_gemini.rs`: Anthropic ↔ Gemini Native  
✅ `transform_bedrock.rs`: Anthropic ↔ Bedrock Converse API  
✅ `transform_codex_chat.rs`: Codex Responses ↔ Chat Completions  

### 前端 API 格式支持现状
- **Claude**: ✅ 支持 apiFormat 选择器（anthropic/openai_chat/openai_responses/gemini_native/bedrock）
- **Codex**: ✅ 支持 apiFormat 选择器（openai_responses/openai_chat）
- **Gemini**: ❌ 无 apiFormat 选择器（固定使用 Gemini Native）
- **Claude Desktop**: ❌ 无 apiFormat 选择器（固定使用 Anthropic）
- **OpenCode/OpenClaw/Hermes**: 各自独立配置体系，不使用 apiFormat

### Linux CLI 模式
❌ 完全缺失，需要从头实现

---

## Phase 1: Gemini 应用 API 格式选择支持

### Task 1.1: 扩展 Gemini 类型定义

**Files:**
- Modify: `src/types.ts:200-250`

- [ ] **Step 1: 添加 GeminiApiFormat 类型**

在 `src/types.ts` 中找到 `CodexApiFormat` 定义位置（约 250 行），在其后添加：

```typescript
// Gemini API 格式类型
// - "gemini_native": Gemini Native generateContent API 格式，直接透传
// - "openai_chat": OpenAI Chat Completions 格式，需要格式转换
// - "openai_responses": OpenAI Responses API 格式，需要格式转换
// - "anthropic": Anthropic Messages API 格式，需要格式转换
export type GeminiApiFormat = 
  | "gemini_native"
  | "openai_chat"
  | "openai_responses"
  | "anthropic";
```

- [ ] **Step 2: 在 ProviderMeta 中添加 geminiApiFormat 字段**

在 `ProviderMeta` 接口（约 177-228 行）中，找到 `apiFormat` 字段（205 行），在其后添加：

```typescript
  // Gemini API 格式（Gemini 供应商使用）
  // - "gemini_native": Gemini Native generateContent API 格式，直接透传
  // - "openai_chat": OpenAI Chat Completions 格式，需要格式转换
  // - "openai_responses": OpenAI Responses API 格式，需要格式转换
  // - "anthropic": Anthropic Messages API 格式，需要格式转换
  geminiApiFormat?: GeminiApiFormat;
```

- [ ] **Step 3: 提交**

```bash
git add src/types.ts
git commit -m "feat(types): add GeminiApiFormat type definition"
```

---

### Task 1.2: 创建 Gemini 表单 API 格式选择器

**Files:**
- Modify: `src/components/providers/forms/GeminiFormFields.tsx`

- [ ] **Step 1: 读取 GeminiFormFields.tsx 完整内容**

```bash
wc -l src/components/providers/forms/GeminiFormFields.tsx
```

预期：约 200-300 行

- [ ] **Step 2: 在组件 props 中添加 geminiApiFormat 相关回调**

在文件顶部找到 `GeminiFormFieldsProps` 接口定义，添加：

```typescript
  // Gemini API 格式
  geminiApiFormat?: GeminiApiFormat;
  onGeminiApiFormatChange?: (format: GeminiApiFormat) => void;
```

- [ ] **Step 3: 在表单中添加 API 格式选择器 UI**

在表单的合适位置（通常在 API Key 输入框之后），添加：

```tsx
{/* Gemini API 格式选择 */}
<div className="space-y-2">
  <FormLabel>API 格式</FormLabel>
  <Select
    value={geminiApiFormat || "gemini_native"}
    onValueChange={(value) => onGeminiApiFormatChange?.(value as GeminiApiFormat)}
  >
    <SelectTrigger>
      <SelectValue />
    </SelectTrigger>
    <SelectContent>
      <SelectItem value="gemini_native">
        Gemini Native（原生格式）
      </SelectItem>
      <SelectItem value="openai_chat">
        OpenAI Chat Completions
      </SelectItem>
      <SelectItem value="openai_responses">
        OpenAI Responses API
      </SelectItem>
      <SelectItem value="anthropic">
        Anthropic Messages API
      </SelectItem>
    </SelectContent>
  </Select>
  <p className="text-xs text-muted-foreground">
    选择供应商支持的 API 格式。CC Switch 会自动转换请求和响应格式。
  </p>
</div>
```

- [ ] **Step 4: 提交**

```bash
git add src/components/providers/forms/GeminiFormFields.tsx
git commit -m "feat(gemini): add API format selector to Gemini form"
```

---

### Task 1.3: 后端 Gemini API 格式处理

**Files:**
- Modify: `src-tauri/src/proxy/providers/gemini.rs`
- Modify: `src-tauri/src/proxy/handlers.rs`

- [ ] **Step 1: 在 gemini.rs 中添加 get_gemini_api_format 函数**

在 `src-tauri/src/proxy/providers/gemini.rs` 文件顶部（约 50 行后）添加：

```rust
/// 获取 Gemini 供应商的 API 格式
///
/// 从 provider.meta.geminiApiFormat 读取，默认返回 "gemini_native"
pub fn get_gemini_api_format(provider: &Provider) -> String {
    provider
        .meta
        .as_ref()
        .and_then(|meta| meta.get("geminiApiFormat"))
        .and_then(|v| v.as_str())
        .unwrap_or("gemini_native")
        .to_string()
}

/// 判断是否需要格式转换
pub fn gemini_api_format_needs_transform(format: &str) -> bool {
    format != "gemini_native"
}
```

- [ ] **Step 2: 在 handlers.rs 中添加 Gemini 格式转换逻辑**

在 `handle_gemini_request` 函数中（需要先定位该函数），添加格式转换分支：

```rust
// 在发送请求前检查是否需要格式转换
let api_format = get_gemini_api_format(&provider);
if gemini_api_format_needs_transform(&api_format) {
    // 根据 api_format 调用对应的转换函数
    let transformed_body = match api_format.as_str() {
        "openai_chat" => {
            // Gemini Native -> OpenAI Chat
            // 注意：这里需要反向转换，因为客户端发送的是 Gemini 格式
            // 但实际场景中 Gemini 客户端通常只用 gemini_native
            // 这个分支主要用于测试和特殊场景
            unimplemented!("Gemini to OpenAI Chat conversion not yet implemented")
        }
        "openai_responses" => {
            unimplemented!("Gemini to OpenAI Responses conversion not yet implemented")
        }
        "anthropic" => {
            // Gemini Native -> Anthropic Messages
            // 同样，这个场景很少见
            unimplemented!("Gemini to Anthropic conversion not yet implemented")
        }
        _ => body.clone(),
    };
    // 使用 transformed_body 发送请求
}
```

**注意：** 实际上 Gemini 客户端（如 Gemini CLI）通常只发送 Gemini Native 格式，所以反向转换的需求很低。这个功能主要是为了完整性。

- [ ] **Step 3: 提交**

```bash
git add src-tauri/src/proxy/providers/gemini.rs src-tauri/src/proxy/handlers.rs
git commit -m "feat(gemini): add backend API format handling"
```

---

## Phase 2: Claude Desktop API 格式选择支持

### Task 2.1: 扩展 Claude Desktop 类型定义

**Files:**
- Modify: `src/types.ts`

- [ ] **Step 1: 添加 ClaudeDesktopApiFormat 类型**

在 `GeminiApiFormat` 定义后添加：

```typescript
// Claude Desktop API 格式类型
// - "anthropic": Anthropic Messages API 格式，直接透传
// - "openai_chat": OpenAI Chat Completions 格式，需要格式转换
// - "openai_responses": OpenAI Responses API 格式，需要格式转换
// - "gemini_native": Gemini Native API 格式，需要格式转换
// - "bedrock": Amazon Bedrock Converse API 格式，需要格式转换
export type ClaudeDesktopApiFormat = 
  | "anthropic"
  | "openai_chat"
  | "openai_responses"
  | "gemini_native"
  | "bedrock";
```

- [ ] **Step 2: 在 ProviderMeta 中添加 claudeDesktopApiFormat 字段**

```typescript
  // Claude Desktop API 格式
  claudeDesktopApiFormat?: ClaudeDesktopApiFormat;
```

- [ ] **Step 3: 提交**

```bash
git add src/types.ts
git commit -m "feat(types): add ClaudeDesktopApiFormat type definition"
```

---

### Task 2.2: 创建 Claude Desktop 表单 API 格式选择器

**Files:**
- Modify: `src/components/providers/forms/ClaudeDesktopProviderForm.tsx`

- [ ] **Step 1: 读取 ClaudeDesktopProviderForm.tsx**

```bash
wc -l src/components/providers/forms/ClaudeDesktopProviderForm.tsx
```

- [ ] **Step 2: 添加 API 格式选择器**

参考 Task 1.2 的实现，在 Claude Desktop 表单中添加类似的 API 格式选择器，选项包括：
- Anthropic（默认）
- OpenAI Chat Completions
- OpenAI Responses API
- Gemini Native
- Amazon Bedrock

- [ ] **Step 3: 提交**

```bash
git add src/components/providers/forms/ClaudeDesktopProviderForm.tsx
git commit -m "feat(claude-desktop): add API format selector"
```

---

### Task 2.3: 后端 Claude Desktop API 格式处理

**Files:**
- Modify: `src-tauri/src/proxy/providers/claude.rs`

- [ ] **Step 1: 添加 get_claude_desktop_api_format 函数**

```rust
/// 获取 Claude Desktop 供应商的 API 格式
pub fn get_claude_desktop_api_format(provider: &Provider) -> String {
    provider
        .meta
        .as_ref()
        .and_then(|meta| meta.get("claudeDesktopApiFormat"))
        .and_then(|v| v.as_str())
        .unwrap_or("anthropic")
        .to_string()
}
```

- [ ] **Step 2: 在请求处理中添加格式转换**

类似 Task 1.3，在 Claude Desktop 的请求处理流程中添加格式转换逻辑。

- [ ] **Step 3: 提交**

```bash
git add src-tauri/src/proxy/providers/claude.rs
git commit -m "feat(claude-desktop): add backend API format handling"
```

---

## Phase 3: Linux CLI 模式实现

### Task 3.1: 创建 CLI 二进制入口

**Files:**
- Create: `src-tauri/src/bin/cc-switch-cli.rs`
- Modify: `src-tauri/Cargo.toml`

- [ ] **Step 1: 在 Cargo.toml 中添加 CLI 二进制目标**

在 `[[bin]]` 部分添加：

```toml
[[bin]]
name = "cc-switch-cli"
path = "src/bin/cc-switch-cli.rs"
```

在 `[dependencies]` 中添加：

```toml
clap = { version = "4.5", features = ["derive"] }
tokio = { version = "1.36", features = ["full"] }
```

- [ ] **Step 2: 创建 CLI 入口文件**

```rust
// src-tauri/src/bin/cc-switch-cli.rs

use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "cc-switch-cli")]
#[command(about = "CC Switch CLI - Headless proxy and provider management")]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Start the proxy server
    Start {
        /// Listen address (default: 127.0.0.1)
        #[arg(short, long, default_value = "127.0.0.1")]
        address: String,
        
        /// Listen port (default: 15721)
        #[arg(short, long, default_value = "15721")]
        port: u16,
    },
    
    /// Stop the proxy server
    Stop,
    
    /// List all providers
    ListProviders {
        /// App type filter (claude/codex/gemini/claude-desktop/opencode/openclaw/hermes)
        #[arg(short, long)]
        app: Option<String>,
    },
    
    /// Add a new provider
    AddProvider {
        /// App type
        #[arg(short, long)]
        app: String,
        
        /// Provider name
        #[arg(short, long)]
        name: String,
        
        /// Base URL
        #[arg(short, long)]
        url: String,
        
        /// API key
        #[arg(short = 'k', long)]
        api_key: String,
        
        /// API format (anthropic/openai_chat/openai_responses/gemini_native/bedrock)
        #[arg(short, long)]
        format: Option<String>,
    },
    
    /// Remove a provider
    RemoveProvider {
        /// Provider ID
        #[arg(short, long)]
        id: String,
    },
    
    /// Switch current provider
    SwitchProvider {
        /// App type
        #[arg(short, long)]
        app: String,
        
        /// Provider ID
        #[arg(short, long)]
        id: String,
    },
    
    /// Show proxy status
    Status,
    
    /// Show configuration file paths
    Config,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();
    
    match cli.command {
        Commands::Start { address, port } => {
            println!("Starting proxy server on {}:{}", address, port);
            // TODO: 调用代理服务器启动逻辑
            start_proxy_server(&address, port).await?;
        }
        
        Commands::Stop => {
            println!("Stopping proxy server");
            // TODO: 停止代理服务器
        }
        
        Commands::ListProviders { app } => {
            list_providers(app.as_deref()).await?;
        }
        
        Commands::AddProvider { app, name, url, api_key, format } => {
            add_provider(&app, &name, &url, &api_key, format.as_deref()).await?;
        }
        
        Commands::RemoveProvider { id } => {
            remove_provider(&id).await?;
        }
        
        Commands::SwitchProvider { app, id } => {
            switch_provider(&app, &id).await?;
        }
        
        Commands::Status => {
            show_status().await?;
        }
        
        Commands::Config => {
            show_config_paths();
        }
    }
    
    Ok(())
}

async fn start_proxy_server(address: &str, port: u16) -> Result<(), Box<dyn std::error::Error>> {
    // 复用 cc_switch_lib::proxy::server 的逻辑
    // 需要提取出不依赖 Tauri 的部分
    println!("Proxy server started. Press Ctrl+C to stop.");
    
    // 这里需要调用实际的代理服务器启动代码
    // 暂时用占位符
    tokio::signal::ctrl_c().await?;
    println!("\nShutting down...");
    
    Ok(())
}

async fn list_providers(app: Option<&str>) -> Result<(), Box<dyn std::error::Error>> {
    // 从数据库读取供应商列表
    println!("Providers:");
    // TODO: 实现
    Ok(())
}

async fn add_provider(
    app: &str,
    name: &str,
    url: &str,
    api_key: &str,
    format: Option<&str>,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("Adding provider '{}' for app '{}'", name, app);
    // TODO: 实现
    Ok(())
}

async fn remove_provider(id: &str) -> Result<(), Box<dyn std::error::Error>> {
    println!("Removing provider '{}'", id);
    // TODO: 实现
    Ok(())
}

async fn switch_provider(app: &str, id: &str) -> Result<(), Box<dyn std::error::Error>> {
    println!("Switching {} to provider '{}'", app, id);
    // TODO: 实现
    Ok(())
}

async fn show_status() -> Result<(), Box<dyn std::error::Error>> {
    println!("Proxy Status:");
    // TODO: 实现
    Ok(())
}

fn show_config_paths() {
    let home = std::env::var("HOME").unwrap_or_else(|_| "/root".to_string());
    let config_dir = PathBuf::from(&home).join(".cc-switch");
    
    println!("Configuration Directory: {}", config_dir.display());
    println!("  Database: {}/cc-switch.db", config_dir.display());
    println!("  Settings: {}/settings.json", config_dir.display());
}
```

- [ ] **Step 3: 测试编译**

```bash
cd src-tauri
cargo build --bin cc-switch-cli
```

- [ ] **Step 4: 提交**

```bash
git add src-tauri/Cargo.toml src-tauri/src/bin/cc-switch-cli.rs
git commit -m "feat(cli): add CLI binary entry point with command structure"
```

---

### Task 3.2: 提取核心逻辑模块

**Files:**
- Create: `src-tauri/src/core/mod.rs`
- Create: `src-tauri/src/core/database.rs`
- Create: `src-tauri/src/core/provider_manager.rs`

- [ ] **Step 1: 创建 core 模块**

将不依赖 Tauri 的核心逻辑提取到 `core` 模块：

```rust
// src-tauri/src/core/mod.rs

pub mod database;
pub mod provider_manager;
```

- [ ] **Step 2: 提取数据库初始化逻辑**

从 `src-tauri/src/database/mod.rs` 中提取出不依赖 Tauri 的数据库初始化代码：

```rust
// src-tauri/src/core/database.rs

use rusqlite::Connection;
use std::path::PathBuf;

pub fn get_config_dir() -> PathBuf {
    let home = std::env::var("HOME").unwrap_or_else(|_| "/root".to_string());
    PathBuf::from(&home).join(".cc-switch")
}

pub fn init_database() -> Result<Connection, Box<dyn std::error::Error>> {
    let config_dir = get_config_dir();
    std::fs::create_dir_all(&config_dir)?;
    
    let db_path = config_dir.join("cc-switch.db");
    let conn = Connection::open(&db_path)?;
    
    // 执行数据库迁移
    // ... 复用现有的迁移逻辑
    
    Ok(conn)
}
```

- [ ] **Step 3: 提取供应商管理逻辑**

```rust
// src-tauri/src/core/provider_manager.rs

use rusqlite::Connection;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Provider {
    pub id: String,
    pub name: String,
    pub app_type: String,
    pub base_url: String,
    pub api_key: String,
    pub api_format: Option<String>,
    // ... 其他字段
}

pub struct ProviderManager {
    conn: Connection,
}

impl ProviderManager {
    pub fn new(conn: Connection) -> Self {
        Self { conn }
    }
    
    pub fn list_providers(&self, app_type: Option<&str>) -> Result<Vec<Provider>, Box<dyn std::error::Error>> {
        // 从数据库查询供应商
        let mut sql = "SELECT id, name, app_type, base_url, api_key, api_format FROM providers".to_string();
        
        if let Some(app) = app_type {
            sql.push_str(&format!(" WHERE app_type = '{}'", app));
        }
        
        let mut stmt = self.conn.prepare(&sql)?;
        let providers = stmt.query_map([], |row| {
            Ok(Provider {
                id: row.get(0)?,
                name: row.get(1)?,
                app_type: row.get(2)?,
                base_url: row.get(3)?,
                api_key: row.get(4)?,
                api_format: row.get(5)?,
            })
        })?;
        
        Ok(providers.filter_map(|p| p.ok()).collect())
    }
    
    pub fn add_provider(&mut self, provider: &Provider) -> Result<(), Box<dyn std::error::Error>> {
        self.conn.execute(
            "INSERT INTO providers (id, name, app_type, base_url, api_key, api_format) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            rusqlite::params![
                provider.id,
                provider.name,
                provider.app_type,
                provider.base_url,
                provider.api_key,
                provider.api_format,
            ],
        )?;
        
        Ok(())
    }
    
    // ... 其他方法
}
```

- [ ] **Step 4: 提交**

```bash
git add src-tauri/src/core/
git commit -m "refactor: extract core logic modules for CLI reuse"
```

---

### Task 3.3: 实现 CLI 命令逻辑

**Files:**
- Modify: `src-tauri/src/bin/cc-switch-cli.rs`

- [ ] **Step 1: 实现 list_providers 命令**

```rust
async fn list_providers(app: Option<&str>) -> Result<(), Box<dyn std::error::Error>> {
    let conn = cc_switch_lib::core::database::init_database()?;
    let manager = cc_switch_lib::core::provider_manager::ProviderManager::new(conn);
    
    let providers = manager.list_providers(app)?;
    
    if providers.is_empty() {
        println!("No providers found.");
        return Ok(());
    }
    
    println!("Providers:\n");
    println!("{:<36} {:<15} {:<20} {:<30}", "ID", "App", "Name", "Base URL");
    println!("{}", "-".repeat(101));
    
    for p in providers {
        println!(
            "{:<36} {:<15} {:<20} {:<30}",
            p.id,
            p.app_type,
            p.name,
            p.base_url
        );
    }
    
    Ok(())
}
```

- [ ] **Step 2: 实现 add_provider 命令**

```rust
async fn add_provider(
    app: &str,
    name: &str,
    url: &str,
    api_key: &str,
    format: Option<&str>,
) -> Result<(), Box<dyn std::error::Error>> {
    let conn = cc_switch_lib::core::database::init_database()?;
    let mut manager = cc_switch_lib::core::provider_manager::ProviderManager::new(conn);
    
    let provider = cc_switch_lib::core::provider_manager::Provider {
        id: uuid::Uuid::new_v4().to_string(),
        name: name.to_string(),
        app_type: app.to_string(),
        base_url: url.to_string(),
        api_key: api_key.to_string(),
        api_format: format.map(|s| s.to_string()),
    };
    
    manager.add_provider(&provider)?;
    
    println!("Provider added successfully!");
    println!("  ID: {}", provider.id);
    println!("  Name: {}", provider.name);
    println!("  App: {}", provider.app_type);
    
    Ok(())
}
```

- [ ] **Step 3: 实现其他命令**

类似地实现 `remove_provider`, `switch_provider`, `show_status` 等命令。

- [ ] **Step 4: 测试 CLI**

```bash
# 构建
cargo build --bin cc-switch-cli

# 测试帮助
./target/debug/cc-switch-cli --help

# 测试列出供应商
./target/debug/cc-switch-cli list-providers

# 测试添加供应商
./target/debug/cc-switch-cli add-provider \
  --app claude \
  --name "Test Provider" \
  --url "https://api.example.com" \
  --api-key "sk-test" \
  --format anthropic

# 测试启动代理
./target/debug/cc-switch-cli start --port 8080
```

- [ ] **Step 5: 提交**

```bash
git add src-tauri/src/bin/cc-switch-cli.rs
git commit -m "feat(cli): implement CLI command logic"
```

---

### Task 3.4: 集成代理服务器启动

**Files:**
- Modify: `src-tauri/src/bin/cc-switch-cli.rs`
- Modify: `src-tauri/src/proxy/server.rs`

- [ ] **Step 1: 在 server.rs 中提取不依赖 Tauri 的启动函数**

```rust
// src-tauri/src/proxy/server.rs

pub async fn start_proxy_standalone(
    address: &str,
    port: u16,
    config: ProxyConfig,
) -> Result<(), Box<dyn std::error::Error>> {
    // 复用现有的代理服务器启动逻辑
    // 但不依赖 Tauri 的上下文
    
    let addr = format!("{}:{}", address, port);
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    
    println!("Proxy server listening on {}", addr);
    
    // 构建路由
    // ... 复用现有逻辑
    
    // 启动服务器循环
    loop {
        let (socket, _) = listener.accept().await?;
        tokio::spawn(handle_connection(socket));
    }
}
```

- [ ] **Step 2: 在 CLI 中调用代理启动**

```rust
async fn start_proxy_server(address: &str, port: u16) -> Result<(), Box<dyn std::error::Error>> {
    let config = cc_switch_lib::proxy::types::ProxyConfig {
        listen_address: address.to_string(),
        listen_port: port,
        ..Default::default()
    };
    
    cc_switch_lib::proxy::server::start_proxy_standalone(address, port, config).await?;
    
    Ok(())
}
```

- [ ] **Step 3: 测试代理启动**

```bash
./target/debug/cc-switch-cli start --port 8080
```

然后在另一个终端测试：

```bash
curl http://localhost:8080/v1/messages \
  -H "Content-Type: application/json" \
  -d '{"model":"claude-3-opus","messages":[{"role":"user","content":"Hello"}]}'
```

- [ ] **Step 4: 提交**

```bash
git add src-tauri/src/proxy/server.rs src-tauri/src/bin/cc-switch-cli.rs
git commit -m "feat(cli): integrate proxy server startup"
```

---

## Phase 4: 文档编写

### Task 4.1: CLI 使用文档

**Files:**
- Create: `docs/cli-usage.md`

- [ ] **Step 1: 创建 CLI 使用文档**

```markdown
# CC Switch CLI 使用指南

CC Switch CLI 提供无头服务器模式，适用于 Linux 服务器和自动化场景。

## 安装

```bash
# 从源码编译
cd src-tauri
cargo build --release --bin cc-switch-cli

# 复制到 PATH
sudo cp target/release/cc-switch-cli /usr/local/bin/
```

## 基本命令

### 启动代理服务器

```bash
cc-switch-cli start --address 127.0.0.1 --port 15721
```

### 停止代理服务器

```bash
cc-switch-cli stop
```

### 列出供应商

```bash
# 列出所有供应商
cc-switch-cli list-providers

# 只列出 Claude 供应商
cc-switch-cli list-providers --app claude
```

### 添加供应商

```bash
cc-switch-cli add-provider \
  --app claude \
  --name "My Provider" \
  --url "https://api.example.com" \
  --api-key "sk-xxx" \
  --format anthropic
```

支持的 `--format` 值：
- `anthropic`: Anthropic Messages API（默认）
- `openai_chat`: OpenAI Chat Completions
- `openai_responses`: OpenAI Responses API
- `gemini_native`: Gemini Native API
- `bedrock`: Amazon Bedrock Converse API

### 删除供应商

```bash
cc-switch-cli remove-provider --id <provider-id>
```

### 切换当前供应商

```bash
cc-switch-cli switch-provider --app claude --id <provider-id>
```

### 查看状态

```bash
cc-switch-cli status
```

### 查看配置路径

```bash
cc-switch-cli config
```

## systemd 服务配置

创建 `/etc/systemd/system/cc-switch.service`：

```ini
[Unit]
Description=CC Switch Proxy Server
After=network.target

[Service]
Type=simple
User=your-user
ExecStart=/usr/local/bin/cc-switch-cli start --address 0.0.0.0 --port 15721
Restart=on-failure
RestartSec=10

[Install]
WantedBy=multi-user.target
```

启用并启动服务：

```bash
sudo systemctl daemon-reload
sudo systemctl enable cc-switch
sudo systemctl start cc-switch
sudo systemctl status cc-switch
```

## 配置 Claude Code 使用代理

在 Claude Code 的配置文件中设置代理：

```bash
# ~/.claude/settings.json
{
  "apiBaseUrl": "http://your-server:15721"
}
```

## 环境变量

- `CC_SWITCH_HOME`: 覆盖配置目录（默认 `~/.cc-switch`）
- `CC_SWITCH_LOG_LEVEL`: 日志级别（error/warn/info/debug/trace）

## 故障排查

### 查看日志

```bash
journalctl -u cc-switch -f
```

### 测试连接

```bash
curl http://localhost:15721/health
```

### 检查数据库

```bash
sqlite3 ~/.cc-switch/cc-switch.db
> SELECT * FROM providers;
```
```

- [ ] **Step 2: 提交**

```bash
git add docs/cli-usage.md
git commit -m "docs: add CLI usage guide"
```

---

### Task 4.2: API 格式转换矩阵文档

**Files:**
- Create: `docs/api-format-matrix.md`

- [ ] **Step 1: 创建 API 格式转换矩阵文档**

```markdown
# API 格式转换矩阵

CC Switch 支持多种 API 格式之间的自动转换，允许任意客户端调用任意模型后端。

## 支持的 API 格式

| 格式 | 说明 | 典型使用场景 |
|------|------|--------------|
| `anthropic` | Anthropic Messages API | Claude Code, Claude Desktop |
| `openai_chat` | OpenAI Chat Completions API | 大多数第三方供应商 |
| `openai_responses` | OpenAI Responses API | Codex, ChatGPT Plus/Pro |
| `gemini_native` | Gemini Native generateContent API | Gemini CLI |
| `bedrock` | Amazon Bedrock Converse API | AWS Bedrock |

## 客户端 → 后端转换矩阵

### Claude Code / Claude Desktop 客户端

| 后端格式 | 支持状态 | 说明 |
|----------|----------|------|
| `anthropic` | ✅ 直接透传 | 原生格式，无需转换 |
| `openai_chat` | ✅ 支持 | 自动转换请求和响应 |
| `openai_responses` | ✅ 支持 | 自动转换请求和响应 |
| `gemini_native` | ✅ 支持 | 自动转换请求和响应 |
| `bedrock` | ✅ 支持 | 自动转换请求和响应 |

### Codex 客户端

| 后端格式 | 支持状态 | 说明 |
|----------|----------|------|
| `openai_responses` | ✅ 直接透传 | 原生格式，无需转换 |
| `openai_chat` | ✅ 支持 | Responses ↔ Chat 自动转换 |
| `anthropic` | ⚠️ 间接支持 | 通过 Chat → Anthropic 转换 |
| `gemini_native` | ❌ 不支持 | 需要多级转换，暂未实现 |
| `bedrock` | ❌ 不支持 | 需要多级转换，暂未实现 |

### Gemini CLI 客户端

| 后端格式 | 支持状态 | 说明 |
|----------|----------|------|
| `gemini_native` | ✅ 直接透传 | 原生格式，无需转换 |
| `anthropic` | ⚠️ 理论支持 | 很少使用此场景 |
| `openai_chat` | ⚠️ 理论支持 | 很少使用此场景 |
| `openai_responses` | ⚠️ 理论支持 | 很少使用此场景 |
| `bedrock` | ❌ 不支持 | 暂未实现 |

## 配置示例

### Claude Code 使用 OpenAI 供应商

1. 在 CC Switch 中添加供应商：
   - App: `claude`
   - Name: `OpenAI Provider`
   - Base URL: `https://api.openai.com`
   - API Key: `sk-xxx`
   - API Format: `openai_chat`

2. CC Switch 会自动将 Claude 格式的请求转换为 OpenAI Chat 格式

### Claude Code 使用 AWS Bedrock

1. 添加供应商：
   - App: `claude`
   - Name: `Bedrock Provider`
   - Base URL: `https://bedrock-runtime.us-east-1.amazonaws.com`
   - API Key: `your-aws-access-key`
   - API Format: `bedrock`

2. CC Switch 会自动转换请求和响应格式

### Codex 使用第三方 Chat Completions 供应商

1. 添加供应商：
   - App: `codex`
   - Name: `Custom Provider`
   - Base URL: `https://api.example.com`
   - API Key: `sk-xxx`
   - API Format: `openai_chat`

2. CC Switch 会自动将 Codex Responses 格式转换为 Chat Completions 格式

## 转换流程

```
客户端请求 (格式 A)
    ↓
CC Switch 代理
    ↓
检测客户端格式 (ClientFormat)
    ↓
读取供应商配置的目标格式 (apiFormat)
    ↓
如果格式不同，调用转换函数
    ↓
发送请求到上游 (格式 B)
    ↓
接收响应 (格式 B)
    ↓
转换回客户端格式 (格式 A)
    ↓
返回给客户端
```

## 流式响应转换

所有转换都支持流式响应（SSE）：

- `anthropic_to_openai` + `create_anthropic_sse_stream_from_openai`
- `anthropic_to_responses` + `create_anthropic_sse_stream_from_responses`
- `anthropic_to_gemini` + `create_anthropic_sse_stream_from_gemini`
- `anthropic_to_bedrock` + `create_anthropic_sse_stream_from_bedrock`（待实现）

## 限制和注意事项

1. **工具调用（Tool Use）**: 不同格式的工具调用结构差异较大，转换可能不完全支持所有特性
2. **Thinking/Extended Thinking**: Anthropic 的 thinking 特性在其他格式中可能没有对应实现
3. **多模态输入**: 图片等多模态输入的转换支持程度取决于目标格式
4. **缓存控制**: Anthropic 的 cache_control 在其他格式中可能不支持

## 未来计划

- [ ] 实现 Bedrock 流式响应转换
- [ ] 支持 Codex → Gemini 多级转换
- [ ] 支持 Codex → Bedrock 多级转换
- [ ] 完善工具调用的双向转换
- [ ] 添加转换质量测试套件
```

- [ ] **Step 2: 提交**

```bash
git add docs/api-format-matrix.md
git commit -m "docs: add API format conversion matrix"
```

---

## Phase 5: 测试和验证

### Task 5.1: 集成测试

- [ ] **Step 1: 测试 Gemini API 格式切换**

```bash
# 启动代理
./target/debug/cc-switch-cli start --port 8080

# 添加一个 OpenAI 供应商给 Gemini 使用
./target/debug/cc-switch-cli add-provider \
  --app gemini \
  --name "OpenAI for Gemini" \
  --url "https://api.openai.com" \
  --api-key "sk-xxx" \
  --format openai_chat

# 切换到此供应商
./target/debug/cc-switch-cli switch-provider --app gemini --id <provider-id>

# 使用 Gemini CLI 测试
gemini-cli "Hello"
```

- [ ] **Step 2: 测试 Claude Desktop API 格式切换**

类似步骤，测试 Claude Desktop 使用不同后端格式。

- [ ] **Step 3: 测试 CLI 代理服务器**

```bash
# 启动 CLI 代理
./target/debug/cc-switch-cli start --port 9090

# 在另一个终端测试
curl http://localhost:9090/v1/messages \
  -H "Content-Type: application/json" \
  -H "x-api-key: sk-xxx" \
  -d '{"model":"claude-3-opus","messages":[{"role":"user","content":"Hello"}]}'
```

- [ ] **Step 4: 提交测试结果**

```bash
git add tests/
git commit -m "test: add integration tests for new features"
```

---

## 实施顺序建议

1. **Phase 1** (Gemini API 格式): 最简单，风险最低
2. **Phase 2** (Claude Desktop API 格式): 类似 Phase 1
3. **Phase 3** (CLI 模式): 工作量最大，但相对独立
4. **Phase 4** (文档): 在功能完成后编写
5. **Phase 5** (测试): 贯穿整个过程

## 风险和注意事项

1. **数据库迁移**: 添加新字段时需要考虑旧数据库的兼容性
2. **前端状态管理**: 确保 API 格式选择正确保存到 provider.meta
3. **CLI 与 GUI 共享配置**: 确保两者读写同一个数据库文件
4. **Bedrock 流式转换**: 当前 transform_bedrock.rs 可能缺少流式支持，需要验证
5. **多级转换**: 某些转换路径需要多级转换（如 Codex → Gemini），当前未实现

## 验收标准

- [ ] Gemini 应用可以选择并使用非 Gemini Native 后端
- [ ] Claude Desktop 应用可以选择并使用非 Anthropic 后端
- [ ] CLI 可以独立启动代理服务器
- [ ] CLI 可以管理供应商（增删改查）
- [ ] CLI 和 GUI 共享同一份配置
- [ ] 完整的文档覆盖所有新功能
- [ ] 所有转换路径都有测试用例
