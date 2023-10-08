use octocrab::{Error, Octocrab};
use std::process::exit;

pub mod config;
use config::Config;

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

    let roma_version = get_release_version(
        &github_client,
        "dexwritescode",
        "release-on-merge-action",
        &config,
    )
    .await;
    eprintln!("Roma version to increment {}", &roma_version);

    let octocrab_version =
        get_release_version(&github_client, "XAMPPRocky", "octocrab", &config).await;

    eprintln!("Octocrab version to bump {}", octocrab_version);

    Ok(())
}

async fn get_release_version(
    github_client: &Octocrab,
    owner: &str,
    repo: &str,
    config: &Config,
) -> String {
    github_client
        .repos(owner, repo)
        .releases()
        .get_latest()
        .await
        .map_or_else(
            |e| match e {
                Error::GitHub { ref source, .. } => {
                    if source.message.eq_ignore_ascii_case("Not Found") && source.errors.is_none() {
                        config.default_version.clone()
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
            |r| r.tag_name,
        )
}
