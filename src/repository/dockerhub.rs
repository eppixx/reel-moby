use serde::Deserialize;

use crate::error::Error;

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
    pub fn from_tag(images: &Self) -> super::Tag {
        super::Tag {
            name: images.tag_name.clone(),
            last_updated: Some(images.last_updated.clone()),
            details: images
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
    pub async fn create_repo(repo: &str) -> Result<super::Repo, Error> {
        let request = format!("https://hub.docker.com/v2/repositories/{}/tags", repo);
        Self::with_url(&request).await
    }

    /// fetches tag information from a url
    pub async fn with_url(url: &str) -> Result<super::Repo, Error> {
        let response = reqwest::get(url).await?;

        //convert it to json
        let tags = response.json::<Self>().await?;
        if tags.results.is_empty() {
            return Err(Error::NoTagsFound);
        }

        Ok(super::Repo {
            tags: tags.results.iter().map(Images::from_tag).collect(),
            next_page: tags.next_page,
        })
    }
}
