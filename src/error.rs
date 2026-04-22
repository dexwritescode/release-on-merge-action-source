use std::fmt;

#[derive(Debug)]
pub enum ActionError {
    MissingEnv(&'static str),
    InvalidStrategy(String),
    InvalidTag(String),
    Unauthorized,
    ApiError(String),
    UnexpectedStatus(u16),
}

impl fmt::Display for ActionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ActionError::MissingEnv(var) => write!(f, "Required environment variable not set: {var}"),
            ActionError::InvalidStrategy(s) => write!(f, "Invalid version-increment-strategy: {s}"),
            ActionError::InvalidTag(s) => write!(f, "Could not parse tag as semver: {s}"),
            ActionError::Unauthorized => write!(f, "Unauthorized — check your GITHUB_TOKEN"),
            ActionError::ApiError(msg) => write!(f, "GitHub API error: {msg}"),
            ActionError::UnexpectedStatus(code) => write!(f, "Unexpected HTTP status: {code}"),
        }
    }
}
