use octocrab::{Error, Octocrab};
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

    let github_client = Octocrab::builder().personal_token(github_token).build()?;

    let roma_version =
        get_release_version(&github_client, "dexwritescode", "release-on-merge-action").await;
    eprintln!("Roma version to increment {}", &roma_version);

    let octocrab_version = get_release_version(&github_client, "XAMPPRocky", "octocrab").await;

    eprintln!("Octocrab version to bump {}", octocrab_version);

    Ok(())
}

async fn get_release_version(github_client: &Octocrab, owner: &str, repo: &str) -> String {
    github_client
        .repos(owner, repo)
        .releases()
        .get_latest()
        .await
        .map_or_else(
            |e| match e {
                Error::GitHub { ref source, .. } => {
                    if source.message.eq_ignore_ascii_case("Not Found") && source.errors.is_none() {
                        get_default_version()
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

fn get_default_version() -> String {
    let default_version = "v0.1.0".to_string();
    match env::var("INPUT_DEFAULT-VERSION") {
        Ok(value) => value,
        Err(_) => {
            eprintln!("inputs.default-version not set. Using {}", default_version);
            default_version
        }
    }
}
