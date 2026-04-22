use self::models::{CreateReleaseRequest, PullRequest, TagName};
use crate::error::ActionError;
use crate::Config;

use reqwest::blocking::Client;
use reqwest::header;
use reqwest::StatusCode;

pub mod models;

pub trait GithubApi {
    fn get_latest_release(&self) -> Result<Option<TagName>, ActionError>;
    fn create_release(&self, request: &CreateReleaseRequest) -> Result<Option<TagName>, ActionError>;
    fn get_pr_for_commit(&self, sha: &str) -> Result<Option<PullRequest>, ActionError>;
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
}
