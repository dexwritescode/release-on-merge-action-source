use std::env;
use std::fs::write;

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
    write(github_output_path, output_text).unwrap();
}
