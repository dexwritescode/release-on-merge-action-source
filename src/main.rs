use std::{process::exit, str::FromStr};

pub mod config;
use config::Config;
pub mod github_client;
use github_client::GithubClient;
pub mod semver;
use semver::Semver;
pub mod releases;
use releases::Releases;

use crate::semver::VersionIncrementStrategy;
pub mod writer;

fn main() {
    let config = Config::new();
    eprintln!("Config: {:?}", &config);

    if config.increment_strategy == VersionIncrementStrategy::NoRelease {
        eprintln!("Increment strategy NoRelease - exiting");
        exit(0);
    }

    let github_client = GithubClient::new(&config);
    let releases = Releases::new(&config, github_client);
    let latest_release = releases.get_latest_release();
    eprintln!("Retrieved release {:?}", &latest_release);
    let default_tag = Semver::from_str(&config.get_default_tag()).unwrap();

    let new_tag = latest_release.map_or(default_tag, |v| {
        Semver::from_str(&v.tag_name)
            .unwrap()
            .increment(&config.increment_strategy)
    });
    eprintln!("Incremented version {}", &new_tag);

    if config.dry_run {
        eprintln!("Dry run mode active. Not creating a new release.");
    } else {
        eprintln!("Creating new release.");
        let release = releases.create_release(&new_tag);
        eprintln!("New release created {:?}", &release);
    };

    let mut w = writer::Writer::new(&config.github_output_path);
    w.write("version", &new_tag.get_version());
    w.write("tag", &new_tag.get_tag());
}
