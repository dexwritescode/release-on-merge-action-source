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

    let version = "0.1.0";
    let output_text = format!("semver={version}");
    eprintln!("Writing: {}", output_text);
    w.write("semver", version);

    let github_client = Octocrab::builder()
        .personal_token(config.github_token.0.clone())
        .build()?;

    let version = get_release_version(&github_client, &config.owner, &config.repo, &config).await;
    eprintln!("Release version {}", &version);
    eprintln!(
        "Incremented version {}",
        &version.increment(&config.increment_strategy)
    );

    Ok(())
}

async fn get_release_version(
    github_client: &Octocrab,
    owner: &str,
    repo: &str,
    config: &Config,
) -> Semver {
    github_client
        .repos(owner, repo)
        .releases()
        .get_latest()
        .await
        .map_or_else(
            |e| match e {
                Error::GitHub { ref source, .. } => {
                    if source.message.eq_ignore_ascii_case("Not Found") && source.errors.is_none() {
                        Semver::from_str(&config.default_version).unwrap()
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
            |r| Semver::from_str(&r.tag_name).unwrap(),
        )
}
