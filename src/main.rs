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
    eprintln!("Retrieved release version {:?}", &release);
    let new_release = match release {
        Some(v) => v.increment(&config.increment_strategy),
        None => Semver::from_str(&config.default_version).unwrap(),
    };
    eprintln!("Incremented version {}", &new_release);

    let w = writer::Writer::new(&config.github_output_path);
    w.write("semver", &new_release.to_string());

    Ok(())
}
