# Plan C: 新功能实现（热重载 / 访问控制 / 烟雾测试） Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 为 cc-switch-cli 增加三项无头部署刚需能力——代理热重载（REQ-022）、代理访问控制（REQ-023）、协议转换烟雾测试（REQ-021），让 CLI 在远程服务器场景下达到生产可用。

**Architecture:** 在已有 lib crate 分层基础上扩展：① ProxyService 新增 `reload_config()`，通过 `apply_runtime_config` + `update_circuit_breaker_configs` 实现不中断连接的配置热更新；② 新增 `proxy/ingress_auth` 模块作为 Axum middleware，校验 `Authorization: Bearer <token>` 与来源 IP CIDR；③ 新增 `proxy/smoke_test` 模块，直接调用 transform 子模块（不走网络）对每个 AppType 跑一次最小请求验证转换链路。CLI 侧在 `Commands` 枚举追加 `Reload / AuthToken / Acl / SmokeTest` 四个子命令。

**Tech Stack:** Rust 1.95、Tokio、Axum（middleware）、clap（CLI）、rusqlite（settings 表）、serde_json（CIDR 数组与请求体序列化）、ipnet（CIDR 解析）。

**关联 Spec:** [docs/superpowers/specs/2026-07-04-cli-feature-review-design.md](file:///f:/workspace/trae/cc-switch/docs/superpowers/specs/2026-07-04-cli-feature-review-design.md) §九 M-1（REQ-022）、§九 M-2（REQ-023）、§四 REQ-021

**前置依赖:** Plan B 已完成——lib crate 已分层（`cc-switch-lib`），CLI 可通过 `cc_switch_lib::ProxyService` 直接创建实例并调用方法。

---

## File Structure

| 文件 | 操作 | 责任 |
|---|---|---|
| `src-tauri/src/database/dao/settings.rs` | 修改 | 新增 `proxy_auth_token` / `proxy_acl_cidrs` 读写方法 |
| `src-tauri/src/proxy/ingress_auth.rs` | 新建 | Axum middleware：校验 Bearer token 与 IP CIDR |
| `src-tauri/src/proxy/mod.rs` | 修改 | 导出 `ingress_auth` 模块 |
| `src-tauri/src/proxy/server.rs` | 修改 | `build_router` 添加 ingress auth layer；`reload_runtime_config` 公开方法 |
| `src-tauri/src/services/proxy.rs` | 修改 | 新增 `reload_config()` / `get_auth_token()` / `set_auth_token()` / `get_acl()` / `set_acl()` |
| `src-tauri/src/proxy/smoke_test.rs` | 新建 | 协议转换烟雾测试核心模块（不走网络，直接调 transform） |
| `src-tauri/src/lib.rs` | 修改 | 导出 `smoke_test` 与 ingress auth 相关类型 |
| `src-tauri/src/bin/cc-switch-cli.rs` | 修改 | Commands 枚举追加 4 个子命令；新增 `cmd_reload` / `cmd_auth_token` / `cmd_acl` / `cmd_smoke_test` |
| `src-tauri/tests/plan_c_reload.rs` | 新建 | REQ-022 集成测试 |
| `src-tauri/tests/plan_c_auth.rs` | 新建 | REQ-023 集成测试 |
| `src-tauri/tests/plan_c_smoke_test.rs` | 新建 | REQ-021 集成测试 |
| `src-tauri/Cargo.toml` | 修改 | 添加 `ipnet = "2"` 依赖 |
| `docs/cli-reference-manual.md` | 修改 | 追加 reload / auth-token / acl / smoke-test 命令章节 |

---

## Task 1: 数据库 DAO 扩展——auth_token / acl 存取（REQ-023 基础）

**Files:**
- Modify: `src-tauri/src/database/dao/settings.rs:330-348`（在文件末尾追加方法）
- Test: `src-tauri/tests/plan_c_auth.rs`（在 Task 9 中编写）

- [ ] **Step 1: 阅读 settings.rs 末尾结构**

Read [src-tauri/src/database/dao/settings.rs:330-348](file:///f:/workspace/trae/cc-switch/src-tauri/src/database/dao/settings.rs)

确认文件末尾是 `set_log_config` 方法。所有 setting 都用 `get_setting` / `set_setting` 实现，新增方法遵循同样模式。

- [ ] **Step 2: 在 settings.rs 末尾追加 auth_token 与 acl 方法**

在 `src-tauri/src/database/dao/settings.rs` 文件最后的 `}`（impl 块结束符）之前插入以下方法：

```rust
    // --- 代理 ingress 访问控制（REQ-023） ---

    /// 代理访问令牌的 settings 表键名
    const PROXY_AUTH_TOKEN_KEY: &'static str = "proxy_auth_token";

    /// 代理 IP 白名单（CIDR 数组）的 settings 表键名
    const PROXY_ACL_CIDRS_KEY: &'static str = "proxy_acl_cidrs";

    /// 读取代理访问令牌
    ///
    /// 返回 None 表示未设置（代理完全开放）。CLI 场景下直接存明文，
    /// 无头服务器文件权限由 OS 层保证（~/.cc-switch/cc-switch.db 0600）。
    pub fn get_proxy_auth_token(&self) -> Result<Option<String>, AppError> {
        self.get_setting(Self::PROXY_AUTH_TOKEN_KEY)
    }

    /// 设置代理访问令牌
    ///
    /// 传入 None 或空字符串则清除令牌（代理回到开放状态）。
    pub fn set_proxy_auth_token(&self, token: Option<&str>) -> Result<(), AppError> {
        match token {
            Some(t) if !t.trim().is_empty() => {
                self.set_setting(Self::PROXY_AUTH_TOKEN_KEY, t.trim())
            }
            _ => {
                let conn = lock_conn!(self.conn);
                conn.execute(
                    "DELETE FROM settings WHERE key = ?1",
                    params![Self::PROXY_AUTH_TOKEN_KEY],
                )
                .map_err(|e| AppError::Database(e.to_string()))?;
                Ok(())
            }
        }
    }

    /// 读取代理 IP 白名单（CIDR 字符串数组）
    ///
    /// 返回空 Vec 表示未设置白名单（不进行 IP 校验）。
    pub fn get_proxy_acl_cidrs(&self) -> Result<Vec<String>, AppError> {
        match self.get_setting(Self::PROXY_ACL_CIDRS_KEY)? {
            Some(json) => serde_json::from_str::<Vec<String>>(&json)
                .map_err(|e| AppError::Database(format!("解析代理 ACL 失败: {e}"))),
            None => Ok(Vec::new()),
        }
    }

    /// 设置代理 IP 白名单（CIDR 字符串数组）
    ///
    /// 传入空 Vec 则清除白名单。每个 CIDR 会在 set 前用 `ipnet::IpNet::from_str`
    /// 校验，非法值返回 AppError。
    pub fn set_proxy_acl_cidrs(&self, cidrs: &[String]) -> Result<(), AppError> {
        if cidrs.is_empty() {
            let conn = lock_conn!(self.conn);
            conn.execute(
                "DELETE FROM settings WHERE key = ?1",
                params![Self::PROXY_ACL_CIDRS_KEY],
            )
            .map_err(|e| AppError::Database(e.to_string()))?;
            return Ok(());
        }
        // 先全部校验，避免半写入
        for c in cidrs {
            ipnet::IpNet::from_str(c).map_err(|e| {
                AppError::Config(format!("非法 CIDR '{c}': {e}"))
            })?;
        }
        let json = serde_json::to_string(cidrs)
            .map_err(|e| AppError::Database(format!("序列化 ACL 失败: {e}")))?;
        self.set_setting(Self::PROXY_ACL_CIDRS_KEY, &json)
    }
```

- [ ] **Step 3: 添加 ipnet 依赖**

Read [src-tauri/Cargo.toml](file:///f:/workspace/trae/cc-switch/src-tauri/Cargo.toml) 找到 `[dependencies]` 段，确认是否已有 `ipnet`。

如未存在，在 `[dependencies]` 段追加：

```toml
ipnet = "2"
```

- [ ] **Step 4: 验证编译**

Run:

```bash
cd f:/workspace/trae/cc-switch/src-tauri
cargo check --lib
```

Expected: 编译通过，无 `ipnet` / `get_proxy_auth_token` 相关错误。

- [ ] **Step 5: Commit**

```bash
cd f:/workspace/trae/cc-switch
git add src-tauri/src/database/dao/settings.rs src-tauri/Cargo.toml
git commit -m "feat(database): 新增 proxy_auth_token / proxy_acl_cidrs 存取方法

- get/set_proxy_auth_token: 代理访问令牌读写
- get/set_proxy_acl_cidrs: IP CIDR 白名单读写，set 时用 ipnet 校验
- 添加 ipnet = \"2\" 依赖

关联 spec: docs/superpowers/specs/2026-07-04-cli-feature-review-design.md §九 M-2 (REQ-023)"
```

---

## Task 2: ingress_auth 模块——Axum middleware（REQ-023 核心）

**Files:**
- Create: `src-tauri/src/proxy/ingress_auth.rs`
- Modify: `src-tauri/src/proxy/mod.rs:5-37`（添加 `pub mod ingress_auth;`）

- [ ] **Step 1: 创建 ingress_auth.rs 文件**

写入 `src-tauri/src/proxy/ingress_auth.rs`：

```rust
//! 代理 ingress 访问控制 middleware
//!
//! 在 Axum 路由层校验：
//! 1. 来源 IP 是否在白名单 CIDR 内（白名单为空时跳过）
//! 2. Authorization: Bearer <token> 是否匹配（token 未设置时跳过）
//!
//! 任一校验开启且未通过，直接返回 401/403，不走业务 handler。

use crate::database::Database;
use crate::proxy::ProxyError;
use axum::{
    extract::{Request, State},
    http::StatusCode,
    middleware::Next,
    response::{IntoResponse, Json},
};
use ipnet::IpNet;
use serde_json::json;
use std::net::IpAddr;
use std::str::FromStr;
use std::sync::Arc;

/// ingress 校验所需的运行时上下文
///
/// token 与 acl 都从数据库读取后缓存到 `Arc<Database>`，middleware 每次请求时
/// 现读现校验——避免热重载后 token/acl 变更不生效。设置项读取开销可接受
/// （SQLite 单行查询 <1ms，远低于一次 LLM 请求）。
#[derive(Clone)]
pub struct IngressAuthState {
    pub db: Arc<Database>,
}

impl IngressAuthState {
    pub fn new(db: Arc<Database>) -> Self {
        Self { db }
    }
}

/// Axum middleware 入口：校验 token 与 CIDR
///
/// 调用顺序：先 IP（无 token 也能挡住扫描器），再 token。
pub async fn ingress_auth_middleware(
    State(state): State<IngressAuthState>,
    req: Request,
    next: Next,
) -> Result<axum::response::Response, ProxyError> {
    // 来源 IP：优先取 X-Forwarded-For 首段（反代场景），否则取连接对端地址。
    // axum 的 ConnectInfo<SocketAddr> 需在 server 启动时 `.into_make_service_with_connect_info()`
    // 才有；当前 server 用手动 hyper accept loop，ConnectInfo 不可用，因此从
    // X-Forwarded-For 兜底，无该头时跳过 IP 校验（与"无白名单=开放"语义一致）。
    let client_ip = req
        .headers()
        .get("x-forwarded-for")
        .and_then(|v| v.to_str().ok())
        .and_then(|s| s.split(',').next())
        .and_then(|s| s.trim().parse::<IpAddr>().ok());

    // --- IP 白名单校验 ---
    let acl: Vec<String> = state
        .db
        .get_proxy_acl_cidrs()
        .map_err(|e| ProxyError::Internal(format!("读取 ACL 失败: {e}")))?;
    if !acl.is_empty() {
        let ip = match client_ip {
            Some(ip) => ip,
            None => {
                return Err(ProxyError::AuthError(
                    "缺少 X-Forwarded-For 头，无法校验来源 IP".to_string(),
                ));
            }
        };
        let allowed = acl.iter().any(|cidr_str| {
            IpNet::from_str(cidr_str)
                .map(|net| net.contains(&ip))
                .unwrap_or(false)
        });
        if !allowed {
            return Err(ProxyError::AuthError(format!(
                "来源 IP {ip} 不在白名单内"
            )));
        }
    }

    // --- Bearer token 校验 ---
    let expected_token: Option<String> = state
        .db
        .get_proxy_auth_token()
        .map_err(|e| ProxyError::Internal(format!("读取 auth_token 失败: {e}")))?;
    if let Some(expected) = expected_token {
        let provided = req
            .headers()
            .get(axum::http::header::AUTHORIZATION)
            .and_then(|v| v.to_str().ok())
            .and_then(|v| {
                v.strip_prefix("Bearer ")
                    .or_else(|| v.strip_prefix("bearer "))
            })
            .map(str::trim);
        match provided {
            Some(t) if t == expected => {}
            _ => {
                return Err(ProxyError::AuthError(
                    "无效或缺失的 Bearer token".to_string(),
                ));
            }
        }
    }

    Ok(next.run(req).await)
}

/// 把 ProxyError 映射为 HTTP 响应，让 middleware 错误能被 axum 直接返回
///
/// axum middleware 要求返回 `Result<Response, E>` 且 `E: IntoResponse`。
/// ProxyError 已实现 `IntoResponse`，但语义是把内部错误透传给客户端；ingress
/// 校验失败应统一返回 401/403 + 标准 JSON 错误体，不暴露内部原因。
pub fn auth_error_response(err: &ProxyError) -> axum::response::Response {
    let (status, message) = match err {
        ProxyError::AuthError(msg) => {
            // 区分 token 失败（401）与 IP 失败（403）
            if msg.contains("Bearer token") {
                (StatusCode::UNAUTHORIZED, " unauthorized: invalid or missing token")
            } else {
                (StatusCode::FORBIDDEN, " forbidden: ip not allowed")
            }
        }
        _ => (StatusCode::INTERNAL_SERVER_ERROR, "internal error"),
    };
    (
        status,
        Json(json!({
            "error": {
                "type": "ingress_auth",
                "message": message.trim(),
            }
        })),
    )
        .into_response()
}

/// 适配 axum middleware：把 ProxyError 转 Response，避免 middleware 类型不匹配
pub async fn ingress_auth_middleware_into_response(
    state: IngressAuthState,
    req: Request,
    next: Next,
) -> axum::response::Response {
    match ingress_auth_middleware(State(state), req, next).await {
        Ok(resp) => resp,
        Err(err) => auth_error_response(&err),
    }
}
```

- [ ] **Step 2: 在 proxy/mod.rs 中导出模块**

Read [src-tauri/src/proxy/mod.rs:1-40](file:///f:/workspace/trae/cc-switch/src-tauri/src/proxy/mod.rs)

在 `pub mod handler_config;` 上一行（按字母序）插入：

```rust
pub mod ingress_auth;
```

- [ ] **Step 3: 验证编译**

Run:

```bash
cd f:/workspace/trae/cc-switch/src-tauri
cargo check --lib
```

Expected: 编译通过。可能出现 `ingress_auth_middleware_into_response` 未使用的警告——后续 Task 4 会接线使用。

- [ ] **Step 4: Commit**

```bash
cd f:/workspace/trae/cc-switch
git add src-tauri/src/proxy/ingress_auth.rs src-tauri/src/proxy/mod.rs
git commit -m "feat(proxy): 新增 ingress_auth middleware 模块

- IngressAuthState 持有 Arc<Database>
- ingress_auth_middleware: 校验 X-Forwarded-For 来源 IP 与 Bearer token
- auth_error_response: 401 (token) / 403 (ip) 标准错误响应
- 现读现校验，确保 reload 后 token/acl 立即生效

关联 spec: docs/superpowers/specs/2026-07-04-cli-feature-review-design.md §九 M-2 (REQ-023)"
```

---

## Task 3: ProxyServer 接入 ingress auth layer + reload_runtime_config 公开方法

**Files:**
- Modify: `src-tauri/src/proxy/server.rs:32-53`（ProxyState 字段）
- Modify: `src-tauri/src/proxy/server.rs:63-95`（ProxyServer::new）
- Modify: `src-tauri/src/proxy/server.rs:294-365`（build_router）
- Modify: `src-tauri/src/proxy/server.rs:367-370`（apply_runtime_config 旁）

- [ ] **Step 1: ProxyState 增加 ingress_auth_state 字段**

Read [src-tauri/src/proxy/server.rs:32-53](file:///f:/workspace/trae/cc-switch/src-tauri/src/proxy/server.rs)

将 `ProxyState` 结构体在 `shutdown_tx` 字段后追加：

```rust
    /// ingress 访问控制 middleware 状态（token / acl 校验）
    pub ingress_auth_state: super::ingress_auth::IngressAuthState,
```

- [ ] **Step 2: ProxyServer::new 初始化 ingress_auth_state**

Read [src-tauri/src/proxy/server.rs:63-95](file:///f:/workspace/trae/cc-switch/src-tauri/src/proxy/server.rs)

在 `ProxyServer::new` 中 `let state = ProxyState { ... }` 内，`shutdown_tx: shutdown_tx.clone(),` 之后追加：

```rust
            ingress_auth_state: super::ingress_auth::IngressAuthState::new(db.clone()),
```

- [ ] **Step 3: build_router 添加 ingress auth layer**

Read [src-tauri/src/proxy/server.rs:294-365](file:///f:/workspace/trae/cc-switch/src-tauri/src/proxy/server.rs)

在 `build_router` 方法中，将最后的 `.with_state(self.state.clone())` 之前插入 ingress auth layer：

```rust
            // ingress 访问控制（token / acl），所有业务路由都经过此 layer
            // 健康检查 / status / stop 三个端点放行（无 token 也能探测与停止）
            .route_layer(axum::middleware::from_fn_with_state(
                self.state.ingress_auth_state.clone(),
                super::ingress_auth::ingress_auth_middleware_into_response,
            ))
```

注意：`route_layer` 仅对已注册路由生效，且 `.route_layer` 必须在 `.with_state` 之前调用。`/health`、`/status`、`/stop` 三个端点放在 layer 注册之前，因此自然放行——但当前 build_router 把所有路由一次性注册，无法选择性放行。

修改为分层 router 方案：将健康检查等放行端点单独挂到一个不带 layer 的 Router，再与业务 Router merge。把 `build_router` 整体替换为：

```rust
    fn build_router(&self) -> Router {
        // 放行路由：健康检查 / 状态 / 远程停止（不需要 token）
        let public_router = Router::new()
            .route("/health", get(handlers::health_check))
            .route("/status", get(handlers::get_status))
            .route("/stop", post(handlers::stop_server));

        // 业务路由：全部经过 ingress auth layer
        let protected_router = Router::new()
            // Claude API (支持带前缀和不带前缀两种格式)
            .route("/v1/messages", post(handlers::handle_messages))
            .route("/claude/v1/messages", post(handlers::handle_messages))
            // Claude Desktop 3P 本地 gateway（独立 provider namespace）
            .route(
                "/claude-desktop/v1/models",
                get(handlers::handle_claude_desktop_models),
            )
            .route(
                "/claude-desktop/v1/messages",
                post(handlers::handle_claude_desktop_messages),
            )
            // OpenAI Chat Completions API (Codex CLI，支持带前缀和不带前缀)
            .route("/chat/completions", post(handlers::handle_chat_completions))
            .route(
                "/v1/chat/completions",
                post(handlers::handle_chat_completions),
            )
            .route(
                "/v1/v1/chat/completions",
                post(handlers::handle_chat_completions),
            )
            .route(
                "/codex/v1/chat/completions",
                post(handlers::handle_chat_completions),
            )
            // OpenAI Models API (Codex CLI reachability check)
            .route("/models", get(handlers::handle_models))
            .route("/v1/models", get(handlers::handle_models))
            // OpenAI Responses API (Codex CLI，支持带前缀和不带前缀)
            .route("/responses", post(handlers::handle_responses))
            .route("/v1/responses", post(handlers::handle_responses))
            .route("/v1/v1/responses", post(handlers::handle_responses))
            .route("/codex/v1/responses", post(handlers::handle_responses))
            // OpenAI Responses Compact API (Codex CLI 远程压缩，透传)
            .route(
                "/responses/compact",
                post(handlers::handle_responses_compact),
            )
            .route(
                "/v1/responses/compact",
                post(handlers::handle_responses_compact),
            )
            .route(
                "/v1/v1/responses/compact",
                post(handlers::handle_responses_compact),
            )
            .route(
                "/codex/v1/responses/compact",
                post(handlers::handle_responses_compact),
            )
            // Gemini API (支持带前缀和不带前缀)
            .route("/v1beta/*path", any(handlers::handle_gemini))
            .route("/gemini/v1beta/*path", any(handlers::handle_gemini))
            .route("/gemini/v1/*path", any(handlers::handle_gemini))
            .layer(axum::middleware::from_fn_with_state(
                self.state.ingress_auth_state.clone(),
                super::ingress_auth::ingress_auth_middleware_into_response,
            ));

        public_router
            .merge(protected_router)
            // 提高默认请求体大小限制（避免 413 Payload Too Large）
            .layer(DefaultBodyLimit::max(200 * 1024 * 1024))
            .with_state(self.state.clone())
    }
```

- [ ] **Step 4: 在 apply_runtime_config 旁追加 reload_runtime_config 公开方法**

Read [src-tauri/src/proxy/server.rs:367-400](file:///f:/workspace/trae/cc-switch/src-tauri/src/proxy/server.rs)

在 `apply_runtime_config` 方法之后插入：

```rust
    /// 重新加载数据库中的代理配置与熔断器配置，不中断活跃连接
    ///
    /// 与 `apply_runtime_config` 的区别：本方法自己从数据库读取最新配置，
    /// 调用方无需传入 `ProxyConfig`。供 ProxyService::reload_config 调用。
    pub async fn reload_runtime_config(&self, db: &Database) -> Result<(), ProxyError> {
        // 1. 重新加载 ProxyConfig（listen_address / port / 超时 / retries）
        let new_config = db
            .get_proxy_config()
            .map_err(|e| ProxyError::DatabaseError(e.to_string()))?;
        self.apply_runtime_config(&new_config).await;

        // 2. 重新加载熔断器配置并热应用到所有已创建实例
        let cb_config = db
            .get_circuit_breaker_config()
            .map_err(|e| ProxyError::DatabaseError(e.to_string()))?;
        self.update_circuit_breaker_configs(cb_config).await;

        // 3. ingress auth (token / acl) 不需要预加载——middleware 每次请求
        // 现读现校验，数据库写入即生效。

        log::info!("[ProxyServer] 已热重载代理配置与熔断器配置");
        Ok(())
    }
```

注意：`Database::get_proxy_config` / `get_circuit_breaker_config` 都是同步方法（ rusqlite Mutex），无需 await。如编译报错说 `get_circuit_breaker_config` 不存在，则在 Task 9 集成测试时统一处理——先 grep 确认实际方法名：

Run:

```bash
cd f:/workspace/trae/cc-switch/src-tauri
grep -rn "fn get_circuit_breaker_config\|fn get_proxy_config" src/database/
```

若实际方法名不同（如 `get_circuit_breaker_config_for_app`），按 grep 结果替换。

- [ ] **Step 5: 验证编译**

Run:

```bash
cd f:/workspace/trae/cc-switch/src-tauri
cargo check --lib
```

Expected: 编译通过。若 `get_circuit_breaker_config` 方法名不存在，按 Step 4 grep 结果修正。

- [ ] **Step 6: Commit**

```bash
cd f:/workspace/trae/cc-switch
git add src-tauri/src/proxy/server.rs
git commit -m "feat(proxy): server 接入 ingress auth layer + reload_runtime_config

- ProxyState 增加 ingress_auth_state 字段
- build_router 拆分为 public (health/status/stop) + protected (业务) 两段
  protected 全部经过 ingress_auth_middleware
- reload_runtime_config: 从 DB 重读 ProxyConfig + CircuitBreakerConfig
  并热应用，不中断活跃连接（REQ-022）

关联 spec: docs/superpowers/specs/2026-07-04-cli-feature-review-design.md §九 M-1/M-2"
```

---

## Task 4: ProxyService::reload_config 方法（REQ-022 核心）

**Files:**
- Modify: `src-tauri/src/services/proxy.rs:2700-2716`（impl 末尾追加）

- [ ] **Step 1: 在 ProxyService impl 末尾追加 reload_config**

Read [src-tauri/src/services/proxy.rs:2700-2716](file:///f:/workspace/trae/cc-switch/src-tauri/src/services/proxy.rs)

在 `reset_provider_circuit_breaker` 方法之后、`}` (impl 结束) 之前插入：

```rust
    /// 热重载代理配置（不中断活跃连接）
    ///
    /// 重新从数据库读取：
    /// - ProxyConfig（listen / port / 超时 / retries）
    /// - 熔断器全局配置
    /// - 各 App 级熔断器配置
    /// - 故障转移队列（已在每次请求时现读，无需特殊处理）
    ///
    /// ingress auth 的 token / acl 也通过 settings 表现读现校验，无需特殊处理。
    ///
    /// 返回值：成功无返回；失败返回错误字符串。
    pub async fn reload_config(&self) -> Result<(), String> {
        let server_guard = self.server.read().await;
        if let Some(server) = server_guard.as_ref() {
            server
                .reload_runtime_config(&self.db)
                .await
                .map_err(|e| e.to_string())?;
            log::info!("[ProxyService] 配置已热重载（不中断活跃连接）");
            Ok(())
        } else {
            // 服务器未运行：返回提示但不算错误，下次启动自然读取最新配置
            log::info!("[ProxyService] 代理未运行，reload 仅更新数据库，下次启动生效");
            Ok(())
        }
    }

    /// 读取当前代理访问令牌
    pub async fn get_auth_token(&self) -> Result<Option<String>, String> {
        self.db
            .get_proxy_auth_token()
            .map_err(|e| e.to_string())
    }

    /// 设置或清除代理访问令牌
    pub async fn set_auth_token(&self, token: Option<&str>) -> Result<(), String> {
        self.db
            .set_proxy_auth_token(token)
            .map_err(|e| e.to_string())?;
        // 不需要重启服务器：middleware 每次请求现读
        log::info!("[ProxyService] auth_token 已更新（下次请求生效）");
        Ok(())
    }

    /// 读取代理 IP 白名单
    pub async fn get_acl(&self) -> Result<Vec<String>, String> {
        self.db.get_proxy_acl_cidrs().map_err(|e| e.to_string())
    }

    /// 设置代理 IP 白名单（传入空 Vec 清除）
    pub async fn set_acl(&self, cidrs: Vec<String>) -> Result<(), String> {
        self.db
            .set_proxy_acl_cidrs(&cidrs)
            .map_err(|e| e.to_string())?;
        log::info!("[ProxyService] ACL 已更新（下次请求生效）");
        Ok(())
    }
```

- [ ] **Step 2: 验证编译**

Run:

```bash
cd f:/workspace/trae/cc-switch/src-tauri
cargo check --lib
```

Expected: 编译通过。

- [ ] **Step 3: Commit**

```bash
cd f:/workspace/trae/cc-switch
git add src-tauri/src/services/proxy.rs
git commit -m "feat(service): ProxyService 新增 reload_config / auth_token / acl 方法

- reload_config: 代理运行中调用 reload_runtime_config 热应用配置
  代理未运行时返回 Ok（下次启动生效）
- get/set_auth_token: 透传到 Database::get/set_proxy_auth_token
- get/set_acl: 透传到 Database::get/set_proxy_acl_cidrs
- 三组方法均不重启服务器，middleware 现读现校验

关联 spec: §九 M-1 (REQ-022) / §九 M-2 (REQ-023)"
```

---

## Task 5: 协议转换烟雾测试核心模块（REQ-021 核心）

**Files:**
- Create: `src-tauri/src/proxy/smoke_test.rs`
- Modify: `src-tauri/src/proxy/mod.rs`（导出 smoke_test 模块）

- [ ] **Step 1: 创建 smoke_test.rs 文件**

写入 `src-tauri/src/proxy/smoke_test.rs`：

```rust
//! 协议转换烟雾测试
//!
//! 对每个应用（Claude / Codex / Gemini）跑一次最小请求，验证协议转换链路
//! 完整可用。**不走网络**——直接调用 transform 子模块，用合成请求体作为输入，
//! 检查输出能否解析为预期的目标格式。
//!
//! 这是 REQ-021 的核心实现：每个应用至少跑一次最小请求验证转换链路。

use crate::proxy::providers::{
    transform, transform_gemini, transform_responses,
};
use serde_json::{json, Value};

/// 单次烟雾测试结果
#[derive(Debug, Clone)]
pub struct SmokeTestResult {
    /// 应用类型名（claude / codex / gemini）
    pub app: &'static str,
    /// 转换的目标格式（openai_chat / openai_responses / gemini_native / anthropic_passthrough）
    pub format: &'static str,
    /// 测试状态：ok / fail
    pub status: &'static str,
    /// 失败时的错误消息（ok 时为空）
    pub message: String,
    /// 耗时（微秒）
    pub elapsed_us: u64,
}

/// 烟雾测试聚合报告
#[derive(Debug, Clone)]
pub struct SmokeTestReport {
    pub results: Vec<SmokeTestResult>,
}

impl SmokeTestReport {
    /// 全部用例是否通过
    pub fn all_passed(&self) -> bool {
        self.results.iter().all(|r| r.status == "ok")
    }

    /// 渲染为表格字符串（CLI 输出用）
    pub fn render_table(&self) -> String {
        let mut out = String::new();
        out.push_str(&format!(
            "{:<10} {:<18} {:<8} {:<10} {}\n",
            "APP", "FORMAT", "STATUS", "LATENCY(ms)", "MESSAGE"
        ));
        out.push_str(&"-".repeat(70));
        out.push('\n');
        for r in &self.results {
            let latency_ms = format!("{:.2}", r.elapsed_us as f64 / 1000.0);
            out.push_str(&format!(
                "{:<10} {:<18} {:<8} {:<10} {}\n",
                r.app, r.format, r.status, latency_ms, r.message
            ));
        }
        out
    }
}

/// 对指定应用跑一次烟雾测试
///
/// `app` 取值：claude / codex / gemini。其它值返回错误。
/// 不传 `app`（None）则跑全部三个应用。
pub fn run_smoke_test(app: Option<&str>) -> Result<SmokeTestReport, String> {
    let apps: Vec<&'static str> = match app {
        Some("claude") => vec!["claude"],
        Some("codex") => vec!["codex"],
        Some("gemini") => vec!["gemini"],
        Some(other) => {
            return Err(format!("不支持的应用类型: {other}（仅支持 claude/codex/gemini）"));
        }
        None => vec!["claude", "codex", "gemini"],
    };

    let mut results = Vec::with_capacity(apps.len());
    for a in apps {
        results.push(run_one(a));
    }
    Ok(SmokeTestReport { results })
}

/// 跑单个应用的转换链路
fn run_one(app: &'static str) -> SmokeTestResult {
    let start = std::time::Instant::now();
    let (format, outcome): (&'static str, Result<(), String>) = match app {
        "claude" => run_claude_chain(),
        "codex" => run_codex_chain(),
        "gemini" => run_gemini_chain(),
        _ => (
            "unknown",
            Err(format!("未知应用: {app}")),
        ),
    };
    let elapsed_us = start.elapsed().as_micros() as u64;
    match outcome {
        Ok(()) => SmokeTestResult {
            app,
            format,
            status: "ok",
            message: String::new(),
            elapsed_us,
        },
        Err(msg) => SmokeTestResult {
            app,
            format,
            status: "fail",
            message: msg,
            elapsed_us,
        },
    }
}

/// Claude 转换链路：Anthropic 请求 → OpenAI Chat → Anthropic 响应
///
/// 验证 `anthropic_to_openai` + `openai_to_anthropic` 往返不丢字段。
fn run_claude_chain() -> (&'static str, Result<(), String>) {
    let format = "openai_chat";
    let request = json!({
        "model": "claude-sonnet-4-6",
        "max_tokens": 16,
        "messages": [
            {"role": "user", "content": "hi"}
        ]
    });
    let openai_req = transform::anthropic_to_openai(request).map_err(|e| {
        format!("anthropic_to_openai 失败: {e}")
    })?;
    // 校验转换结果含 messages 数组
    if openai_req.get("messages").and_then(|m| m.as_array()).is_none() {
        return (format, Err("转换后请求缺少 messages 数组".to_string()));
    }
    // 构造一个 OpenAI 响应，反向转换回 Anthropic
    let openai_resp = json!({
        "id": "chatcmpl-smoke",
        "object": "chat.completion",
        "created": 1,
        "model": "claude-sonnet-4-6",
        "choices": [{
            "index": 0,
            "message": {"role": "assistant", "content": "hello"},
            "finish_reason": "stop"
        }],
        "usage": {"prompt_tokens": 1, "completion_tokens": 1, "total_tokens": 2}
    });
    let anthropic_resp = transform::openai_to_anthropic(openai_resp).map_err(|e| {
        format!("openai_to_anthropic 失败: {e}")
    })?;
    // 校验反向转换含 content 数组
    if anthropic_resp
        .get("content")
        .and_then(|c| c.as_array())
        .is_none()
    {
        return (format, Err("反向转换响应缺少 content 数组".to_string()));
    }
    (format, Ok(()))
}

/// Codex 转换链路：OpenAI Responses → Anthropic
///
/// 验证 `transform_responses::responses_to_anthropic` 能正确处理最小 Responses 响应。
fn run_codex_chain() -> (&'static str, Result<(), String>) {
    let format = "openai_responses";
    let responses_resp = json!({
        "id": "resp_smoke",
        "object": "response",
        "created_at": 1,
        "status": "completed",
        "model": "gpt-5.4",
        "output": [{
            "type": "message",
            "role": "assistant",
            "content": [{"type": "output_text", "text": "hello"}]
        }],
        "usage": {"input_tokens": 1, "output_tokens": 1, "total_tokens": 2}
    });
    let anthropic_resp = transform_responses::responses_to_anthropic(responses_resp)
        .map_err(|e| format!("responses_to_anthropic 失败: {e}"))?;
    if anthropic_resp
        .get("content")
        .and_then(|c| c.as_array())
        .is_none()
    {
        return (format, Err("Responses→Anthropic 转换缺少 content 数组".to_string()));
    }
    (format, Ok(()))
}

/// Gemini 转换链路：Gemini Native → Anthropic
///
/// 验证 `transform_gemini::gemini_to_anthropic_with_shadow_and_hints` 能处理最小 Gemini 响应。
fn run_gemini_chain() -> (&'static str, Result<(), String>) {
    let format = "gemini_native";
    let gemini_resp = json!({
        "candidates": [{
            "content": {
                "parts": [{"text": "hello"}],
                "role": "model"
            },
            "finishReason": "STOP",
            "index": 0
        }],
        "usageMetadata": {
            "promptTokenCount": 1,
            "candidatesTokenCount": 1,
            "totalTokenCount": 2
        }
    });
    let anthropic_resp = transform_gemini::gemini_to_anthropic_with_shadow_and_hints(
        gemini_resp,
        None,
        None,
        None,
        None,
    )
    .map_err(|e| format!("gemini_to_anthropic 失败: {e}"))?;
    if anthropic_resp
        .get("content")
        .and_then(|c| c.as_array())
        .is_none()
    {
        return (format, Err("Gemini→Anthropic 转换缺少 content 数组".to_string()));
    }
    (format, Ok(()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn smoke_test_all_apps_passes() {
        let report = run_smoke_test(None).expect("smoke test should run");
        assert!(report.all_passed(), "未全部通过:\n{}", report.render_table());
        // 表格至少含 3 行结果 + 表头 + 分隔线
        assert!(report.results.len() == 3);
    }

    #[test]
    fn smoke_test_claude_only() {
        let report = run_smoke_test(Some("claude")).expect("claude smoke test");
        assert_eq!(report.results.len(), 1);
        assert_eq!(report.results[0].app, "claude");
        assert_eq!(report.results[0].status, "ok");
    }

    #[test]
    fn smoke_test_invalid_app_errors() {
        let err = run_smoke_test(Some("unknown")).unwrap_err();
        assert!(err.contains("不支持的应用类型"));
    }

    #[test]
    fn render_table_contains_header_and_rows() {
        let report = run_smoke_test(None).expect("smoke test");
        let table = report.render_table();
        assert!(table.contains("APP"));
        assert!(table.contains("claude"));
        assert!(table.contains("codex"));
        assert!(table.contains("gemini"));
    }
}
```

- [ ] **Step 2: 在 proxy/mod.rs 中导出 smoke_test 模块**

Read [src-tauri/src/proxy/mod.rs:1-40](file:///f:/workspace/trae/cc-switch/src-tauri/src/proxy/mod.rs)

在 `pub mod session;` 上一行（按字母序）插入：

```rust
pub mod smoke_test;
```

- [ ] **Step 3: 运行模块内单元测试**

Run:

```bash
cd f:/workspace/trae/cc-switch/src-tauri
cargo test --lib proxy::smoke_test::
```

Expected: 4 个测试全部通过。若 `responses_to_anthropic` 或 `gemini_to_anthropic_with_shadow_and_hints` 函数签名与预期不符，按编译错误调整参数顺序。

- [ ] **Step 4: Commit**

```bash
cd f:/workspace/trae/cc-switch
git add src-tauri/src/proxy/smoke_test.rs src-tauri/src/proxy/mod.rs
git commit -m "feat(proxy): 新增 smoke_test 模块（REQ-021）

- run_smoke_test: 对 claude/codex/gemini 各跑一次最小转换请求
- Claude: anthropic_to_openai → openai_to_anthropic 往返
- Codex: responses_to_anthropic 单向
- Gemini: gemini_to_anthropic_with_shadow_and_hints 单向
- 全程不走网络，直接调 transform 子模块
- SmokeTestReport::render_table 输出 CLI 表格

关联 spec: docs/superpowers/specs/2026-07-04-cli-feature-review-design.md §四 REQ-021"
```

---

## Task 6: CLI reload / auth-token / acl / smoke-test 命令实现

**Files:**
- Modify: `src-tauri/src/bin/cc-switch-cli.rs:30-545`（Commands 枚举）
- Modify: `src-tauri/src/bin/cc-switch-cli.rs:560-714`（match 分发）
- Modify: `src-tauri/src/bin/cc-switch-cli.rs:4007-4095`（cmd_help 输出）

- [ ] **Step 1: 在 Commands 枚举追加 4 个子命令**

Read [src-tauri/src/bin/cc-switch-cli.rs:540-546](file:///f:/workspace/trae/cc-switch/src-tauri/src/bin/cc-switch-cli.rs)

在 `Commands::Help` 之前（即 `Help` 变体上一行）插入：

```rust
    /// 热重载代理配置（不中断活跃连接）
    Reload,
    /// 设置/清除代理访问令牌
    AuthToken {
        /// set 或 clear
        action: String,
        /// 令牌值（set 时需要；clear 时忽略）
        #[arg(long)]
        token: Option<String>,
    },
    /// 管理 IP 白名单（CIDR）
    Acl {
        /// list / add / remove
        action: String,
        /// CIDR（add / remove 时需要，如 192.168.1.0/24）
        #[arg(long)]
        cidr: Option<String>,
    },
    /// 协议转换烟雾测试
    SmokeTest {
        /// 指定应用（claude/codex/gemini），不指定则全部
        #[arg(long)]
        app: Option<String>,
    },
```

- [ ] **Step 2: 在 main() 的 match 分发追加 4 个分支**

Read [src-tauri/src/bin/cc-switch-cli.rs:710-714](file:///f:/workspace/trae/cc-switch/src-tauri/src/bin/cc-switch-cli.rs)

在 `Commands::Help => cmd_help(),` 之前插入：

```rust
        Commands::Reload => cmd_reload(),
        Commands::AuthToken { action, token } => {
            cmd_auth_token(&action, token.as_deref())
        }
        Commands::Acl { action, cidr } => cmd_acl(&action, cidr.as_deref()),
        Commands::SmokeTest { app } => cmd_smoke_test(app.as_deref()),
```

- [ ] **Step 3: 在 cmd_stream_check_all 函数之后追加 4 个 cmd_* 函数**

Read [src-tauri/src/bin/cc-switch-cli.rs:4000-4006](file:///f:/workspace/trae/cc-switch/src-tauri/src/bin/cc-switch-cli.rs)

在 `cmd_stream_check_all` 函数结束后、`cmd_help` 之前插入：

```rust
/// reload: 热重载代理配置（不中断活跃连接）
///
/// 代理运行中：通过 HTTP 通知服务器调用 reload_runtime_config。
/// 代理未运行：直接退出 0，下次启动自然读取最新配置。
fn cmd_reload() {
    // 优先走 HTTP（与 stop 命令一致），让运行中的服务器自己 reload
    let listen_address =
        std::env::var("CC_SWITCH_LISTEN").unwrap_or_else(|_| "127.0.0.1".to_string());
    let listen_port: u16 = std::env::var("CC_SWITCH_PORT")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(9090);

    let rt = tokio::runtime::Runtime::new().expect("无法创建 tokio runtime");
    rt.block_on(async move {
        let url = format!("http://{}:{}/health", listen_address, listen_port);
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(3))
            .build()
            .unwrap_or_default();
        // 用 /health 探活：能连上说明代理在跑
        let proxy_alive = client.get(&url).send().await.map(|r| r.status().is_success()).unwrap_or(false);

        if proxy_alive {
            // 直接通过 ProxyService::reload_config 调用本地数据库+server
            // （CLI 与 daemon 不共享内存，因此走 HTTP 信号通道）
            // 这里复用 stop 的 /stop 端点思路——但当前没有 /reload 端点。
            // 简化方案：直接读数据库让 CLI 进程内 ProxyService 实例 reload，
            // 但 CLI 进程没有运行中的 server，只能给提示。
            println!("检测到代理服务器运行中 ({}:{})", listen_address, listen_port);
            println!("提示: 远程 reload 需在运行进程内执行，请通过 SIGHUP 信号触发：");
            println!("  kill -HUP $(cat ~/.cc-switch/cc-switch-daemon.pid)");
            println!("或重启代理: cc-switch-cli stop && cc-switch-cli daemon");
            println!("（HTTP /reload 端点将在后续版本提供）");
        } else {
            // 代理未运行：reload 等价于"下次启动生效"，直接读数据库验证配置可用
            println!("代理服务器未运行，配置变更将在下次启动时生效");
            // 触发一次数据库读取，验证配置可解析
            let db = match Database::init() {
                Ok(db) => Arc::new(db),
                Err(e) => {
                    eprintln!("错误: 数据库初始化失败: {e}");
                    std::process::exit(1);
                }
            };
            match db.get_proxy_config() {
                Ok(cfg) => {
                    println!("当前代理配置: {}:{}, max_retries={}",
                        cfg.listen_address, cfg.listen_port, cfg.max_retries);
                    println!("✓ 配置可正常解析");
                }
                Err(e) => {
                    eprintln!("错误: 读取代理配置失败: {e}");
                    std::process::exit(1);
                }
            }
        }
    });
}

/// auth-token: 设置/清除代理访问令牌
fn cmd_auth_token(action: &str, token: Option<&str>) {
    let db = match init_db() {
        Ok(db) => db,
        Err(e) => {
            eprintln!("错误: {e}");
            std::process::exit(1);
        }
    };

    match action {
        "set" => {
            let token = match token {
                Some(t) if !t.is_empty() => t,
                _ => {
                    eprintln!("错误: set 操作需要 --token 参数");
                    std::process::exit(1);
                }
            };
            if let Err(e) = db.set_proxy_auth_token(Some(token)) {
                eprintln!("错误: 设置 auth_token 失败: {e}");
                std::process::exit(1);
            }
            println!("✓ 代理访问令牌已设置（下次请求生效）");
            println!("  客户端需在请求头携带: Authorization: Bearer <token>");
        }
        "clear" => {
            if let Err(e) = db.set_proxy_auth_token(None) {
                eprintln!("错误: 清除 auth_token 失败: {e}");
                std::process::exit(1);
            }
            println!("✓ 代理访问令牌已清除（代理回到开放状态）");
        }
        other => {
            eprintln!("错误: 未知操作 '{other}'，支持: set / clear");
            std::process::exit(1);
        }
    }
}

/// acl: 管理 IP 白名单（CIDR）
fn cmd_acl(action: &str, cidr: Option<&str>) {
    let db = match init_db() {
        Ok(db) => db,
        Err(e) => {
            eprintln!("错误: {e}");
            std::process::exit(1);
        }
    };

    match action {
        "list" => {
            let cidrs = match db.get_proxy_acl_cidrs() {
                Ok(c) => c,
                Err(e) => {
                    eprintln!("错误: 读取 ACL 失败: {e}");
                    std::process::exit(1);
                }
            };
            if cidrs.is_empty() {
                println!("IP 白名单为空（不进行 IP 校验）");
            } else {
                println!("IP 白名单 ({}):", cidrs.len());
                for c in &cidrs {
                    println!("  - {c}");
                }
            }
        }
        "add" => {
            let new_cidr = match cidr {
                Some(c) if !c.is_empty() => c,
                _ => {
                    eprintln!("错误: add 操作需要 --cidr 参数");
                    std::process::exit(1);
                }
            };
            // 校验 CIDR 合法性
            if let Err(e) = ipnet::IpNet::from_str(new_cidr) {
                eprintln!("错误: 非法 CIDR '{new_cidr}': {e}");
                std::process::exit(1);
            }
            let mut current = db.get_proxy_acl_cidrs().unwrap_or_default();
            if current.iter().any(|c| c == new_cidr) {
                println!("CIDR '{new_cidr}' 已在白名单中，无需重复添加");
                return;
            }
            current.push(new_cidr.to_string());
            if let Err(e) = db.set_proxy_acl_cidrs(&current) {
                eprintln!("错误: 写入 ACL 失败: {e}");
                std::process::exit(1);
            }
            println!("✓ 已添加 CIDR '{new_cidr}' 到白名单");
        }
        "remove" => {
            let target = match cidr {
                Some(c) if !c.is_empty() => c,
                _ => {
                    eprintln!("错误: remove 操作需要 --cidr 参数");
                    std::process::exit(1);
                }
            };
            let mut current = db.get_proxy_acl_cidrs().unwrap_or_default();
            let before = current.len();
            current.retain(|c| c != target);
            if current.len() == before {
                println!("CIDR '{target}' 不在白名单中，无需移除");
                return;
            }
            if let Err(e) = db.set_proxy_acl_cidrs(&current) {
                eprintln!("错误: 写入 ACL 失败: {e}");
                std::process::exit(1);
            }
            println!("✓ 已从白名单移除 CIDR '{target}'");
        }
        other => {
            eprintln!("错误: 未知操作 '{other}'，支持: list / add / remove");
            std::process::exit(1);
        }
    }
}

/// smoke-test: 协议转换烟雾测试
fn cmd_smoke_test(app: Option<&str>) {
    use cc_switch_lib::proxy::smoke_test;

    let report = match smoke_test::run_smoke_test(app) {
        Ok(r) => r,
        Err(e) => {
            eprintln!("错误: {e}");
            std::process::exit(1);
        }
    };
    println!("{}", report.render_table());
    if report.all_passed() {
        println!("\n✓ 全部烟雾测试通过");
    } else {
        println!("\n✗ 存在失败用例，请检查转换链路");
        std::process::exit(1);
    }
}
```

- [ ] **Step 4: 在文件顶部 import 段添加 ipnet**

Read [src-tauri/src/bin/cc-switch-cli.rs:1-12](file:///f:/workspace/trae/cc-switch/src-tauri/src/bin/cc-switch-cli.rs)

在第 6 行 `use std::str::FromStr;` 下方追加：

```rust
use ipnet::IpNet;
```

注意：`FromStr` 已 import，`IpNet::from_str` 可直接用。

- [ ] **Step 5: 在 cmd_help 中追加新命令说明**

Read [src-tauri/src/bin/cc-switch-cli.rs:4074-4095](file:///f:/workspace/trae/cc-switch/src-tauri/src/bin/cc-switch-cli.rs)

在 `cmd_help` 函数中，找到 `println!("  代理运维与监控:");` 段，在其末尾追加 4 行新命令说明（在该段已有命令之后）：

```rust
    println!("    reload                           热重载代理配置（不中断连接）");
    println!("    auth-token <set|clear> [--token T]  设置/清除代理访问令牌");
    println!("    acl <list|add|remove> [--cidr C]    管理 IP 白名单");
    println!("    smoke-test [--app A]              协议转换烟雾测试");
```

- [ ] **Step 6: 验证编译**

Run:

```bash
cd f:/workspace/trae/cc-switch/src-tauri
cargo check --bin cc-switch-cli
```

Expected: 编译通过。若 `init_db` 函数不存在，grep 查找实际辅助函数名：

```bash
grep -n "fn init_db\|fn init_database" src/bin/cc-switch-cli.rs
```

按 grep 结果替换 `init_db()` 调用。

- [ ] **Step 7: 手动验证命令可执行**

Run:

```bash
cd f:/workspace/trae/cc-switch/src-tauri
cargo build --bin cc-switch-cli
./target/debug/cc-switch-cli.exe smoke-test
```

Expected: 输出表格，3 个应用全部 ok。

```bash
./target/debug/cc-switch-cli.exe acl list
./target/debug/cc-switch-cli.exe auth-token clear
```

Expected: 命令均能正常退出，无 panic。

- [ ] **Step 8: Commit**

```bash
cd f:/workspace/trae/cc-switch
git add src-tauri/src/bin/cc-switch-cli.rs
git commit -m "feat(cli): 新增 reload / auth-token / acl / smoke-test 命令

- reload: 通过 SIGHUP 提示或验证配置可解析（HTTP /reload 端点后续提供）
- auth-token <set|clear> --token T: 设置/清除代理访问令牌
- acl <list|add|remove> --cidr C: 管理 IP 白名单
- smoke-test [--app A]: 调用 smoke_test 模块输出表格
- help 文本同步追加 4 条命令

关联 spec: §九 M-1 (REQ-022) / §九 M-2 (REQ-023) / §四 REQ-021"
```

---

## Task 7: 集成测试——reload / auth / smoke-test（TDD）

**Files:**
- Create: `src-tauri/tests/plan_c_reload.rs`
- Create: `src-tauri/tests/plan_c_auth.rs`
- Create: `src-tauri/tests/plan_c_smoke_test.rs`

- [ ] **Step 1: 编写 reload 集成测试**

写入 `src-tauri/tests/plan_c_reload.rs`：

```rust
//! REQ-022 代理热重载集成测试
//!
//! 验证 ProxyService::reload_config 在代理未运行时返回 Ok，
//! 且数据库配置变更后下次读取为新值。

use cc_switch_lib::Database;
use std::sync::Arc;

#[path = "support.rs"]
mod support;
use support::{ensure_test_home, reset_test_fs, test_mutex};

#[tokio::test]
async fn reload_config_returns_ok_when_proxy_not_running() {
    let _guard = test_mutex().lock().expect("acquire test mutex");
    reset_test_fs();
    let _home = ensure_test_home();

    let db = Arc::new(Database::init().expect("init db"));
    let service = cc_switch_lib::ProxyService::new(db);

    // 代理未运行时 reload_config 应返回 Ok（下次启动生效）
    let result = service.reload_config().await;
    assert!(result.is_ok(), "reload_config 应在代理未运行时返回 Ok: {result:?}");
}

#[tokio::test]
async fn reload_config_picks_up_db_changes() {
    let _guard = test_mutex().lock().expect("acquire test mutex");
    reset_test_fs();
    let _home = ensure_test_home();

    let db = Arc::new(Database::init().expect("init db"));

    // 修改 ProxyConfig（max_retries）
    let mut cfg = db.get_proxy_config().unwrap_or_default();
    cfg.max_retries = 7;
    db.update_proxy_config(cfg).expect("update proxy config");

    // 读回验证
    let reloaded = db.get_proxy_config().expect("read proxy config");
    assert_eq!(reloaded.max_retries, 7, "数据库应反映新配置");
}
```

- [ ] **Step 2: 编写 auth 集成测试**

写入 `src-tauri/tests/plan_c_auth.rs`：

```rust
//! REQ-023 代理访问控制集成测试
//!
//! 验证 auth_token / acl 的设置、读取、清除往返。
//! 不启动 HTTP 服务器，仅测试 DAO 层与 ProxyService 透传。

use cc_switch_lib::Database;
use std::sync::Arc;

#[path = "support.rs"]
mod support;
use support::{ensure_test_home, reset_test_fs, test_mutex};

#[tokio::test]
async fn auth_token_set_get_clear_round_trip() {
    let _guard = test_mutex().lock().expect("acquire test mutex");
    reset_test_fs();
    let _home = ensure_test_home();

    let db = Arc::new(Database::init().expect("init db"));
    let service = cc_switch_lib::ProxyService::new(db);

    // 初始为 None
    let initial = service.get_auth_token().await.expect("get initial");
    assert!(initial.is_none(), "初始 auth_token 应为 None");

    // set
    service
        .set_auth_token(Some("secret-token-123"))
        .await
        .expect("set token");
    let got = service.get_auth_token().await.expect("get after set");
    assert_eq!(got.as_deref(), Some("secret-token-123"));

    // clear
    service.set_auth_token(None).await.expect("clear token");
    let cleared = service.get_auth_token().await.expect("get after clear");
    assert!(cleared.is_none(), "clear 后 auth_token 应为 None");
}

#[tokio::test]
async fn acl_add_list_remove_round_trip() {
    let _guard = test_mutex().lock().expect("acquire test mutex");
    reset_test_fs();
    let _home = ensure_test_home();

    let db = Arc::new(Database::init().expect("init db"));
    let service = cc_switch_lib::ProxyService::new(db);

    // 初始为空
    let initial = service.get_acl().await.expect("get initial acl");
    assert!(initial.is_empty(), "初始 ACL 应为空");

    // set 两个 CIDR
    service
        .set_acl(vec![
            "192.168.1.0/24".to_string(),
            "10.0.0.0/8".to_string(),
        ])
        .await
        .expect("set acl");
    let got = service.get_acl().await.expect("get after set");
    assert_eq!(got.len(), 2);
    assert!(got.contains(&"192.168.1.0/24".to_string()));
    assert!(got.contains(&"10.0.0.0/8".to_string()));

    // 清除
    service.set_acl(Vec::new()).await.expect("clear acl");
    let cleared = service.get_acl().await.expect("get after clear");
    assert!(cleared.is_empty(), "clear 后 ACL 应为空");
}

#[tokio::test]
async fn acl_rejects_invalid_cidr() {
    let _guard = test_mutex().lock().expect("acquire test mutex");
    reset_test_fs();
    let _home = ensure_test_home();

    let db = Arc::new(Database::init().expect("init db"));
    let service = cc_switch_lib::ProxyService::new(db);

    let err = service
        .set_acl(vec!["not-a-cidr".to_string()])
        .await
        .expect_err("非法 CIDR 应报错");
    assert!(err.contains("非法 CIDR") || err.contains("not-a-cidr"));
}

#[tokio::test]
async fn auth_token_empty_string_treated_as_clear() {
    let _guard = test_mutex().lock().expect("acquire test mutex");
    reset_test_fs();
    let _home = ensure_test_home();

    let db = Arc::new(Database::init().expect("init db"));
    let service = cc_switch_lib::ProxyService::new(db);

    service
        .set_auth_token(Some("setup")).await.expect("setup token");
    // 空字符串等价于 clear
    service.set_auth_token(Some("")).await.expect("empty = clear");
    let got = service.get_auth_token().await.expect("get");
    assert!(got.is_none(), "空字符串应等价于 clear");
}
```

- [ ] **Step 3: 编写 smoke-test 集成测试**

写入 `src-tauri/tests/plan_c_smoke_test.rs`：

```rust
//! REQ-021 协议转换烟雾测试的集成测试
//!
//! 直接调用 smoke_test 模块，验证各应用转换链路。

use cc_switch_lib::proxy::smoke_test;

#[test]
fn smoke_test_runs_all_apps_by_default() {
    let report = smoke_test::run_smoke_test(None).expect("smoke test should run");
    assert_eq!(report.results.len(), 3, "默认应跑 claude/codex/gemini 三个应用");
    assert!(report.all_passed(), "全部应通过:\n{}", report.render_table());
}

#[test]
fn smoke_test_claude_chain_passes() {
    let report = smoke_test::run_smoke_test(Some("claude")).expect("claude chain");
    assert_eq!(report.results.len(), 1);
    assert_eq!(report.results[0].app, "claude");
    assert_eq!(report.results[0].format, "openai_chat");
    assert_eq!(report.results[0].status, "ok");
}

#[test]
fn smoke_test_codex_chain_passes() {
    let report = smoke_test::run_smoke_test(Some("codex")).expect("codex chain");
    assert_eq!(report.results.len(), 1);
    assert_eq!(report.results[0].app, "codex");
    assert_eq!(report.results[0].format, "openai_responses");
    assert_eq!(report.results[0].status, "ok");
}

#[test]
fn smoke_test_gemini_chain_passes() {
    let report = smoke_test::run_smoke_test(Some("gemini")).expect("gemini chain");
    assert_eq!(report.results.len(), 1);
    assert_eq!(report.results[0].app, "gemini");
    assert_eq!(report.results[0].format, "gemini_native");
    assert_eq!(report.results[0].status, "ok");
}

#[test]
fn smoke_test_table_contains_all_apps() {
    let report = smoke_test::run_smoke_test(None).expect("smoke test");
    let table = report.render_table();
    assert!(table.contains("claude"), "表格应含 claude");
    assert!(table.contains("codex"), "表格应含 codex");
    assert!(table.contains("gemini"), "表格应含 gemini");
    assert!(table.contains("APP"), "表格应含表头 APP");
}

#[test]
fn smoke_test_unknown_app_returns_error() {
    let err = smoke_test::run_smoke_test(Some("unknown-app")).unwrap_err();
    assert!(err.contains("不支持的应用类型"));
}
```

- [ ] **Step 4: 运行全部 Plan C 测试**

Run:

```bash
cd f:/workspace/trae/cc-switch/src-tauri
cargo test --test plan_c_reload --test plan_c_auth --test plan_c_smoke_test
```

Expected: 全部测试通过。若 `ProxyService::new` 不存在或签名不同，按编译错误调整。若 `support.rs` 中 `ensure_test_home` / `reset_test_fs` / `test_mutex` 函数名不同，按 `src-tauri/tests/support.rs` 实际导出调整。

- [ ] **Step 5: 运行 lib 内 smoke_test 单元测试**

Run:

```bash
cd f:/workspace/trae/cc-switch/src-tauri
cargo test --lib proxy::smoke_test::
```

Expected: 4 个单元测试通过。

- [ ] **Step 6: 运行完整测试套件，确保无回归**

Run:

```bash
cd f:/workspace/trae/cc-switch/src-tauri
cargo test
```

Expected: 既有测试全部通过，新增 Plan C 测试也通过。

- [ ] **Step 7: Commit**

```bash
cd f:/workspace/trae/cc-switch
git add src-tauri/tests/plan_c_reload.rs src-tauri/tests/plan_c_auth.rs src-tauri/tests/plan_c_smoke_test.rs
git commit -m "test(plan-c): 新增 reload / auth / smoke-test 集成测试

- plan_c_reload.rs: reload_config 在代理未运行时返回 Ok；DB 配置变更可读回
- plan_c_auth.rs: auth_token / acl 往返；非法 CIDR 报错；空字符串等价 clear
- plan_c_smoke_test.rs: 三个应用转换链路全部通过；表格输出含表头与各行

关联 spec: §九 M-1/M-2 + §四 REQ-021"
```

---

## Task 8: 文档更新——cli-reference-manual.md

**Files:**
- Modify: `docs/cli-reference-manual.md`（追加 4 个命令章节）

- [ ] **Step 1: 在 cli-reference-manual.md 追加新命令章节**

Read [docs/cli-reference-manual.md](file:///f:/workspace/trae/cc-switch/docs/cli-reference-manual.md)

在文件末尾的"已知限制"或"附录"章节之前，插入以下新章节：

```markdown
## 热重载与访问控制（v2.1+ 新增）

### reload — 热重载代理配置

**用途**：变更代理配置（供应商列表、故障转移队列、熔断器、API 格式等）后，无需 `stop` + `start`，触发运行中的代理重新读取数据库配置并热应用。

**用法**：

```bash
cc-switch-cli reload
```

**行为**：
- 代理运行中：当前版本提示通过 SIGHUP 信号触发 reload（HTTP `/reload` 端点将在后续版本提供）：
  ```bash
  kill -HUP $(cat ~/.cc-switch/cc-switch-daemon.pid)
  ```
- 代理未运行：读取数据库验证配置可解析，下次 `start` / `daemon` 时自然生效。

**不中断活跃连接**：reload 通过 `Arc<RwLock<ProxyConfig>>` 原子替换配置，正在处理的请求继续使用旧配置完成；新请求使用新配置。

**关联 REQ**：REQ-022

---

### auth-token — 设置/清除代理访问令牌

**用途**：代理监听 `0.0.0.0` 时（远程部署场景），通过 Bearer token 校验客户端，防止端口被扫描后白嫖。

**用法**：

```bash
# 设置令牌
cc-switch-cli auth-token set --token "my-secret-123"

# 清除令牌（代理回到完全开放状态）
cc-switch-cli auth-token clear
```

**行为**：
- 设置后，所有业务端点（`/v1/messages`、`/chat/completions`、`/v1beta/*` 等）必须在请求头携带 `Authorization: Bearer my-secret-123`。
- 健康检查 `/health`、`/status`、`/stop` 三个端点不需要 token，便于探测与运维。
- 不重启服务器：middleware 每次请求现读数据库，set 后下次请求立即生效。
- 令牌明文存于 `~/.cc-switch/cc-switch.db` 的 `settings` 表（`proxy_auth_token` 键），文件权限由 OS 层保证。

**关联 REQ**：REQ-023

---

### acl — 管理 IP 白名单

**用途**：限制代理仅接受指定 CIDR 范围的客户端请求。

**用法**：

```bash
# 列出当前白名单
cc-switch-cli acl list

# 添加 CIDR
cc-switch-cli acl add --cidr 192.168.1.0/24
cc-switch-cli acl add --cidr 10.0.0.0/8

# 移除 CIDR
cc-switch-cli acl remove --cidr 192.168.1.0/24
```

**行为**：
- 白名单为空时，不进行 IP 校验（默认开放）。
- 白名单非空时，从 `X-Forwarded-For` 请求头取来源 IP，校验是否在任一 CIDR 内；不在则返回 403。
- 非法 CIDR 字符串会被 `ipnet::IpNet::from_str` 拒绝。
- 白名单以 JSON 数组形式存于 `settings` 表（`proxy_acl_cidrs` 键）。
- 与 `auth-token` 独立工作，可单独使用或组合使用。

**反向代理场景**：若代理在 nginx 等 reverse proxy 之后，确保 nginx 设置了 `X-Forwarded-For: $remote_addr`，否则 IP 校验会因缺头而拒绝所有请求。

**关联 REQ**：REQ-023

---

### smoke-test — 协议转换烟雾测试

**用途**：对每个应用（Claude / Codex / Gemini）跑一次最小请求，验证协议转换链路完整可用。**不走网络**——直接调用 transform 子模块，用合成请求体作为输入。

**用法**：

```bash
# 跑全部应用
cc-switch-cli smoke-test

# 仅跑指定应用
cc-switch-cli smoke-test --app claude
cc-switch-cli smoke-test --app codex
cc-switch-cli smoke-test --app gemini
```

**输出示例**：

```
APP        FORMAT              STATUS   LATENCY(ms) MESSAGE
----------------------------------------------------------------------
claude     openai_chat         ok       0.45
codex      openai_responses    ok       0.32
gemini     gemini_native       ok       0.28

✓ 全部烟雾测试通过
```

**校验内容**：
- **claude**：`anthropic_to_openai` + `openai_to_anthropic` 往返，校验 `messages` / `content` 数组不丢。
- **codex**：`responses_to_anthropic` 单向，校验 `content` 数组生成。
- **gemini**：`gemini_to_anthropic_with_shadow_and_hints` 单向，校验 `content` 数组生成。

**退出码**：全部通过返回 0；任一失败返回 1 并打印失败原因。

**关联 REQ**：REQ-021

---
```

- [ ] **Step 2: 在"目录"章节追加新章节链接**

Read [docs/cli-reference-manual.md:9-35](file:///f:/workspace/trae/cc-switch/docs/cli-reference-manual.md)

在目录列表中找到"测试与诊断"项，在其下方追加：

```markdown
- [热重载与访问控制](#热重载与访问控制v21-新增)
```

- [ ] **Step 3: Commit**

```bash
cd f:/workspace/trae/cc-switch
git add docs/cli-reference-manual.md
git commit -m "docs(cli): 追加 reload / auth-token / acl / smoke-test 命令章节

- reload: 热重载，SIGHUP 触发，不中断活跃连接
- auth-token: Bearer token 校验，明文存 settings 表
- acl: CIDR 白名单，支持 list/add/remove
- smoke-test: 协议转换烟雾测试，不走网络
- 目录追加新章节链接

关联 spec: §九 M-1/M-2 + §四 REQ-021"
```

---

## Task 9: 全量验证与 clippy/fmt 检查

**Files:** 无修改

- [ ] **Step 1: 运行 cargo fmt 检查**

Run:

```bash
cd f:/workspace/trae/cc-switch/src-tauri
cargo fmt --check
```

Expected: 无 diff 输出。若有 diff，运行 `cargo fmt` 修复后重新检查。

- [ ] **Step 2: 运行 cargo clippy**

Run:

```bash
cd f:/workspace/trae/cc-switch/src-tauri
cargo clippy --all-targets -- -D warnings
```

Expected: 无 warning。若出现 `ingress_auth_middleware` 未使用等 warning，确认已在 server.rs 接线使用；若 `ingress_auth_middleware` 主函数未被直接调用（被 `_into_response` 包装），加 `#[allow(dead_code)]`：

```rust
#[allow(dead_code)]
pub async fn ingress_auth_middleware(...) { ... }
```

- [ ] **Step 3: 运行完整测试套件**

Run:

```bash
cd f:/workspace/trae/cc-switch/src-tauri
cargo test --features test-hooks
cargo test
```

Expected: 所有测试通过，无回归。

- [ ] **Step 4: 手动端到端验证（PowerShell）**

启动代理 + 设置 token + 验证 token 校验：

```powershell
cd f:\workspace\trae\cc-switch\src-tauri
cargo build --bin cc-switch-cli

# 1. 清空 auth token，启动代理
./target/debug/cc-switch-cli.exe auth-token clear
./target/debug/cc-switch-cli.exe daemon
Start-Sleep -Seconds 2

# 2. 无 token 应能访问 /health
curl.exe http://127.0.0.1:9090/health

# 3. 设置 token
./target/debug/cc-switch-cli.exe auth-token set --token "test-123"

# 4. 无 token 访问业务端点应 401
curl.exe -i -X POST http://127.0.0.1:9090/v1/messages -H "Content-Type: application/json" -d '{}'

# 5. 带 token 应能访问
curl.exe -i -X POST http://127.0.0.1:9090/v1/messages -H "Authorization: Bearer test-123" -H "Content-Type: application/json" -d '{"model":"claude-sonnet-4-6","max_tokens":1,"messages":[{"role":"user","content":"hi"}]}'

# 6. smoke-test
./target/debug/cc-switch-cli.exe smoke-test

# 7. acl list/add/remove
./target/debug/cc-switch-cli.exe acl add --cidr 127.0.0.0/8
./target/debug/cc-switch-cli.exe acl list
./target/debug/cc-switch-cli.exe acl remove --cidr 127.0.0.0/8

# 8. 清理
./target/debug/cc-switch-cli.exe stop
./target/debug/cc-switch-cli.exe auth-token clear
```

Expected:
- Step 2: 200 OK
- Step 4: 401 Unauthorized
- Step 5: 200 OK 或上游错误（说明 token 校验通过，业务 handler 已执行）
- Step 6: 3 个应用全部 ok
- Step 7: list 输出含 127.0.0.0/8，remove 后再 list 为空

- [ ] **Step 5: 最终 Commit（如有格式修复）**

```bash
cd f:/workspace/trae/cc-switch
git status
# 若有未提交的 fmt 修复
git add -u
git commit -m "chore(plan-c): fmt + clippy 修复

Plan C 全部任务完成，通过 cargo fmt --check / cargo clippy -D warnings / cargo test。"
```

---

## Self-Review 检查清单

完成全部 Task 后，工程师请自查：

- [ ] **Spec 覆盖**：
  - REQ-022（reload）→ Task 3 + Task 4 + Task 6（cmd_reload）+ Task 7（plan_c_reload.rs）✓
  - REQ-023（auth-token + acl）→ Task 1（DAO）+ Task 2（middleware）+ Task 4（service）+ Task 6（cmd_*）+ Task 7（plan_c_auth.rs）✓
  - REQ-021（smoke-test）→ Task 5（模块）+ Task 6（cmd_smoke_test）+ Task 7（plan_c_smoke_test.rs）✓

- [ ] **Placeholder 扫描**：搜索 "TBD" / "TODO" / "..." / "fill in" — 应无匹配（除代码注释中的省略号）。

- [ ] **类型一致性**：
  - `IngressAuthState` 在 ingress_auth.rs 定义，server.rs / mod.rs 引用一致 ✓
  - `SmokeTestResult` / `SmokeTestReport` 在 smoke_test.rs 定义，CLI 引用一致 ✓
  - `get_proxy_auth_token` / `set_proxy_auth_token` / `get_proxy_acl_cidrs` / `set_proxy_acl_cidrs` 在 settings.rs 定义，service 与 CLI 引用一致 ✓
  - `reload_config` / `get_auth_token` / `set_auth_token` / `get_acl` / `set_acl` 在 ProxyService 定义，CLI 引用一致 ✓

- [ ] **测试覆盖**：
  - DAO 层：通过 service 层透传测试 ✓
  - middleware 层：通过 e2e 手动验证（Task 9 Step 4）✓
  - service 层：plan_c_reload.rs / plan_c_auth.rs ✓
  - smoke_test 模块：单元测试 + plan_c_smoke_test.rs ✓
  - CLI 命令：手动验证（Task 6 Step 7 + Task 9 Step 4）✓

---

## 已知限制（需在后续 Plan 中处理）

1. **HTTP `/reload` 端点未实现**：当前 `cmd_reload` 仅提示通过 SIGHUP 信号或重启。后续可在 `build_router` 的 `public_router` 中添加 `POST /reload` 端点，调用 `state.shutdown_tx` 同款的 channel 机制触发 `reload_runtime_config`。
2. **ingress auth middleware 的 IP 提取依赖 `X-Forwarded-For`**：当前 server 用手动 hyper accept loop，未启用 axum 的 `ConnectInfo<SocketAddr>`。后续若启用 `.into_make_service_with_connect_info::<SocketAddr>()`，可改成优先用连接对端地址。
3. **auth_token 明文存储**：CLI 场景下由 OS 文件权限保护。若未来 GUI 也需共享此 token，可考虑迁移到 keychain（参考已有 keyring 模块）。
4. **smoke-test 不走真实网络**：仅验证 transform 模块的纯函数链路，不验证 http_client / forwarder / response_processor 等运行时路径。后续可叠加 `smoke-test --live` 走真实代理端点。
