use std::str::FromStr;

pub mod config;
use config::Config;
pub mod release;
use release::Release;
pub mod semver;
use semver::Semver;
pub mod writer;

#[tokio::main]
async fn main() -> octocrab::Result<()> {
    let config = Config::new();
    eprintln!("Config: {:?}", &config);

    let rel = Release::new(&config);
    let latest_tag = rel.get_latest_tag().await;
    eprintln!("Retrieved version {:?}", &latest_tag);
    let default_tag = Semver::from_str(&config.get_default_tag()).unwrap();
    let new_tag = latest_tag.map_or(default_tag, |v| v.increment(&config.increment_strategy));
    eprintln!("Incremented version {}", &new_tag);

    let w = writer::Writer::new(&config.github_output_path);
    w.write("version", &new_tag.get_version());
    w.write("tag", &new_tag.get_tag());

    Ok(())
}
