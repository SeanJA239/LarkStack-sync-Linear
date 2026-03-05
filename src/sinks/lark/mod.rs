//! Lark (Feishu) notification sink — group webhook cards and bot DMs.

mod bot;
pub mod cards;
pub mod event_handler;
pub mod models;
pub(crate) mod webhook;

pub use bot::LarkBotClient;
pub use event_handler::lark_event_handler;

use tracing::error;

use crate::{config::AppState, event::Event};

/// Sends a card notification for `event` to the Linear Lark group.
///
/// Prefers Bot API (`target_chat_id`) when available, falls back to
/// `webhook_url`.
pub async fn notify(event: &Event, state: &AppState) {
    let card = cards::build_lark_card(event);
    deliver(
        &card,
        &state.http,
        state.lark_bot.as_ref(),
        state.lark.target_chat_id.as_deref(),
        &state.lark.webhook_url,
        "Linear",
    )
    .await;
}

/// Sends a card notification for `event` to the GitHub Lark group.
///
/// Uses `LARK_GITHUB_*` credentials/webhook when set, otherwise falls back
/// to the shared Linear Lark app and `LARK_WEBHOOK_URL`.
pub async fn notify_github(event: &Event, state: &AppState) {
    let card = cards::build_lark_card(event);
    let bot = state.github_lark_bot.as_ref().or(state.lark_bot.as_ref());
    let chat_id = state
        .lark
        .github_target_chat_id
        .as_deref()
        .or(state.lark.target_chat_id.as_deref());
    let webhook = if !state.lark.github_webhook_url.is_empty() {
        &state.lark.github_webhook_url
    } else {
        &state.lark.webhook_url
    };
    deliver(&card, &state.http, bot, chat_id, webhook, "GitHub").await;
}

async fn deliver(
    card: &models::LarkMessage,
    http: &reqwest::Client,
    bot: Option<&LarkBotClient>,
    chat_id: Option<&str>,
    webhook_url: &str,
    source: &str,
) {
    match (bot, chat_id) {
        (Some(bot), Some(chat_id)) => {
            if let Err(e) = bot.send_to_chat(chat_id, &card.card).await {
                error!("failed to send {source} card to chat {chat_id}: {e}");
            }
        }
        _ if !webhook_url.is_empty() => {
            webhook::send_lark_card(http, webhook_url, card).await;
        }
        _ => {
            error!("no Lark delivery method configured for {source} events");
        }
    }
}

/// DMs the assignee about `event` (no-op when `bot` is `None` or event
/// does not support DM notifications).
pub async fn try_dm(event: &Event, bot: &LarkBotClient, email: &str) {
    if let Some(card) = cards::build_assign_dm_card(event)
        && let Err(e) = bot.send_dm(email, &card).await
    {
        error!("failed to DM {email}: {e}");
    }
}
