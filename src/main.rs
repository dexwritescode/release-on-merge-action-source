use std::{process::exit, str::FromStr};

pub mod config;
use config::Config;
pub mod error;
use error::ActionError;
pub mod github_client;
use github_client::models::PullRequest;
use github_client::GithubClient;
pub mod semver;
use semver::{Semver, VersionIncrementStrategy};
pub mod releases;
use releases::Releases;
pub mod writer;

fn resolve_label_strategy(pr: &PullRequest, config: &Config) -> VersionIncrementStrategy {
    let names: Vec<&str> = pr.labels.iter().map(|l| l.name.as_str()).collect();
    if names.contains(&config.label_major.as_str()) {
        VersionIncrementStrategy::Major
    } else if names.contains(&config.label_minor.as_str()) {
        VersionIncrementStrategy::Minor
    } else if names.contains(&config.label_skip.as_str()) {
        VersionIncrementStrategy::NoRelease
    } else if names.contains(&config.label_patch.as_str()) {
        VersionIncrementStrategy::Patch
    } else {
        config.increment_strategy.clone()
    }
}

fn run() -> Result<(), ActionError> {
    let config = Config::new()?;
    eprintln!("Config: {:?}", &config);

    let github_client = GithubClient::new(&config);
    let releases = Releases::new(&config, github_client);

    let strategy = if config.use_label_strategy {
        let pr = releases.get_pr_for_commit(&config.commitish)?;
        eprintln!("PR for commit: {:?}", &pr);
        pr.map(|p| resolve_label_strategy(&p, &config))
            .unwrap_or_else(|| config.increment_strategy.clone())
    } else {
        config.increment_strategy.clone()
    };
    eprintln!("Effective strategy: {:?}", &strategy);

    if strategy == VersionIncrementStrategy::NoRelease && !config.prerelease {
        eprintln!("NoRelease strategy — skipping release creation");
        return Ok(());
    }

    let latest_release = releases.get_latest_release()?;
    eprintln!("Retrieved release {:?}", &latest_release);

    let default_tag = Semver::from_str(&config.get_default_tag())
        .map_err(|_| ActionError::InvalidTag(config.get_default_tag()))?;

    let latest_semver = latest_release
        .map(|v| {
            Semver::from_str(&v.tag_name)
                .map_err(|_| ActionError::InvalidTag(v.tag_name.clone()))
        })
        .transpose()?;

    let new_tag = if config.prerelease {
        let id = &config.prerelease_identifier;
        match latest_semver {
            None => default_tag.with_pre_release(id, 1),
            Some(ref v) if v.pre_release_matches(id) => v.bump_pre_release(),
            Some(ref v) => v.base_version().increment(&strategy).with_pre_release(id, 1),
        }
    } else {
        match latest_semver {
            None => default_tag,
            Some(v) => v.base_version().increment(&strategy),
        }
    };
    eprintln!("New version: {}", &new_tag);

    if config.dry_run {
        eprintln!("Dry run mode active. Not creating a new release.");
    } else {
        eprintln!("Creating new release.");
        let release = releases.create_release(&new_tag)?;
        eprintln!("New release created {:?}", &release);
    }

    let mut w = writer::Writer::new(&config.github_output_path);
    w.write("version", &new_tag.get_version());
    w.write("tag", &new_tag.to_string());

    Ok(())
}

fn main() {
    if let Err(e) = run() {
        eprintln!("Error: {e}");
        exit(1);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Token;
    use crate::github_client::models::Label;

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
            body: "".to_string(),
            generate_release_notes: true,
            dry_run: false,
            prerelease: false,
            prerelease_identifier: "rc".to_string(),
            use_label_strategy: true,
            label_major: "release:major".to_string(),
            label_minor: "release:minor".to_string(),
            label_patch: "release:patch".to_string(),
            label_skip: "release:skip".to_string(),
        }
    }

    fn pr_with(labels: &[&str]) -> PullRequest {
        PullRequest {
            labels: labels.iter().map(|n| Label { name: n.to_string() }).collect(),
        }
    }

    #[test]
    fn label_major_resolves_to_major() {
        let config = test_config();
        assert_eq!(resolve_label_strategy(&pr_with(&["release:major"]), &config), VersionIncrementStrategy::Major);
    }

    #[test]
    fn label_minor_resolves_to_minor() {
        let config = test_config();
        assert_eq!(resolve_label_strategy(&pr_with(&["release:minor"]), &config), VersionIncrementStrategy::Minor);
    }

    #[test]
    fn label_patch_resolves_to_patch() {
        let config = test_config();
        assert_eq!(resolve_label_strategy(&pr_with(&["release:patch"]), &config), VersionIncrementStrategy::Patch);
    }

    #[test]
    fn label_skip_resolves_to_norelease() {
        let config = test_config();
        assert_eq!(resolve_label_strategy(&pr_with(&["release:skip"]), &config), VersionIncrementStrategy::NoRelease);
    }

    #[test]
    fn no_matching_label_falls_back_to_config_strategy() {
        let config = test_config(); // increment_strategy = Patch
        assert_eq!(resolve_label_strategy(&pr_with(&["bug", "documentation"]), &config), VersionIncrementStrategy::Patch);
    }

    #[test]
    fn major_takes_precedence_over_minor() {
        let config = test_config();
        assert_eq!(
            resolve_label_strategy(&pr_with(&["release:minor", "release:major"]), &config),
            VersionIncrementStrategy::Major
        );
    }
}
