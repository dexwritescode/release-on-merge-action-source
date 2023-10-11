use std::{env, fmt, process::exit, str::FromStr};

use crate::semver::VersionIncrementStrategy;

#[derive(Debug)]
pub struct Config {
    pub github_output_path: String,
    pub github_token: Token,
    pub increment_strategy: VersionIncrementStrategy,
    pub default_version: String,
    pub tag_prefix: String,
    pub repo: String,
    pub owner: String,
}

pub struct Token(pub String);

impl fmt::Debug for Token {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "***")
    }
}

impl Default for Config {
    fn default() -> Self {
        Self::new()
    }
}

impl Config {
    pub fn new() -> Config {
        let (owner, repo) = get_repo_info();
        Config {
            github_output_path: get_github_output_path(),
            github_token: Token(get_github_token()),
            increment_strategy: get_version_increment_strategy(),
            default_version: get_default_version(),
            tag_prefix: get_tag_prefix(),
            repo,
            owner,
        }
    }

    pub fn get_default_tag(&self) -> String {
        format!("{}{}", self.tag_prefix, self.default_version)
    }
}

fn get_github_token() -> String {
    match env::var("INPUT_GITHUB-TOKEN") {
        Ok(value) => value,
        Err(_) => {
            eprintln!("GITHUB_TOKEN is empty!");
            exit(1);
        }
    }
}

fn get_github_output_path() -> String {
    match env::var("GITHUB_OUTPUT") {
        Ok(value) => value,
        Err(_) => {
            eprintln!("Could not read GITHUB_OUTPUT");
            exit(1);
        }
    }
}

fn get_version_increment_strategy() -> VersionIncrementStrategy {
    env::var("INPUT_VERSION-INCREMENT-STRATEGY").map_or_else(
        |e| {
            eprintln!("INPUT_VERSION-INCREMENT-STRATEGY should be set {:?}", e);
            exit(1);
        },
        |value| {
            VersionIncrementStrategy::from_str(&value).map_or_else(
                |_| {
                    eprintln!("Invalid version-increment-strategy value: {}", value);
                    exit(1);
                },
                |vis| vis,
            )
        },
    )
}

fn get_default_version() -> String {
    env::var("INPUT_INITIAL-VERSION").unwrap_or("0.1.0".to_string())
}

fn get_repo_info() -> (String, String) {
    match env::var("GITHUB_REPOSITORY") {
        Ok(value) => {
            let info: Vec<&str> = value.split('/').collect();
            eprintln!("GITHUB_REPOSITORY {}", value);
            eprintln!("Repo {:?}", info);
            (info[0].to_owned(), info[1].to_owned())
        }
        Err(_) => {
            eprintln!("GITHUB_REPOSITORY is empty!");
            exit(1);
        }
    }
}

fn get_tag_prefix() -> String {
    env::var("INPUT_TAG-PREFIX").unwrap_or("v".to_string())
}
