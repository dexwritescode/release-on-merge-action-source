use crate::error::ActionError;
use crate::github_client::models::{CreateReleaseRequest, TagName};
use crate::github_client::GithubApi;
use crate::Config;
use crate::Semver;

pub struct Releases<'config, C: GithubApi> {
    config: &'config Config,
    client: C,
}

impl<C: GithubApi> Releases<'_, C> {
    pub fn new(config: &Config, client: C) -> Releases<'_, C> {
        Releases { config, client }
    }

    pub fn get_latest_release(&self) -> Result<Option<TagName>, ActionError> {
        self.client.get_latest_release()
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
        self.client.create_release(&req)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Token;
    use crate::semver::VersionIncrementStrategy;
    use std::cell::RefCell;

    struct MockGithubApi {
        latest: Result<Option<TagName>, ActionError>,
        captured_request: RefCell<Option<CreateReleaseRequest>>,
        create_response: Result<Option<TagName>, ActionError>,
    }

    impl MockGithubApi {
        fn returning_latest(tag: &str) -> Self {
            MockGithubApi {
                latest: Ok(Some(TagName { tag_name: tag.to_string() })),
                captured_request: RefCell::new(None),
                create_response: Ok(Some(TagName { tag_name: tag.to_string() })),
            }
        }

        fn with_no_release() -> Self {
            MockGithubApi {
                latest: Ok(None),
                captured_request: RefCell::new(None),
                create_response: Ok(Some(TagName { tag_name: "v0.1.0".to_string() })),
            }
        }

        fn failing_with(err: ActionError) -> Self {
            MockGithubApi {
                latest: Err(err.clone()),
                captured_request: RefCell::new(None),
                create_response: Err(err),
            }
        }
    }

    impl GithubApi for MockGithubApi {
        fn get_latest_release(&self) -> Result<Option<TagName>, ActionError> {
            self.latest.clone()
        }

        fn create_release(&self, request: &CreateReleaseRequest) -> Result<Option<TagName>, ActionError> {
            *self.captured_request.borrow_mut() = Some(request.clone());
            self.create_response.clone()
        }
    }

    fn test_config() -> Config {
        Config {
            github_output_path: "/tmp/output".to_string(),
            github_token: Token("test-token".to_string()),
            github_host: "https://api.github.com".to_string(),
            increment_strategy: VersionIncrementStrategy::Patch,
            default_version: "0.1.0".to_string(),
            tag_prefix: "v".to_string(),
            repo: "test-repo".to_string(),
            owner: "test-owner".to_string(),
            commitish: "abc123".to_string(),
            body: "release body".to_string(),
            generate_release_notes: true,
            dry_run: false,
        }
    }

    #[test]
    fn get_latest_release_returns_tag_from_client() {
        let config = test_config();
        let releases = Releases::new(&config, MockGithubApi::returning_latest("v1.2.3"));
        let result = releases.get_latest_release().unwrap();
        assert_eq!(result.unwrap().tag_name, "v1.2.3");
    }

    #[test]
    fn get_latest_release_returns_none_when_no_release() {
        let config = test_config();
        let releases = Releases::new(&config, MockGithubApi::with_no_release());
        let result = releases.get_latest_release().unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn get_latest_release_propagates_error() {
        let config = test_config();
        let releases = Releases::new(&config, MockGithubApi::failing_with(ActionError::Unauthorized));
        assert_eq!(releases.get_latest_release().unwrap_err(), ActionError::Unauthorized);
    }

    #[test]
    fn create_release_builds_request_from_config_and_tag() {
        let config = test_config();
        let mock = MockGithubApi::returning_latest("v1.0.0");
        let releases = Releases::new(&config, mock);
        let tag = Semver::new(1, 2, 3, "v");

        releases.create_release(&tag).unwrap();

        let captured = releases.client.captured_request.borrow();
        let req = captured.as_ref().unwrap();
        assert_eq!(req.tag_name, "v1.2.3");
        assert_eq!(req.name, "v1.2.3");
        assert_eq!(req.target_commitish, "abc123");
        assert_eq!(req.body, "release body");
        assert!(req.generate_release_notes);
        assert!(!req.draft);
        assert!(!req.prerelease);
    }

    #[test]
    fn create_release_propagates_error() {
        let config = test_config();
        let releases = Releases::new(&config, MockGithubApi::failing_with(ActionError::UnexpectedStatus(500)));
        let tag = Semver::new(1, 0, 0, "v");
        assert_eq!(
            releases.create_release(&tag).unwrap_err(),
            ActionError::UnexpectedStatus(500)
        );
    }
}
