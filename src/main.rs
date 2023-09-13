use std::{env, process::exit};

use octorust::{auth::Credentials, Client};

fn main() {
    let github_token = env::var("GITHUB_TOKEN").ok();
    println!("GITHUB_TOKEN: {:?}", &github_token);
    
    let github_api_url = env::var("GITHUB_API_URL").ok();    
    println!("GITHUB_API_URL: {:?}", github_api_url);

    let version_increment_strategy = env::var("INPUT_VERSION-INCREMENT-STRATEGY").ok();    
    println!("INPUT_VERSION-INCREMENT-STRATEGY: {:?}", version_increment_strategy);

    let github_token = match github_token {
      Some(value) => value,
      None => {
        println!("GITHUB_TOKEN is empty. Exiting.");
        exit(1);
      },
    };

    let _github = Client::new(
        String::from("user-agent-name"),
        Credentials::Token(
          github_token
        ),
      );
      
      env::set_var("GITHUB_OUTPUT", "semver=The version we just incremented");
}
