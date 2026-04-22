use std::{env, fmt, str::FromStr};

use crate::error::ActionError;
use crate::semver::VersionIncrementStrategy;

const INITIAL_VERSION: &str = "INPUT_INITIAL-VERSION";
const GITHUB_REPOSITORY: &str = "GITHUB_REPOSITORY";
const GITHUB_OUTPUT: &str = "GITHUB_OUTPUT";
const GITHUB_TOKEN: &str = "GITHUB_TOKEN";
const TAG_PREFIX: &str = "INPUT_TAG-PREFIX";
const GITHUB_HOST: &str = "INPUT_GITHUB-HOST";
const COMMITISH: &str = "GITHUB_SHA";
const BODY: &str = "INPUT_BODY";
const GENERATE_RELEASE_NOTES: &str = "INPUT_GENERATE-RELEASE-NOTES";
const DRY_RUN: &str = "INPUT_DRY-RUN";
const INCREMENT_STRATEGY: &str = "INPUT_VERSION-INCREMENT-STRATEGY";

#[derive(Debug)]
pub struct Config {
    pub github_output_path: String,
    pub github_token: Token,
    pub github_host: String,
    pub increment_strategy: VersionIncrementStrategy,
    pub default_version: String,
    pub tag_prefix: String,
    pub repo: String,
    pub owner: String,
    pub commitish: String,
    pub body: String,
    pub generate_release_notes: bool,
    pub dry_run: bool,
}

pub struct Token(pub String);

impl fmt::Debug for Token {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "***")
    }
}

impl Config {
    pub fn new() -> Result<Config, ActionError> {
        let (owner, repo) = get_repo_info()?;
        Ok(Config {
            github_output_path: get_github_output_path()?,
            github_token: Token(get_github_token()?),
            github_host: get_github_host(),
            increment_strategy: get_version_increment_strategy()?,
            default_version: get_default_version(),
            tag_prefix: get_tag_prefix(),
            repo,
            owner,
            commitish: get_commitish()?,
            body: get_body(),
            generate_release_notes: get_generate_release_notes()?,
            dry_run: is_dry_run()?,
        })
    }

    pub fn get_default_tag(&self) -> String {
        format!("{}{}", self.tag_prefix, self.default_version)
    }
}

fn require_env(var: &'static str) -> Result<String, ActionError> {
    env::var(var).map_err(|_| ActionError::MissingEnv(var))
}

fn get_github_token() -> Result<String, ActionError> {
    require_env(GITHUB_TOKEN)
}

fn get_github_output_path() -> Result<String, ActionError> {
    require_env(GITHUB_OUTPUT)
}

fn get_version_increment_strategy() -> Result<VersionIncrementStrategy, ActionError> {
    let value = require_env(INCREMENT_STRATEGY)?;
    VersionIncrementStrategy::from_str(&value)
        .map_err(|_| ActionError::InvalidStrategy(value))
}

fn get_default_version() -> String {
    env::var(INITIAL_VERSION).unwrap_or_else(|_| "0.1.0".to_string())
}

fn get_repo_info() -> Result<(String, String), ActionError> {
    let v = require_env(GITHUB_REPOSITORY)?;
    let mut parts = v.splitn(2, '/');
    let owner = parts.next().unwrap_or("").to_owned();
    let repo = parts.next().unwrap_or("").to_owned();
    Ok((owner, repo))
}

fn get_tag_prefix() -> String {
    env::var(TAG_PREFIX).unwrap_or_else(|_| "v".to_string())
}

fn get_github_host() -> String {
    env::var(GITHUB_HOST).unwrap_or_else(|_| "https://api.github.com".to_string())
}

fn get_commitish() -> Result<String, ActionError> {
    require_env(COMMITISH)
}

fn get_body() -> String {
    env::var(BODY).unwrap_or_default()
}

fn get_generate_release_notes() -> Result<bool, ActionError> {
    let v = require_env(GENERATE_RELEASE_NOTES)?;
    Ok(matches!(v.to_ascii_lowercase().as_str(), "true"))
}

fn is_dry_run() -> Result<bool, ActionError> {
    let v = require_env(DRY_RUN)?;
    Ok(matches!(v.to_ascii_lowercase().as_str(), "true"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;

    static ENV_LOCK: Mutex<()> = Mutex::new(());

    fn set_required_env() {
        std::env::set_var("GITHUB_REPOSITORY", "owner/repo");
        std::env::set_var("GITHUB_OUTPUT", "/tmp/output");
        std::env::set_var("GITHUB_TOKEN", "test-token");
        std::env::set_var("GITHUB_SHA", "abc123");
        std::env::set_var("INPUT_VERSION-INCREMENT-STRATEGY", "patch");
        std::env::set_var("INPUT_GENERATE-RELEASE-NOTES", "true");
        std::env::set_var("INPUT_DRY-RUN", "false");
    }

    fn clear_env() {
        for var in &[
            "GITHUB_REPOSITORY",
            "GITHUB_OUTPUT",
            "GITHUB_TOKEN",
            "GITHUB_SHA",
            "INPUT_VERSION-INCREMENT-STRATEGY",
            "INPUT_GENERATE-RELEASE-NOTES",
            "INPUT_DRY-RUN",
            "INPUT_INITIAL-VERSION",
            "INPUT_TAG-PREFIX",
            "INPUT_BODY",
            "INPUT_GITHUB-HOST",
        ] {
            std::env::remove_var(var);
        }
    }

    #[test]
    fn config_new_parses_required_fields() {
        let _lock = ENV_LOCK.lock().unwrap();
        set_required_env();
        let config = Config::new().unwrap();
        assert_eq!(config.owner, "owner");
        assert_eq!(config.repo, "repo");
        assert_eq!(config.commitish, "abc123");
        assert_eq!(config.github_output_path, "/tmp/output");
        assert_eq!(config.tag_prefix, "v");
        assert_eq!(config.default_version, "0.1.0");
        assert_eq!(config.github_host, "https://api.github.com");
        assert!(!config.dry_run);
        assert!(config.generate_release_notes);
        clear_env();
    }

    #[test]
    fn config_missing_token_returns_error() {
        let _lock = ENV_LOCK.lock().unwrap();
        set_required_env();
        std::env::remove_var("GITHUB_TOKEN");
        let err = Config::new().unwrap_err();
        assert_eq!(err, ActionError::MissingEnv("GITHUB_TOKEN"));
        clear_env();
    }

    #[test]
    fn config_missing_repository_returns_error() {
        let _lock = ENV_LOCK.lock().unwrap();
        set_required_env();
        std::env::remove_var("GITHUB_REPOSITORY");
        let err = Config::new().unwrap_err();
        assert_eq!(err, ActionError::MissingEnv("GITHUB_REPOSITORY"));
        clear_env();
    }

    #[test]
    fn config_invalid_strategy_returns_error() {
        let _lock = ENV_LOCK.lock().unwrap();
        set_required_env();
        std::env::set_var("INPUT_VERSION-INCREMENT-STRATEGY", "bogus");
        let err = Config::new().unwrap_err();
        assert!(matches!(err, ActionError::InvalidStrategy(_)));
        clear_env();
    }

    #[test]
    fn config_custom_prefix_and_initial_version() {
        let _lock = ENV_LOCK.lock().unwrap();
        set_required_env();
        std::env::set_var("INPUT_TAG-PREFIX", "release-");
        std::env::set_var("INPUT_INITIAL-VERSION", "2.0.0");
        let config = Config::new().unwrap();
        assert_eq!(config.get_default_tag(), "release-2.0.0");
        clear_env();
    }

    #[test]
    fn config_dry_run_parsed_correctly() {
        let _lock = ENV_LOCK.lock().unwrap();
        set_required_env();
        std::env::set_var("INPUT_DRY-RUN", "true");
        let config = Config::new().unwrap();
        assert!(config.dry_run);
        clear_env();
    }
}
