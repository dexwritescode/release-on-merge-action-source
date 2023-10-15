use crate::github_client::GithubClient;
use crate::Config;
use crate::Semver;

use anyhow::Result;
use reqwest::StatusCode;
use serde::Deserialize;

use std::process::exit;

#[derive(Deserialize, Debug)]
pub struct LatestRelease {
    pub tag_name: String,
}

pub struct Releases<'config> {
    _config: &'config Config,
    client: GithubClient,
}

impl Releases<'_> {
    pub fn new(config: &Config, client: GithubClient) -> Releases {
        Releases {
            _config: config,
            client,
        }
    }

    // /repos/{owner}/{repo}/releases/latest
    pub fn get_latest_release(&self) -> Option<LatestRelease> {
        let latest_release_reponse = match self.client.get_latest_release() {
            Ok(v) => v,
            Err(err) => {
                eprintln!("Error getting the latest release {:?}", &err.to_string());
                exit(1);
            }
        };

        let latest_release: Option<LatestRelease> = match latest_release_reponse.status() {
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

    pub fn create_release(&self, tag: &Semver) -> Result<Semver> {
        let _result = self.client.create_release(tag);

        //    self.client
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

        Ok(Semver {
            major: 1,
            minor: 0,
            patch: 0,
            prefix: "v".to_string(),
        })
    }
}
