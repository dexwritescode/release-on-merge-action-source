use std::{process::exit, str::FromStr};

pub mod config;
use config::Config;
pub mod error;
use error::ActionError;
pub mod github_client;
use github_client::GithubClient;
pub mod semver;
use semver::Semver;
pub mod releases;
use releases::Releases;

use crate::semver::VersionIncrementStrategy;
pub mod writer;

fn run() -> Result<(), ActionError> {
    let config = Config::new()?;
    eprintln!("Config: {:?}", &config);

    if config.increment_strategy == VersionIncrementStrategy::NoRelease && !config.prerelease {
        eprintln!("NoRelease strategy — skipping release creation");
        return Ok(());
    }

    let github_client = GithubClient::new(&config);
    let releases = Releases::new(&config, github_client);

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
            Some(ref v) => v
                .base_version()
                .increment(&config.increment_strategy)
                .with_pre_release(id, 1),
        }
    } else {
        match latest_semver {
            None => default_tag,
            Some(v) => v.base_version().increment(&config.increment_strategy),
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
