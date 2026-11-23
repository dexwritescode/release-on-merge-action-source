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
    /// When the PR was merged; None while it's open.
    #[serde(default)]
    pub merged_at: Option<String>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct Comparison {
    pub commits: Vec<ComparedCommit>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct ComparedCommit {
    pub commit: CommitMessage,
}

#[derive(Deserialize, Debug, Clone)]
pub struct CommitMessage {
    pub message: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn comparison_reads_commit_messages() {
        let json = r#"{"ahead_by": 2, "commits": [
            {"sha": "a", "commit": {"message": "Merge pull request #7", "author": {}}},
            {"sha": "b", "commit": {"message": "Next development version 1.0.1-dev"}}
        ]}"#;
        let comparison: Comparison = serde_json::from_str(json).unwrap();
        let messages: Vec<_> = comparison.commits.iter().map(|c| c.commit.message.as_str()).collect();
        assert_eq!(messages, ["Merge pull request #7", "Next development version 1.0.1-dev"]);
    }
}
