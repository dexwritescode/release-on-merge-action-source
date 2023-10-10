use octocrab::{Error, Octocrab};
use semver::Semver;
use std::process::exit;
use std::str::FromStr;

pub mod config;
use config::Config;
pub mod semver;
pub mod writer;

#[tokio::main]
async fn main() -> octocrab::Result<()> {
    let config = Config::new();
    eprintln!("Config: {:?}", &config);

    let w = writer::Writer::new(&config.github_output_path);

    let github_client = Octocrab::builder()
        .personal_token(config.github_token.0.clone())
        .build()?;

    let release = get_release_version(&github_client, &config).await;
    eprintln!("Release version {:?}", &release);
    let new_release = match release {
        Some(v) => v.increment(&config.increment_strategy),
        None => Semver::from_str(&config.default_version).unwrap(),
    };
    eprintln!("Incremented version {}", &new_release);
    eprintln!("Writing: {:?}", new_release.to_string());
    w.write("semver", &new_release.to_string());

    Ok(())
}

async fn get_release_version(github_client: &Octocrab, config: &Config) -> Option<Semver> {
    github_client
        .repos(&config.owner, &config.repo)
        .releases()
        .get_latest()
        .await
        .map_or_else(
            |e| match e {
                Error::GitHub { ref source, .. } => {
                    if source.message.eq_ignore_ascii_case("Not Found") && source.errors.is_none() {
                        None
                    } else {
                        eprintln!("Could not get the version.");
                        eprintln!("Error: {:?}", &e);
                        exit(1);
                    }
                }
                _ => {
                    eprintln!("Could not get the version.");
                    eprintln!("Error: {:?}", &e);
                    exit(1);
                }
            },
            |r| Semver::from_str(&r.tag_name).ok(),
        )
}
