use std::{env, process::exit};

pub mod writer;

const _GITHUB_API_VERSION_HEADER: &str = "X-GitHub-Api-Version";
const _GITHUB_API_VERSION_VALUE: &str = "2022-11-28";

fn main() {
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

    let _github_token = match env::var("INPUT_GITHUB-TOKEN") {
        Ok(value) => value,
        Err(_) => {
            eprintln!("GITHUB_TOKEN is empty!");
            exit(1);
        }
    };

    let version = "0.1.0";
    let output_text = format!("semver={version}");
    eprintln!("Writing: {}", output_text);
    w.write("semver", &version);
}
