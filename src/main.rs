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
    let release = rel.get_release_version().await;
    eprintln!("Retrieved version {:?}", &release);
    let default = Semver::from_str(&config.default_version).unwrap();
    let new_release = release.map_or(default, |v| v.increment(&config.increment_strategy));
    eprintln!("Incremented version {}", &new_release);

    let w = writer::Writer::new(&config.github_output_path);
    w.write("version", &new_release.to_string());

    Ok(())
}
