use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct LatestRelease {
    pub tag_name: String,
}
