use std::{env, sync::Arc};

use axum::{
    Router,
    body::Bytes,
    extract::State,
    http::{HeaderMap, StatusCode},
    routing::post,
};
use hmac::{Hmac, Mac};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use sha2::Sha256;
use tracing::{error, info, warn};

// ---------------------------------------------------------------------------
// Config & shared state
// ---------------------------------------------------------------------------

struct AppState {
    webhook_secret: String,
    lark_webhook_url: String,
    http: Client,
}

// ---------------------------------------------------------------------------
// Linear webhook models
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
struct LinearPayload {
    action: String,
    #[serde(rename = "type")]
    kind: String,
    data: Issue,
    url: String,
}

#[derive(Debug, Deserialize)]
struct Issue {
    #[allow(dead_code)]
    id: String,
    title: String,
    priority: u8,
    state: IssueState,
    assignee: Option<Assignee>,
    identifier: String,
}

#[derive(Debug, Deserialize)]
struct IssueState {
    name: String,
}

#[derive(Debug, Deserialize)]
struct Assignee {
    name: String,
}

// ---------------------------------------------------------------------------
// Lark card models
// ---------------------------------------------------------------------------

#[derive(Serialize)]
struct LarkMessage {
    msg_type: &'static str,
    card: LarkCard,
}

#[derive(Serialize)]
struct LarkCard {
    header: LarkHeader,
    elements: Vec<serde_json::Value>,
}

#[derive(Serialize)]
struct LarkHeader {
    template: &'static str,
    title: LarkTitle,
}

#[derive(Serialize)]
struct LarkTitle {
    content: String,
    tag: &'static str,
}

// ---------------------------------------------------------------------------
// Signature verification
// ---------------------------------------------------------------------------

fn verify_signature(secret: &str, body: &[u8], signature: &str) -> bool {
    let Ok(mut mac) = Hmac::<Sha256>::new_from_slice(secret.as_bytes()) else {
        return false;
    };
    mac.update(body);
    let expected = hex::encode(mac.finalize().into_bytes());
    expected == signature
}

// ---------------------------------------------------------------------------
// Priority → colour mapping
// ---------------------------------------------------------------------------

fn priority_color(priority: u8) -> &'static str {
    match priority {
        1 => "red",
        2 => "orange",
        3 => "yellow",
        _ => "blue", // 0 (No priority) and 4 (Low)
    }
}

fn priority_label(priority: u8) -> &'static str {
    match priority {
        1 => "Urgent",
        2 => "High",
        3 => "Medium",
        4 => "Low",
        _ => "None",
    }
}

// ---------------------------------------------------------------------------
// Build the Lark interactive card
// ---------------------------------------------------------------------------

fn build_lark_card(payload: &LinearPayload) -> LarkMessage {
    let color = priority_color(payload.data.priority);
    let action_label = match payload.action.as_str() {
        "create" => "Created",
        "update" => "Updated",
        _ => &payload.action,
    };

    let assignee = payload
        .data
        .assignee
        .as_ref()
        .map(|a| a.name.as_str())
        .unwrap_or("Unassigned");

    let title_element = serde_json::json!({
        "tag": "div",
        "text": {
            "tag": "lark_md",
            "content": format!("**{}**", payload.data.title),
        }
    });

    let fields_element = serde_json::json!({
        "tag": "div",
        "fields": [
            {
                "is_short": true,
                "text": {
                    "tag": "lark_md",
                    "content": format!("**Status:** {}", payload.data.state.name),
                }
            },
            {
                "is_short": true,
                "text": {
                    "tag": "lark_md",
                    "content": format!("**Priority:** {}", priority_label(payload.data.priority)),
                }
            },
            {
                "is_short": true,
                "text": {
                    "tag": "lark_md",
                    "content": format!("**Assignee:** {}", assignee),
                }
            }
        ]
    });

    let action_element = serde_json::json!({
        "tag": "action",
        "actions": [
            {
                "tag": "button",
                "text": {
                    "tag": "plain_text",
                    "content": "View in Linear"
                },
                "type": "primary",
                "url": payload.url,
            }
        ]
    });

    LarkMessage {
        msg_type: "interactive",
        card: LarkCard {
            header: LarkHeader {
                template: color,
                title: LarkTitle {
                    content: format!(
                        "[Linear] {}: {}",
                        action_label, payload.data.identifier
                    ),
                    tag: "plain_text",
                },
            },
            elements: vec![title_element, fields_element, action_element],
        },
    }
}

// ---------------------------------------------------------------------------
// Webhook handler
// ---------------------------------------------------------------------------

async fn webhook_handler(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    body: Bytes,
) -> StatusCode {
    // 1. Signature verification
    let signature = match headers.get("linear-signature").and_then(|v| v.to_str().ok()) {
        Some(s) => s,
        None => {
            warn!("missing linear-signature header");
            return StatusCode::UNAUTHORIZED;
        }
    };

    if !verify_signature(&state.webhook_secret, &body, signature) {
        warn!("invalid webhook signature");
        return StatusCode::UNAUTHORIZED;
    }

    // 2. Deserialize payload
    let payload: LinearPayload = match serde_json::from_slice(&body) {
        Ok(p) => p,
        Err(e) => {
            error!("failed to parse payload: {e}");
            return StatusCode::BAD_REQUEST;
        }
    };

    // 3. Filter: only Issue create / update
    if payload.kind != "Issue" || !matches!(payload.action.as_str(), "create" | "update") {
        info!(
            "ignoring event: type={}, action={}",
            payload.kind, payload.action
        );
        return StatusCode::OK;
    }

    info!(
        "processing {} {} – {}",
        payload.action, payload.data.identifier, payload.data.title
    );

    // 4. Build & send Lark card
    let card = build_lark_card(&payload);

    match state
        .http
        .post(&state.lark_webhook_url)
        .json(&card)
        .send()
        .await
    {
        Ok(resp) => {
            let status = resp.status();
            let text = resp.text().await.unwrap_or_default();
            if status.is_success() {
                info!("lark notification sent: {text}");
            } else {
                error!("lark returned {status}: {text}");
            }
        }
        Err(e) => {
            error!("failed to send lark notification: {e}");
        }
    }

    StatusCode::OK
}

// ---------------------------------------------------------------------------
// Health-check
// ---------------------------------------------------------------------------

async fn health() -> &'static str {
    "ok"
}

// ---------------------------------------------------------------------------
// Entrypoint
// ---------------------------------------------------------------------------

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info".into()),
        )
        .init();

    let webhook_secret =
        env::var("LINEAR_WEBHOOK_SECRET").expect("LINEAR_WEBHOOK_SECRET must be set");
    let lark_webhook_url = env::var("LARK_WEBHOOK_URL").unwrap_or_else(|_| {
        warn!("LARK_WEBHOOK_URL not set – lark notifications will fail");
        String::new()
    });
    let port = env::var("PORT").unwrap_or_else(|_| "3000".into());

    let state = Arc::new(AppState {
        webhook_secret,
        lark_webhook_url,
        http: Client::new(),
    });

    let app = Router::new()
        .route("/webhook", post(webhook_handler))
        .route("/health", axum::routing::get(health))
        .with_state(state);

    let addr = format!("0.0.0.0:{port}");
    info!("listening on {addr}");

    let listener = tokio::net::TcpListener::bind(&addr)
        .await
        .expect("failed to bind");

    axum::serve(listener, app).await.expect("server error");
}
