use std::{env, process::exit};
use std::fs::write;

fn main() {
    let github_output_file = env::var("GITHUB_OUTPUT").unwrap();
    let github_token = env::var("GITHUB_TOKEN").ok();
    eprintln!("GITHUB_TOKEN: {:?}", &github_token);
    
    let github_api_url = env::var("GITHUB_API_URL").ok();    
    println!("GITHUB_API_URL: {:?}", github_api_url);

    let version_increment_strategy = env::var("INPUT_VERSION-INCREMENT-STRATEGY").ok();    
    println!("INPUT_VERSION-INCREMENT-STRATEGY: {:?}", version_increment_strategy);

    let _github_token = match github_token {
      Some(value) => value,
      None => {
        eprintln!("GITHUB_TOKEN is empty. Exiting.");
        exit(1);
      },
    };
      
    let version = "0.1.0";
    let output_text = format!("semver={version}");
    eprintln!("Writing: {}", output_text);
    write(github_output_file, output_text).unwrap();
}
