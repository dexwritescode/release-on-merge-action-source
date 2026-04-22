use serde::Deserialize;
use serde::Serialize;

#[derive(Deserialize, Debug, Clone)]
pub struct TagName {
    pub tag_name: String,
}

#[derive(Serialize, Debug, Clone)]
pub struct CreateReleaseRequest {
    pub tag_name: String,
    pub target_commitish: String,
    pub name: String,
    pub body: String,
    pub draft: bool,
    pub prerelease: bool,
    pub generate_release_notes: bool,
}

#[derive(Deserialize, Debug, Clone)]
pub struct Label {
    pub name: String,
}

#[derive(Deserialize, Debug, Clone)]
pub struct PullRequest {
    pub labels: Vec<Label>,
}
