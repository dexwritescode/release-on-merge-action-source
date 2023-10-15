use crate::github_client::models::CreateReleaseRequest;
use crate::github_client::models::TagName;
use crate::github_client::GithubClient;
use crate::Config;
use crate::Semver;

use reqwest::StatusCode;

use std::process::exit;

pub struct Releases<'config> {
    config: &'config Config,
    client: GithubClient,
}

impl Releases<'_> {
    pub fn new(config: &Config, client: GithubClient) -> Releases {
        Releases { config, client }
    }

    pub fn get_latest_release(&self) -> Option<TagName> {
        let latest_release_reponse = self.client.get_latest_release().unwrap_or_else(|e| {
            eprintln!("Error getting the latest release {:?}", &e.to_string());
            exit(1);
        });

        let latest_release: Option<TagName> = match latest_release_reponse.status() {
            StatusCode::OK => latest_release_reponse.json().unwrap(),
            StatusCode::NOT_FOUND => None,
            StatusCode::UNAUTHORIZED => {
                eprintln!("Unauthorized. Make sure you are using a valid token.");
                exit(1);
            }
            s => {
                eprintln!("Received response status {}", s);
                exit(1);
            }
        };

        latest_release
    }

    pub fn create_release(&self, tag: &Semver) -> Option<TagName> {
        let req = CreateReleaseRequest {
            tag_name: tag.get_tag(),
            target_commitish: self.config.commitish.clone(),
            name: tag.get_tag(),
            body: self.config.body.clone(),
            draft: false,
            prerelease: false,
            generate_release_notes: self.config.generate_release_notes,
        };

        let response = self.client.create_release(&req).unwrap_or_else(|e| {
            eprintln!("Error creating release {:?}", &req);
            eprintln!("Error {}", e);
            exit(1);
        });

        let release: Option<TagName> = match response.status() {
            StatusCode::CREATED => response.json().unwrap(),
            StatusCode::UNAUTHORIZED => {
                eprintln!("Unauthorized. Make sure you are using a valid token.");
                exit(1);
            }
            s => {
                eprintln!("Received response status {}", s);
                eprintln!("{:?}", &response);
                exit(1);
            }
        };

        release
    }
}
