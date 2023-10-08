use std::{env, fmt, process::exit};

#[derive(Debug)]
pub struct Config {
    pub github_output_path: String,
    pub github_token: Token,
    pub version_increment_strategy: String,
    pub default_version: String,
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
        Config {
            github_output_path: get_github_output_path(),
            github_token: Token(get_github_token()),
            version_increment_strategy: get_version_increment_strategy(),
            default_version: get_default_version(),
        }
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

fn get_version_increment_strategy() -> String {
    match env::var("INPUT_VERSION-INCREMENT-STRATEGY") {
        Ok(value) => value,
        Err(_) => {
            eprintln!("INPUT_VERSION-INCREMENT-STRATEGY should be set");
            exit(1);
        }
    }
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
