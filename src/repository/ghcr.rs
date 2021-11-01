use serde::Deserialize;

use crate::repository::Error;

#[derive(Deserialize)]
struct Token {
    token: String,
}

#[derive(Deserialize)]
pub struct Ghcr {
    tags: Vec<String>,
}

impl Ghcr {
    /// fetches tag information with a repository name in the form of organization/repository or library/repository in the case of official images from docker
    pub fn create_repo(repo: &str) -> Result<super::Repo, Error> {
        let request_token = format!("https://ghcr.io/token?scope=repository:{}:pull", repo);
        let response = match reqwest::blocking::get(request_token) {
            Err(e) => return Err(Error::Fetching(format!("reqwest error: {}", e))),
            Ok(response) => response,
        };

        let token = match response.json::<Token>() {
            Err(e) => return Err(Error::Converting(format!("invalid token json: {}", e))),
            Ok(token) => token.token,
        };

        let request = format!("https://ghcr.io/v2/{}/tags/list?n=100", repo);
        let client = reqwest::blocking::Client::new();
        let response = match client
            .get(request)
            .header(reqwest::header::AUTHORIZATION, format!("Bearer {}", token))
            .send()
        {
            // let response = match reqwest::blocking::get(url) {
            Ok(result) => result,
            Err(e) => return Err(Error::Fetching(format!("reqwest error: {}", e))),
        };

        //convert it to json
        let tags = match response.json::<Self>() {
            Ok(result) => result,
            Err(e) => return Err(Error::Converting(format!("invalid json: {}", e))),
        };

        if tags.tags.is_empty() {
            return Err(Error::NoTagsFound);
        }

        Ok(super::Repo {
            tags: tags
                .tags
                .iter()
                .map(|t| super::Tag {
                    name: t.clone(),
                    details: vec![],
                    last_updated: None,
                })
                .collect(),
            next_page: None,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::Ghcr;
    #[test]
    fn test_ghcr() {
        Ghcr::create_repo("ghcr.io/linuxserver/beets").unwrap();
    }
}
