use crate::Config;
use crate::Semver;

use anyhow::Result;
use reqwest::blocking::Client;
use reqwest::header;
use reqwest::Error;

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

    // GET /repos/{owner}/{repo}/releases/latest
    pub fn get_latest_release(&self) -> Result<reqwest::blocking::Response, Error> {
        self.client
            .get(format!(
                "{}/repos/{}/{}/releases/latest",
                self.github_host, self.owner, self.repo
            ))
            .send()
    }

    // POST /repos/{owner}/{repo}/releases
    pub fn create_release(&self, _tag: &Semver) -> Result<reqwest::blocking::Response, Error> {
        self.client
            .post(format!(
                "{}/repos/{}/{}/releases/latest",
                self.github_host, self.owner, self.repo
            ))
            .send()
    }
    // pub async fn create_release(&self, tag: &Semver) -> Result<octocrab::models::repos::Release> {
    //     self.client
    //         .repos(&self.owner, &self.repo)
    //         .releases()
    //         .create(&tag.get_tag())
    //         //.body(body)
    //         .draft(false)
    //         .make_latest(MakeLatest::True)
    //         .name(&tag.get_tag())
    //         //.target_commitish(target_commitish)
    //         .send()
    //         .await
    // }
}
