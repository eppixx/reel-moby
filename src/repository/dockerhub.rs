use serde::Deserialize;

use crate::repository::Error;

#[derive(Deserialize, Debug, Clone)]
struct ImageDetails {
    architecture: String,
    os: String,
    variant: Option<String>,
    size: usize,
}

#[derive(Deserialize, Clone)]
pub struct Images {
    images: Vec<ImageDetails>,
    #[serde(rename(deserialize = "name"))]
    tag_name: String,
    last_updated: String,
}

impl Images {
    pub fn convert(&self) -> super::Tag {
        super::Tag {
            name: self.tag_name.clone(),
            last_updated: Some(self.last_updated.clone()),
            details: self
                .images
                .iter()
                .map(|d| super::TagDetails {
                    arch: Some(d.architecture.clone()),
                    variant: Some(d.variant.clone().unwrap_or_default()),
                    os: Some(d.os.clone()),
                    size: Some(d.size),
                })
                .collect(),
        }
    }
}

#[derive(Deserialize)]
pub struct DockerHub {
    #[serde(rename(deserialize = "next"))]
    next_page: Option<String>,
    results: Vec<Images>,
}

impl DockerHub {
    /// fetches tag information with a repository name in the form of organization/repository or library/repository in the case of official images from docker
    pub fn create_repo(repo: &str) -> Result<super::Repo, Error> {
        let request = format!("https://hub.docker.com/v2/repositories/{}/tags", repo);
        Self::with_url(&request)
    }

    /// fetches tag information from a url
    pub fn with_url(url: &str) -> Result<super::Repo, Error> {
        let response = match reqwest::blocking::get(url) {
            Ok(result) => result,
            Err(e) => return Err(Error::Fetching(format!("reqwest error: {}", e))),
        };

        //convert it to json
        let tags = match response.json::<Self>() {
            Ok(result) => result,
            Err(e) => return Err(Error::Converting(format!("invalid json: {}", e))),
        };

        if tags.results.is_empty() {
            return Err(Error::NoTagsFound);
        }

        Ok(super::Repo {
            tags: tags.results.iter().map(|t| t.convert()).collect(),
            next_page: tags.next_page,
        })
    }
}
