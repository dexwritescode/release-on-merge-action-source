use std::fs::write;
use std::{env, process::exit};

fn main() {
    let github_output_path = env::var("GITHUB_OUTPUT").unwrap();
    eprintln!("GITHUB_OUTPUT: {:?}", &github_output_path);

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

    let _github_token = match env::var("GITHUB_TOKEN") {
        Ok(value) => value,
        Err(_) => {
            eprintln!("GITHUB_TOKEN is empty!");
            exit(1);
        }
    };
    eprintln!("GITHUB_TOKEN is set");

    let version = "0.1.0";
    let output_text = format!("semver={version}");
    eprintln!("Writing: {}", output_text);
    write(github_output_path, output_text).unwrap();
}
