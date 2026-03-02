//! Deserialization types for GitHub webhook payloads (subset).

use serde::Deserialize;

/// Top-level `pull_request` webhook payload.
#[derive(Debug, Deserialize)]
pub struct PullRequestPayload {
    pub action: String,
    pub pull_request: PullRequest,
    pub requested_reviewer: Option<GitHubUser>,
    pub repository: Repository,
}

#[derive(Debug, Deserialize)]
pub struct PullRequest {
    pub number: u64,
    pub title: String,
    pub html_url: String,
    pub user: GitHubUser,
    pub head: GitRef,
    pub base: GitRef,
    #[serde(default)]
    pub additions: u64,
    #[serde(default)]
    pub deletions: u64,
    #[serde(default)]
    pub merged: bool,
    pub merged_by: Option<GitHubUser>,
}

#[derive(Debug, Deserialize)]
pub struct GitRef {
    #[serde(rename = "ref")]
    pub ref_name: String,
}

#[derive(Debug, Deserialize)]
pub struct GitHubUser {
    pub login: String,
}

/// Top-level `issues` webhook payload.
#[derive(Debug, Deserialize)]
pub struct IssuesPayload {
    pub action: String,
    pub issue: GitHubIssue,
    pub label: Option<Label>,
    pub repository: Repository,
}

#[derive(Debug, Deserialize)]
pub struct GitHubIssue {
    pub number: u64,
    pub title: String,
    pub html_url: String,
    pub user: GitHubUser,
}

#[derive(Debug, Deserialize)]
pub struct Label {
    pub name: String,
}

/// Top-level `push` webhook payload.
#[derive(Debug, Deserialize)]
pub struct PushPayload {
    #[serde(rename = "ref")]
    pub ref_name: String,
    #[serde(default)]
    pub commits: Vec<PushCommit>,
    pub pusher: Pusher,
    pub compare: String,
    pub repository: Repository,
}

#[derive(Debug, Deserialize)]
pub struct PushCommit {
    pub id: String,
    pub message: String,
    pub author: CommitAuthor,
}

#[derive(Debug, Deserialize)]
pub struct CommitAuthor {
    pub name: String,
}

#[derive(Debug, Deserialize)]
pub struct Pusher {
    pub name: String,
}

#[derive(Debug, Deserialize)]
pub struct Repository {
    pub full_name: String,
    pub name: String,
}

/// Top-level `workflow_run` webhook payload.
#[derive(Debug, Deserialize)]
pub struct WorkflowRunPayload {
    pub action: String,
    pub workflow_run: WorkflowRun,
    pub repository: Repository,
}

#[derive(Debug, Deserialize)]
pub struct WorkflowRun {
    pub name: String,
    pub head_branch: String,
    pub conclusion: Option<String>,
    pub html_url: String,
    pub actor: GitHubUser,
}

/// Top-level `secret_scanning_alert` webhook payload.
#[derive(Debug, Deserialize)]
pub struct SecretScanningPayload {
    pub action: String,
    pub alert: SecretScanningAlertData,
    pub repository: Repository,
}

#[derive(Debug, Deserialize)]
pub struct SecretScanningAlertData {
    pub secret_type_display_name: Option<String>,
    pub secret_type: String,
    pub html_url: String,
}

/// Top-level `dependabot_alert` webhook payload.
#[derive(Debug, Deserialize)]
pub struct DependabotPayload {
    pub action: String,
    pub alert: DependabotAlertData,
    pub repository: Repository,
}

#[derive(Debug, Deserialize)]
pub struct DependabotAlertData {
    pub severity: String,
    pub html_url: String,
    pub security_advisory: Option<SecurityAdvisory>,
    pub dependency: Option<DependabotDependency>,
}

#[derive(Debug, Deserialize)]
pub struct SecurityAdvisory {
    pub summary: String,
}

#[derive(Debug, Deserialize)]
pub struct DependabotDependency {
    pub package: Option<DependabotPackage>,
}

#[derive(Debug, Deserialize)]
pub struct DependabotPackage {
    pub name: String,
}
