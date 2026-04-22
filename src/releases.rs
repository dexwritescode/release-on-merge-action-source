use crate::error::ActionError;
use crate::github_client::models::{CreateReleaseRequest, PullRequest, TagName};
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
            prerelease: self.config.prerelease,
            generate_release_notes: self.config.generate_release_notes,
        };
        self.client.create_release(&req)
    }

    pub fn get_pr_for_commit(&self, sha: &str) -> Result<Option<PullRequest>, ActionError> {
        self.client.get_pr_for_commit(sha)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Token;
    use crate::github_client::models::Label;
    use crate::semver::VersionIncrementStrategy;
    use std::cell::RefCell;

    struct MockGithubApi {
        latest: Result<Option<TagName>, ActionError>,
        captured_request: RefCell<Option<CreateReleaseRequest>>,
        create_response: Result<Option<TagName>, ActionError>,
        pr_response: Result<Option<PullRequest>, ActionError>,
    }

    impl MockGithubApi {
        fn returning_latest(tag: &str) -> Self {
            MockGithubApi {
                latest: Ok(Some(TagName { tag_name: tag.to_string() })),
                captured_request: RefCell::new(None),
                create_response: Ok(Some(TagName { tag_name: tag.to_string() })),
                pr_response: Ok(None),
            }
        }

        fn with_no_release() -> Self {
            MockGithubApi {
                latest: Ok(None),
                captured_request: RefCell::new(None),
                create_response: Ok(Some(TagName { tag_name: "v0.1.0".to_string() })),
                pr_response: Ok(None),
            }
        }

        fn failing_with(err: ActionError) -> Self {
            MockGithubApi {
                latest: Err(err.clone()),
                captured_request: RefCell::new(None),
                create_response: Err(err),
                pr_response: Ok(None),
            }
        }

        fn with_pr_labels(mut self, labels: &[&str]) -> Self {
            self.pr_response = Ok(Some(PullRequest {
                labels: labels.iter().map(|n| Label { name: n.to_string() }).collect(),
            }));
            self
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

        fn get_pr_for_commit(&self, _sha: &str) -> Result<Option<PullRequest>, ActionError> {
            self.pr_response.clone()
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
            prerelease: false,
            prerelease_identifier: "rc".to_string(),
            use_label_strategy: false,
            label_major: "release:major".to_string(),
            label_minor: "release:minor".to_string(),
            label_patch: "release:patch".to_string(),
            label_skip: "release:skip".to_string(),
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
        assert!(!req.prerelease); // config.prerelease is false in test_config()
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

    #[test]
    fn get_pr_for_commit_returns_pr_with_labels() {
        let config = test_config();
        let mock = MockGithubApi::with_no_release().with_pr_labels(&["release:minor", "bug"]);
        let releases = Releases::new(&config, mock);
        let pr = releases.get_pr_for_commit("abc123").unwrap().unwrap();
        assert_eq!(pr.labels.len(), 2);
        assert_eq!(pr.labels[0].name, "release:minor");
    }

    #[test]
    fn get_pr_for_commit_returns_none_when_no_pr() {
        let config = test_config();
        let releases = Releases::new(&config, MockGithubApi::with_no_release());
        assert!(releases.get_pr_for_commit("abc123").unwrap().is_none());
    }
}
