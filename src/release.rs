use crate::Config;
use crate::Semver;
use octocrab::{Error, Octocrab};
use std::{process::exit, str::FromStr};

pub struct Release {
    client: Octocrab,
    repo: String,
    owner: String,
}

impl Release {
    pub fn new(config: &Config) -> Release {
        let client = match Octocrab::builder()
            .personal_token(config.github_token.0.clone())
            .build()
        {
            Ok(c) => c,
            Err(e) => {
                eprintln!("Error building client {:?}", e);
                exit(1);
            }
        };
        Release {
            client,
            repo: config.repo.clone(),
            owner: config.owner.clone(),
        }
    }

    pub async fn get_latest_tag(&self) -> Option<Semver> {
        self.client
            .repos(&self.owner, &self.repo)
            .releases()
            .get_latest()
            .await
            .map_or_else(
                |e| match e {
                    Error::GitHub { ref source, .. } => {
                        if source.message.eq_ignore_ascii_case("Not Found")
                            && source.errors.is_none()
                        {
                            None
                        } else {
                            eprintln!("Could not get the version.");
                            eprintln!("Error: {:?}", &e);
                            exit(1);
                        }
                    }
                    _ => {
                        eprintln!("Could not get the version.");
                        eprintln!("Error: {:?}", &e);
                        exit(1);
                    }
                },
                |r| Semver::from_str(&r.tag_name).ok(),
            )
    }
}
