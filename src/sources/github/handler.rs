//! Axum handler for `POST /github/webhook` — receives GitHub webhook payloads,
//! converts them to [`Event`]s, and dispatches immediately (no debounce).

use std::sync::Arc;

use axum::{
    body::Bytes,
    extract::State,
    http::{HeaderMap, StatusCode},
};
use tracing::{info, warn};

use crate::{
    config::{AppState, GitHubConfig},
    dispatch,
    event::{CommitSummary, Event},
};

use super::{
    models::{
        DependabotPayload, IssuesPayload, PullRequestPayload, PushPayload, SecretScanningPayload,
        WorkflowRunPayload,
    },
    utils::{branch_from_ref, verify_github_signature},
};

const MAX_COMMITS: usize = 5;

/// Minimal struct for lightweight repo-name extraction before full deserialization.
#[derive(serde::Deserialize)]
struct RepoProbe {
    repository: RepoName,
}

#[derive(serde::Deserialize)]
struct RepoName {
    name: String,
}

/// Handles incoming GitHub webhook requests.
///
/// 1. Verifies the `X-Hub-Signature-256` HMAC header.
/// 2. Routes by the `X-GitHub-Event` header.
/// 3. Converts to an [`Event`] and dispatches immediately.
pub async fn webhook_handler(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    body: Bytes,
) -> StatusCode {
    let github = match &state.github {
        Some(cfg) => cfg,
        None => {
            warn!("received GitHub webhook but GITHUB_WEBHOOK_SECRET not configured");
            return StatusCode::NOT_FOUND;
        }
    };

    let signature = match headers
        .get("x-hub-signature-256")
        .and_then(|v| v.to_str().ok())
    {
        Some(s) => s,
        None => {
            warn!("missing x-hub-signature-256 header");
            return StatusCode::UNAUTHORIZED;
        }
    };

    if !verify_github_signature(&github.webhook_secret, &body, signature) {
        warn!("invalid GitHub webhook signature");
        return StatusCode::UNAUTHORIZED;
    }

    // Repo whitelist filter — skip events from repos not on the list.
    if !github.repo_whitelist.is_empty() {
        match serde_json::from_slice::<RepoProbe>(&body) {
            Ok(probe) => {
                if !github.repo_whitelist.contains(&probe.repository.name) {
                    info!(
                        "ignoring event from non-whitelisted repo: {}",
                        probe.repository.name
                    );
                    return StatusCode::OK;
                }
            }
            Err(_) => {
                warn!("could not extract repository name for whitelist check");
            }
        }
    }

    let event_type = headers
        .get("x-github-event")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");

    match event_type {
        "pull_request" => handle_pull_request(&state, &body, github).await,
        "issues" => handle_issues(&state, &body, github).await,
        "push" => handle_push(&state, &body).await,
        "workflow_run" => handle_workflow_run(&state, &body).await,
        "secret_scanning_alert" => handle_secret_scanning(&state, &body).await,
        "dependabot_alert" => handle_dependabot(&state, &body).await,
        _ => {
            info!("ignoring GitHub event type: {event_type}");
            StatusCode::OK
        }
    }
}

async fn handle_pull_request(
    state: &Arc<AppState>,
    body: &[u8],
    github: &GitHubConfig,
) -> StatusCode {
    let payload: PullRequestPayload = match serde_json::from_slice(body) {
        Ok(p) => p,
        Err(e) => {
            warn!("failed to parse pull_request payload: {e}");
            return StatusCode::BAD_REQUEST;
        }
    };

    let pr = &payload.pull_request;
    let repo = &payload.repository.full_name;

    match payload.action.as_str() {
        "opened" => {
            info!("GitHub PR opened: {repo}#{}", pr.number);
            let event = Event::PrOpened {
                repo: repo.clone(),
                number: pr.number,
                title: pr.title.clone(),
                author: pr.user.login.clone(),
                head_branch: pr.head.ref_name.clone(),
                base_branch: pr.base.ref_name.clone(),
                additions: pr.additions,
                deletions: pr.deletions,
                url: pr.html_url.clone(),
            };
            dispatch::dispatch(&event, state, None).await;
            StatusCode::OK
        }
        "review_requested" => {
            let reviewer = match &payload.requested_reviewer {
                Some(u) => &u.login,
                None => {
                    info!("review_requested without requested_reviewer, ignoring");
                    return StatusCode::OK;
                }
            };

            info!(
                "GitHub review requested: {repo}#{} reviewer={reviewer}",
                pr.number
            );

            let reviewer_lark_id = github.user_map.get(reviewer).cloned();
            let dm_email = reviewer_lark_id.clone();

            let event = Event::PrReviewRequested {
                repo: repo.clone(),
                number: pr.number,
                title: pr.title.clone(),
                author: pr.user.login.clone(),
                reviewer: reviewer.clone(),
                reviewer_lark_id,
                url: pr.html_url.clone(),
            };
            dispatch::dispatch(&event, state, dm_email.as_deref()).await;
            StatusCode::OK
        }
        "closed" if pr.merged => {
            let merged_by = pr
                .merged_by
                .as_ref()
                .map(|u| u.login.clone())
                .unwrap_or_else(|| pr.user.login.clone());

            info!("GitHub PR merged: {repo}#{} by {merged_by}", pr.number);

            let event = Event::PrMerged {
                repo: repo.clone(),
                number: pr.number,
                title: pr.title.clone(),
                author: pr.user.login.clone(),
                merged_by,
                url: pr.html_url.clone(),
            };
            dispatch::dispatch(&event, state, None).await;
            StatusCode::OK
        }
        _ => {
            info!(
                "ignoring pull_request action: {} for {repo}#{}",
                payload.action, pr.number
            );
            StatusCode::OK
        }
    }
}

async fn handle_issues(state: &Arc<AppState>, body: &[u8], github: &GitHubConfig) -> StatusCode {
    let payload: IssuesPayload = match serde_json::from_slice(body) {
        Ok(p) => p,
        Err(e) => {
            warn!("failed to parse issues payload: {e}");
            return StatusCode::BAD_REQUEST;
        }
    };

    if payload.action != "labeled" {
        info!("ignoring issues action: {}", payload.action);
        return StatusCode::OK;
    }

    let label = match &payload.label {
        Some(l) => &l.name,
        None => return StatusCode::OK,
    };

    if !github.alert_labels.contains(&label.to_lowercase()) {
        info!("ignoring non-alert label: {label}");
        return StatusCode::OK;
    }

    let repo = &payload.repository.full_name;
    let issue = &payload.issue;

    info!(
        "GitHub issue labeled alert: {repo}#{} label={label}",
        issue.number
    );

    let event = Event::IssueLabeledAlert {
        repo: repo.clone(),
        number: issue.number,
        title: issue.title.clone(),
        label: label.clone(),
        author: issue.user.login.clone(),
        url: issue.html_url.clone(),
    };
    dispatch::dispatch(&event, state, None).await;
    StatusCode::OK
}

async fn handle_push(state: &Arc<AppState>, body: &[u8]) -> StatusCode {
    let payload: PushPayload = match serde_json::from_slice(body) {
        Ok(p) => p,
        Err(e) => {
            warn!("failed to parse push payload: {e}");
            return StatusCode::BAD_REQUEST;
        }
    };

    let branch = branch_from_ref(&payload.ref_name);

    if !is_protected_branch(branch) {
        info!("ignoring push to non-protected branch: {branch}");
        return StatusCode::OK;
    }

    let repo = &payload.repository.full_name;
    info!(
        "GitHub push to {repo}@{branch}: {} commit(s)",
        payload.commits.len()
    );

    let commits: Vec<CommitSummary> = payload
        .commits
        .iter()
        .take(MAX_COMMITS)
        .map(|c| CommitSummary {
            sha_short: c.id.chars().take(7).collect(),
            message_line: c.message.lines().next().unwrap_or("").to_string(),
            author: c.author.name.clone(),
        })
        .collect();

    let event = Event::BranchPush {
        repo: repo.clone(),
        branch: branch.to_string(),
        pusher: payload.pusher.name.clone(),
        commits,
        compare_url: payload.compare.clone(),
    };
    dispatch::dispatch(&event, state, None).await;
    StatusCode::OK
}

fn is_protected_branch(branch: &str) -> bool {
    matches!(branch, "main" | "master") || branch.starts_with("release")
}

async fn handle_workflow_run(state: &Arc<AppState>, body: &[u8]) -> StatusCode {
    let payload: WorkflowRunPayload = match serde_json::from_slice(body) {
        Ok(p) => p,
        Err(e) => {
            warn!("failed to parse workflow_run payload: {e}");
            return StatusCode::BAD_REQUEST;
        }
    };

    if payload.action != "completed" {
        info!("ignoring workflow_run action: {}", payload.action);
        return StatusCode::OK;
    }

    let run = &payload.workflow_run;
    let conclusion = run.conclusion.as_deref().unwrap_or("unknown");

    if conclusion != "failure" {
        info!("ignoring workflow_run with conclusion: {conclusion}");
        return StatusCode::OK;
    }

    let repo = &payload.repository.full_name;
    info!(
        "GitHub workflow_run failed: {repo} workflow={} branch={}",
        run.name, run.head_branch
    );

    let event = Event::WorkflowRunFailed {
        repo: repo.clone(),
        workflow_name: run.name.clone(),
        branch: run.head_branch.clone(),
        actor: run.actor.login.clone(),
        conclusion: conclusion.to_string(),
        url: run.html_url.clone(),
    };
    dispatch::dispatch(&event, state, None).await;
    StatusCode::OK
}

async fn handle_secret_scanning(state: &Arc<AppState>, body: &[u8]) -> StatusCode {
    let payload: SecretScanningPayload = match serde_json::from_slice(body) {
        Ok(p) => p,
        Err(e) => {
            warn!("failed to parse secret_scanning_alert payload: {e}");
            return StatusCode::BAD_REQUEST;
        }
    };

    if payload.action != "created" {
        info!("ignoring secret_scanning_alert action: {}", payload.action);
        return StatusCode::OK;
    }

    let repo = &payload.repository.full_name;
    let alert = &payload.alert;
    let display_type = alert
        .secret_type_display_name
        .as_deref()
        .unwrap_or(&alert.secret_type);

    info!("GitHub secret scanning alert: {repo} type={display_type}");

    let event = Event::SecretScanningAlert {
        repo: repo.clone(),
        secret_type: display_type.to_string(),
        url: alert.html_url.clone(),
    };
    dispatch::dispatch(&event, state, None).await;
    StatusCode::OK
}

async fn handle_dependabot(state: &Arc<AppState>, body: &[u8]) -> StatusCode {
    let payload: DependabotPayload = match serde_json::from_slice(body) {
        Ok(p) => p,
        Err(e) => {
            warn!("failed to parse dependabot_alert payload: {e}");
            return StatusCode::BAD_REQUEST;
        }
    };

    if payload.action != "created" {
        info!("ignoring dependabot_alert action: {}", payload.action);
        return StatusCode::OK;
    }

    let alert = &payload.alert;
    let severity = alert.severity.to_lowercase();

    if severity != "critical" && severity != "high" {
        info!("ignoring dependabot_alert with severity: {severity}");
        return StatusCode::OK;
    }

    let repo = &payload.repository.full_name;
    let package = alert
        .dependency
        .as_ref()
        .and_then(|d| d.package.as_ref())
        .map(|p| p.name.as_str())
        .unwrap_or("unknown");
    let summary = alert
        .security_advisory
        .as_ref()
        .map(|a| a.summary.as_str())
        .unwrap_or("No summary available");

    info!("GitHub dependabot alert: {repo} pkg={package} severity={severity}");

    let event = Event::DependabotAlert {
        repo: repo.clone(),
        package: package.to_string(),
        severity,
        summary: summary.to_string(),
        url: alert.html_url.clone(),
    };
    dispatch::dispatch(&event, state, None).await;
    StatusCode::OK
}
