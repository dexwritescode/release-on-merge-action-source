use std::fs::write;
use std::{env, process::exit};

fn main() {
    let github_output_path = env::var("GITHUB_OUTPUT").unwrap();
    eprintln!("GITHUB_OUTPUT: {:?}", &github_output_path);

    let vis_env = env::var("INPUT_VERSION-INCREMENT-STRATEGY").ok();
    if vis_env.is_some() {
        eprintln!("INPUT_VERSION-INCREMENT-STRATEGY: {:?}", vis_env);
    }

    let version = "0.1.0";
    let output_text = format!("semver={version}");
    eprintln!("Writing: {}", output_text);
    write(&github_output_path, output_text).unwrap();

    let args: Vec<String> = env::args().collect();
    let version_increment_strategy = &args[1];

    if !version_increment_strategy.is_empty() {
        eprintln!("Version Increment Strategy: {version_increment_strategy}");
        write(
            github_output_path,
            format!("version_increment_strategy={version_increment_strategy}"),
        )
        .unwrap();
    }
}
