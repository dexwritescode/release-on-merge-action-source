use std::{env, fmt, process::exit, str::FromStr};

use crate::semver::VersionIncrementStrategy;

const INITIAL_VERSION: &str = "INPUT_INITIAL-VERSION";
const GITHUB_REPOSITORY: &str = "GITHUB_REPOSITORY";
const GITHUB_OUTPUT: &str = "GITHUB_OUTPUT";
const GITHUB_TOKEN: &str = "INPUT_GITHUB-TOKEN";
const TAG_PREFIX: &str = "INPUT_TAG-PREFIX";
const GITHUB_HOST: &str = "INPUT_GITHUB-HOST";
const COMMITISH: &str = "GITHUB_SHA";
const BODY: &str = "INPUT_BODY";
const GENERATE_RELEASE_NOTES: &str = "INPUT_GENERATE-RELEASE-NOTES";

#[derive(Debug)]
pub struct Config {
    pub github_output_path: String,
    pub github_token: Token,
    pub github_host: String,
    pub increment_strategy: VersionIncrementStrategy,
    pub default_version: String,
    pub tag_prefix: String,
    pub repo: String,
    pub owner: String,
    pub commitish: String,
    pub body: String,
    pub generate_release_notes: bool,
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
            github_host: get_github_host(),
            increment_strategy: get_version_increment_strategy(),
            default_version: get_default_version(),
            tag_prefix: get_tag_prefix(),
            repo,
            owner,
            commitish: get_commitish(),
            body: get_body(),
            generate_release_notes: get_generate_release_notes(),
        }
    }

    pub fn get_default_tag(&self) -> String {
        format!("{}{}", self.tag_prefix, self.default_version)
    }
}

fn get_github_token() -> String {
    env::var(GITHUB_TOKEN).unwrap_or_else(|e| {
        eprintln!("Could not read {}", GITHUB_TOKEN);
        eprintln!("Error {}", e);
        exit(1);
    })
}

fn get_github_output_path() -> String {
    env::var(GITHUB_OUTPUT).unwrap_or_else(|e| {
        eprintln!("Could not read {}", GITHUB_OUTPUT);
        eprintln!("Error {}", e);
        exit(1);
    })
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
    env::var(INITIAL_VERSION).unwrap_or("0.1.0".to_string())
}

fn get_repo_info() -> (String, String) {
    env::var(GITHUB_REPOSITORY).map_or_else(
        |e| {
            eprintln!("Could not read {}", GITHUB_REPOSITORY);
            eprintln!("Error {}", e);
            exit(1);
        },
        |v| {
            let repo_info = v.split('/').collect::<Vec<&str>>();
            (repo_info[0].to_owned(), repo_info[1].to_owned())
        },
    )
}

fn get_tag_prefix() -> String {
    env::var(TAG_PREFIX).unwrap_or("v".to_string())
}

fn get_github_host() -> String {
    env::var(GITHUB_HOST).unwrap_or("https://api.github.com".to_string())
}

fn get_commitish() -> String {
    env::var(COMMITISH).unwrap_or_else(|e| {
        eprintln!("Could not read {}", COMMITISH);
        eprintln!("Error {}", e);
        exit(1);
    })
}

fn get_body() -> String {
    env::var(BODY).unwrap_or_else(|e| {
        eprintln!("Could not read {}", BODY);
        eprintln!("Error {}", e);
        exit(1);
    })
}

fn get_generate_release_notes() -> bool {
    env::var(GENERATE_RELEASE_NOTES).map_or_else(
        |e| {
            eprintln!("Could not read {}", GENERATE_RELEASE_NOTES);
            eprintln!("Error {}", e);
            exit(1);
        },
        |v| matches!(v.to_ascii_lowercase().as_str(), "true"),
    )
}
