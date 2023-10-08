use octocrab::Octocrab;
use std::{env, process::exit};

pub mod writer;

const _GITHUB_API_VERSION_HEADER: &str = "X-GitHub-Api-Version";
const _GITHUB_API_VERSION_VALUE: &str = "2022-11-28";

#[tokio::main]
async fn main() -> octocrab::Result<()> {
    let github_output_path = env::var("GITHUB_OUTPUT").unwrap();
    eprintln!("GITHUB_OUTPUT: {:?}", &github_output_path);
    let w = writer::Writer::new(github_output_path);

    let version_increment_strategy = match env::var("INPUT_VERSION-INCREMENT-STRATEGY") {
        Ok(value) => value,
        Err(_) => {
            eprintln!("INPUT_VERSION-INCREMENT-STRATEGY should be set");
            exit(1);
        }
    };
    eprintln!(
        "INPUT_VERSION-INCREMENT-STRATEGY: {:?}",
        version_increment_strategy
    );

    let github_token = match env::var("INPUT_GITHUB-TOKEN") {
        Ok(value) => value,
        Err(_) => {
            eprintln!("GITHUB_TOKEN is empty!");
            exit(1);
        }
    };

    let version = "0.1.0";
    let output_text = format!("semver={version}");
    eprintln!("Writing: {}", output_text);
    w.write("semver", version);

    let octocrab = Octocrab::builder().personal_token(github_token).build()?;

    let octocrab_release_version = octocrab
        .repos("XAMPPRocky", "octocrab")
        .releases()
        .get_latest()
        .await
        // Before mapping to '0.1.0' check if the release count is 0 (Zero)
        .map_or_else(|e| "0.1.0".to_string(), |r| r.tag_name);

    eprintln!("Release {:?}", octocrab_release_version);

    let roma_release_version = octocrab
        .repos("dexwritescode", "release-on-merge-action")
        .releases()
        .get_latest()
        .await
        .map_or_else(|e| "0.1.0".to_string(), |r| r.tag_name);

    eprintln!("Release {:?}", roma_release_version);

    Ok(())
}
