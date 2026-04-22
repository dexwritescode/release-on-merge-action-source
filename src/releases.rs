use crate::error::ActionError;
use crate::github_client::models::CreateReleaseRequest;
use crate::github_client::models::TagName;
use crate::github_client::GithubClient;
use crate::Config;
use crate::Semver;

use reqwest::StatusCode;

pub struct Releases<'config> {
    config: &'config Config,
    client: GithubClient,
}

impl Releases<'_> {
    pub fn new(config: &Config, client: GithubClient) -> Releases<'_> {
        Releases { config, client }
    }

    pub fn get_latest_release(&self) -> Result<Option<TagName>, ActionError> {
        let response = self
            .client
            .get_latest_release()
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

    pub fn create_release(&self, tag: &Semver) -> Result<Option<TagName>, ActionError> {
        let req = CreateReleaseRequest {
            tag_name: tag.to_string(),
            target_commitish: self.config.commitish.clone(),
            name: tag.to_string(),
            body: self.config.body.clone(),
            draft: false,
            prerelease: false,
            generate_release_notes: self.config.generate_release_notes,
        };

        let response = self
            .client
            .create_release(&req)
            .map_err(|e| ActionError::ApiError(e.to_string()))?;

        match response.status() {
            StatusCode::CREATED => response
                .json()
                .map_err(|e| ActionError::ApiError(e.to_string())),
            StatusCode::UNAUTHORIZED => Err(ActionError::Unauthorized),
            s => Err(ActionError::UnexpectedStatus(s.as_u16())),
        }
    }
}
