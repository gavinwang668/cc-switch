//! 代理入口访问控制（REQ-023）
//!
//! Axum middleware：校验 `Authorization: Bearer <token>` 与来源 IP CIDR 白名单。
//! 无头服务器部署时，防止代理端口被未经授权访问。

use axum::{
    extract::{ConnectInfo, Request},
    http::StatusCode,
    middleware::Next,
    response::Response,
};
use std::net::SocketAddr;

/// Ingress 认证中间件状态
#[derive(Clone)]
pub struct IngressAuthLayer {
    /// Bearer token（None = 不校验 token）
    pub token: Option<String>,
    /// CIDR 白名单（空 = 不校验 IP）
    pub acl_cidrs: Vec<String>,
}

impl IngressAuthLayer {
    pub fn new(token: Option<String>, acl_cidrs: Vec<String>) -> Self {
        Self { token, acl_cidrs }
    }

    /// 是否启用了任何访问控制
    pub fn is_enabled(&self) -> bool {
        self.token.is_some() || !self.acl_cidrs.is_empty()
    }
}

/// Axum middleware 函数
pub async fn ingress_auth_middleware(
    layer: axum::extract::State<IngressAuthLayer>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    req: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    // 1. 校验 IP CIDR
    if !layer.acl_cidrs.is_empty() {
        let ip = addr.ip();
        let allowed = layer.acl_cidrs.iter().any(|cidr_str| {
            if let Ok(net) = cidr_str.parse::<ipnet::IpNet>() {
                net.contains(&ip)
            } else {
                false
            }
        });
        if !allowed {
            log::warn!("Ingress ACL 拒绝: {ip} (不在白名单中)");
            return Err(StatusCode::FORBIDDEN);
        }
    }

    // 2. 校验 Bearer token
    if let Some(expected) = &layer.token {
        let auth_header = req
            .headers()
            .get("authorization")
            .and_then(|v| v.to_str().ok())
            .unwrap_or("");

        let provided = auth_header.strip_prefix("Bearer ").unwrap_or("").trim();

        if provided != expected.as_str() {
            log::warn!("Ingress auth token 不匹配");
            return Err(StatusCode::UNAUTHORIZED);
        }
    }

    Ok(next.run(req).await)
}
