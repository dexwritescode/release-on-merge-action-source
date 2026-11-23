use self::models::{Comparison, CreateReleaseRequest, PullRequest, TagName};
use crate::error::ActionError;
use crate::version_files::FileEdit;
use crate::Config;

use reqwest::blocking::{Client, RequestBuilder};
use reqwest::header;
use reqwest::StatusCode;

pub mod models;

pub trait GithubApi {
    fn get_latest_release(&self) -> Result<Option<TagName>, ActionError>;
    fn create_release(&self, request: &CreateReleaseRequest) -> Result<Option<TagName>, ActionError>;
    fn get_pr_for_commit(&self, sha: &str) -> Result<Option<PullRequest>, ActionError>;
    /// The messages of the commits in `head` that aren't in `base` (at most 250).
    fn get_commit_messages(&self, base: &str, head: &str) -> Result<Vec<String>, ActionError>;
}

/// The git data calls version-files needs: read the branch, write commits, move the branch.
pub trait GitApi {
    fn get_branch_sha(&self, branch: &str) -> Result<String, ActionError>;
    fn get_commit_tree(&self, sha: &str) -> Result<String, ActionError>;
    /// Every file path in a tree, recursively.
    fn list_paths(&self, tree: &str) -> Result<Vec<String>, ActionError>;
    /// A file's text at a commit; None if it doesn't exist.
    fn get_file(&self, path: &str, commit: &str) -> Result<Option<String>, ActionError>;
    fn create_tree(&self, base: &str, edits: &[FileEdit]) -> Result<String, ActionError>;
    fn create_commit(&self, message: &str, tree: &str, parent: &str) -> Result<String, ActionError>;
    /// Fast-forward only.
    fn update_branch(&self, branch: &str, sha: &str) -> Result<(), ActionError>;
}

pub struct GithubClient {
    client: Client,
    repo: String,
    owner: String,
    github_host: String,
}

impl GithubClient {
    pub fn new(config: &Config) -> GithubClient {
        let mut headers = header::HeaderMap::new();
        headers.insert(
            reqwest::header::AUTHORIZATION,
            format!("token {}", config.github_token.0).parse().unwrap(),
        );
        headers.insert(
            reqwest::header::ACCEPT,
            "application/vnd.github+json".parse().unwrap(),
        );
        headers.insert("X-GitHub-Api-Version", "2022-11-28".parse().unwrap());

        let client = Client::builder()
            .user_agent("release-on-merge-action")
            .default_headers(headers)
            .build()
            .unwrap();

        GithubClient {
            client,
            repo: config.repo.clone(),
            owner: config.owner.clone(),
            github_host: config.github_host.clone(),
        }
    }
}

impl GithubApi for GithubClient {
    // GET /repos/{owner}/{repo}/releases/latest
    fn get_latest_release(&self) -> Result<Option<TagName>, ActionError> {
        let response = self
            .client
            .get(format!(
                "{}/repos/{}/{}/releases/latest",
                self.github_host, self.owner, self.repo
            ))
            .send()
            .map_err(|e| ActionError::ApiError(e.to_string()))?;

        match response.status() {
            StatusCode::OK => response
                .json()
                .map_err(|e| ActionError::ApiError(e.to_string())),
            StatusCode::NOT_FOUND => Ok(None),
            StatusCode::UNAUTHORIZED => Err(ActionError::Unauthorized),
            s => Err(ActionError::UnexpectedStatus(s.as_u16())),
        }
    }

    // POST /repos/{owner}/{repo}/releases
    fn create_release(&self, request: &CreateReleaseRequest) -> Result<Option<TagName>, ActionError> {
        let response = self
            .client
            .post(format!(
                "{}/repos/{}/{}/releases",
                self.github_host, self.owner, self.repo
            ))
            .json(request)
            .send()
            .map_err(|e| ActionError::ApiError(e.to_string()))?;

        match response.status() {
            StatusCode::CREATED => response
                .json()
                .map_err(|e| ActionError::ApiError(e.to_string())),
            StatusCode::UNAUTHORIZED => Err(ActionError::Unauthorized),
            s => Err(ActionError::UnexpectedStatus(s.as_u16())),
        }
    }

    // GET /repos/{owner}/{repo}/commits/{sha}/pulls
    fn get_pr_for_commit(&self, sha: &str) -> Result<Option<PullRequest>, ActionError> {
        let response = self
            .client
            .get(format!(
                "{}/repos/{}/{}/commits/{}/pulls",
                self.github_host, self.owner, self.repo, sha
            ))
            .send()
            .map_err(|e| ActionError::ApiError(e.to_string()))?;

        match response.status() {
            StatusCode::OK => {
                let prs: Vec<PullRequest> = response
                    .json()
                    .map_err(|e| ActionError::ApiError(e.to_string()))?;
                Ok(prs.into_iter().next())
            }
            StatusCode::NOT_FOUND => Ok(None),
            StatusCode::UNAUTHORIZED => Err(ActionError::Unauthorized),
            s => Err(ActionError::UnexpectedStatus(s.as_u16())),
        }
    }

    // GET /repos/{owner}/{repo}/compare/{base}...{head}
    fn get_commit_messages(&self, base: &str, head: &str) -> Result<Vec<String>, ActionError> {
        let value = self
            .json(self.client.get(self.url(&format!("compare/{base}...{head}"))))?
            .ok_or_else(|| ActionError::ApiError(format!("can't compare {base}...{head}")))?;
        let comparison: Comparison =
            serde_json::from_value(value).map_err(|e| ActionError::ApiError(e.to_string()))?;
        Ok(comparison.commits.into_iter().map(|c| c.commit.message).collect())
    }
}

impl GithubClient {
    fn url(&self, path: &str) -> String {
        format!("{}/repos/{}/{}/{}", self.github_host, self.owner, self.repo, path)
    }

    /// Send, and return the body as JSON on success; 404 is None.
    fn json(&self, request: RequestBuilder) -> Result<Option<serde_json::Value>, ActionError> {
        let response = request.send().map_err(|e| ActionError::ApiError(e.to_string()))?;
        let status = response.status();
        if status == StatusCode::NOT_FOUND {
            return Ok(None);
        }
        if status == StatusCode::UNAUTHORIZED {
            return Err(ActionError::Unauthorized);
        }
        let body = response.text().map_err(|e| ActionError::ApiError(e.to_string()))?;
        if !status.is_success() {
            return Err(ActionError::ApiError(format!("{status}: {body}")));
        }
        serde_json::from_str(&body).map(Some).map_err(|e| ActionError::ApiError(e.to_string()))
    }

    fn sha(&self, value: Option<serde_json::Value>, pointer: &str, what: &str) -> Result<String, ActionError> {
        value
            .as_ref()
            .and_then(|v| v.pointer(pointer))
            .and_then(|v| v.as_str())
            .map(str::to_owned)
            .ok_or_else(|| ActionError::ApiError(format!("{what}: not found or no sha in the response")))
    }
}

impl GitApi for GithubClient {
    // GET /repos/{owner}/{repo}/git/ref/heads/{branch}
    fn get_branch_sha(&self, branch: &str) -> Result<String, ActionError> {
        let v = self.json(self.client.get(self.url(&format!("git/ref/heads/{branch}"))))?;
        self.sha(v, "/object/sha", &format!("branch {branch}"))
    }

    // GET /repos/{owner}/{repo}/git/commits/{sha}
    fn get_commit_tree(&self, sha: &str) -> Result<String, ActionError> {
        let v = self.json(self.client.get(self.url(&format!("git/commits/{sha}"))))?;
        self.sha(v, "/tree/sha", &format!("commit {sha}"))
    }

    // GET /repos/{owner}/{repo}/git/trees/{tree}?recursive=1
    fn list_paths(&self, tree: &str) -> Result<Vec<String>, ActionError> {
        let v = self
            .json(self.client.get(self.url(&format!("git/trees/{tree}?recursive=1"))))?
            .ok_or_else(|| ActionError::ApiError(format!("tree {tree} not found")))?;
        if v["truncated"].as_bool() == Some(true) {
            return Err(ActionError::ApiError(format!("tree {tree} is too large to list")));
        }
        Ok(v["tree"]
            .as_array()
            .into_iter()
            .flatten()
            .filter(|e| e["type"] == "blob")
            .filter_map(|e| e["path"].as_str().map(str::to_owned))
            .collect())
    }

    // GET /repos/{owner}/{repo}/contents/{path}?ref={commit}, raw
    fn get_file(&self, path: &str, commit: &str) -> Result<Option<String>, ActionError> {
        let response = self
            .client
            .get(self.url(&format!("contents/{path}?ref={commit}")))
            .header(header::ACCEPT, "application/vnd.github.raw+json")
            .send()
            .map_err(|e| ActionError::ApiError(e.to_string()))?;
        match response.status() {
            StatusCode::OK => response.text().map(Some).map_err(|e| ActionError::ApiError(e.to_string())),
            StatusCode::NOT_FOUND => Ok(None),
            StatusCode::UNAUTHORIZED => Err(ActionError::Unauthorized),
            s => Err(ActionError::UnexpectedStatus(s.as_u16())),
        }
    }

    // POST /repos/{owner}/{repo}/git/trees
    fn create_tree(&self, base: &str, edits: &[FileEdit]) -> Result<String, ActionError> {
        let tree: Vec<_> = edits
            .iter()
            .map(|e| serde_json::json!({"path": e.path, "mode": "100644", "type": "blob", "content": e.content}))
            .collect();
        let body = serde_json::json!({"base_tree": base, "tree": tree});
        let v = self.json(self.client.post(self.url("git/trees")).json(&body))?;
        self.sha(v, "/sha", "new tree")
    }

    // POST /repos/{owner}/{repo}/git/commits
    fn create_commit(&self, message: &str, tree: &str, parent: &str) -> Result<String, ActionError> {
        let body = serde_json::json!({"message": message, "tree": tree, "parents": [parent]});
        let v = self.json(self.client.post(self.url("git/commits")).json(&body))?;
        self.sha(v, "/sha", "new commit")
    }

    // PATCH /repos/{owner}/{repo}/git/refs/heads/{branch}
    fn update_branch(&self, branch: &str, sha: &str) -> Result<(), ActionError> {
        let body = serde_json::json!({"sha": sha, "force": false});
        self.json(self.client.patch(self.url(&format!("git/refs/heads/{branch}"))).json(&body))?
            .map(|_| ())
            .ok_or_else(|| ActionError::ApiError(format!("branch {branch} not found")))
    }
}
