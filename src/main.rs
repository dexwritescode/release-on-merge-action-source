use std::{process::exit, str::FromStr};

pub mod config;
use config::Config;
pub mod release;
use release::Release;
pub mod semver;
use semver::Semver;

use crate::semver::VersionIncrementStrategy;
pub mod writer;

#[tokio::main]
async fn main() -> octocrab::Result<()> {
    let config = Config::new();
    eprintln!("Config: {:?}", &config);

    if config.increment_strategy == VersionIncrementStrategy::NoRelease {
        eprintln!("Increment strategy NoRelease - exiting");
        exit(0);
    }

    let rel = Release::new(&config);
    let latest_tag = rel.get_latest_tag().await;
    eprintln!("Retrieved version {:?}", &latest_tag);
    let default_tag = Semver::from_str(&config.get_default_tag()).unwrap();
    let new_tag = latest_tag.map_or(default_tag, |v| v.increment(&config.increment_strategy));
    eprintln!("Incremented version {}", &new_tag);

    let mut w = writer::Writer::new(&config.github_output_path);
    w.write("version", &new_tag.get_version());
    w.write("tag", &new_tag.get_tag());

    Ok(())
}
