use self::models::CreateReleaseRequest;
use crate::Config;

use reqwest::blocking::Client;
use reqwest::blocking::Response;
use reqwest::header;
use reqwest::Error;

pub struct GithubClient {
    client: Client,
    repo: String,
    owner: String,
    github_host: String,
}

pub mod models;

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

    // GET /repos/{owner}/{repo}/releases/latest
    pub fn get_latest_release(&self) -> Result<Response, Error> {
        self.client
            .get(format!(
                "{}/repos/{}/{}/releases/latest",
                self.github_host, self.owner, self.repo
            ))
            .send()
    }

    // POST /repos/{owner}/{repo}/releases
    pub fn create_release(&self, request: &CreateReleaseRequest) -> Result<Response, Error> {
        self.client
            .post(format!(
                "{}/repos/{}/{}/releases/latest",
                self.github_host, self.owner, self.repo
            ))
            .json(request)
            .send()
    }
}
