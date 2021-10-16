use std::fmt;

use crate::repo;
use chrono::DateTime;
use serde::Deserialize;

#[derive(Deserialize, Debug)]
struct ImageDetails {
    architecture: String,
    // os: String,
    size: usize,
}

impl fmt::Display for ImageDetails {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}|{}MB", self.architecture, self.size / 1024 / 1024)
    }
}

#[derive(Deserialize)]
pub struct Images {
    images: Vec<ImageDetails>,
    #[serde(rename(deserialize = "name"))]
    pub tag_name: String,
    last_updated: String,
}

#[derive(Deserialize)]
pub struct Tags {
    count: usize,
    #[serde(rename(deserialize = "next"))]
    next_page: Option<String>,
    #[serde(rename(deserialize = "previous"))]
    prev_page: Option<String>,
    pub results: Vec<Images>,
}

#[derive(Debug, PartialEq)]
pub enum Error {
    /// couldn't fetch json with reqwest
    Fetching(String),
    /// a serde error
    Converting(String),
    /// invalid repos show a valid json with 0 tags
    NoTagsFound,
    NoPrevPage,
    NoNextPage,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::Fetching(s) => write!(f, "Fetching error: {}", s),
            Error::Converting(s) => write!(f, "Converting error: {}", s),
            Error::NoNextPage => write!(f, "No next page available"),
            Error::NoPrevPage => write!(f, "No previous page available"),
            Error::NoTagsFound => write!(f, "Given Repo has 0 tags. Is it valid?"),
        }
    }
}

impl Tags {
    /// fetches tag information with a repository name in the form of organization/repository or library/repository in the case of official images from docker
    pub fn new(repo: String) -> Result<Self, Error> {
        let request = format!("https://hub.docker.com/v2/repositories/{}/tags", repo);
        Self::with_url(&request)
    }

    /// fetches tag information from a url
    fn with_url(url: &str) -> Result<Self, Error> {
        let res = match reqwest::blocking::get(url) {
            Ok(result) => result,
            Err(e) => return Err(Error::Fetching(format!("reqwest error: {}", e))),
        };

        //convert it to json
        let raw = res.text().unwrap();
        let tags: Self = match serde_json::from_str(&raw) {
            Ok(result) => result,
            Err(e) => return Err(Error::Converting(format!("invalid json: {}", e))),
        };

        if tags.count == 0 {
            return Err(Error::NoTagsFound);
        }

        Ok(tags)
    }

    /// checks the repo name and may add a prefix for official images
    pub fn check_repo(name: &str) -> Result<String, Error> {
        let repo = match repo::split_tag_from_repo(name) {
            Err(e) => return Err(Error::Converting(format!("{}", e))),
            Ok((name, _)) => name,
        };

        match repo::split_repo_without_tag(name) {
            Ok(repo::Repo::Project(s)) => Ok(format!("library/{}", s)),
            Ok(_) => Ok(repo.to_string()),
            Err(e) => Err(Error::Converting(format!("{}", e))),
        }
    }

    /// returns tags of next page
    pub fn next_page(&self) -> Result<Self, Error> {
        match &self.next_page {
            Some(url) => Self::with_url(url),
            None => Err(Error::NoNextPage),
        }
    }

    /// returns tags of previous page
    pub fn prev_page(&self) -> Result<Self, Error> {
        match &self.prev_page {
            Some(url) => Self::with_url(url),
            None => Err(Error::NoPrevPage),
        }
    }
}

impl fmt::Display for Images {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        //architecture infos
        let mut arch = String::new();
        for image in self.images.iter().take(1) {
            arch.push_str(&format!("{}", image));
        }
        for image in self.images.iter().skip(1) {
            arch.push_str(&format!(", {}", image));
        }

        let now = chrono::Utc::now();
        let rfc3339 = DateTime::parse_from_rfc3339(&self.last_updated).unwrap();
        let dif = now - rfc3339.with_timezone(&chrono::Utc);
        write!(
            f,
            "{} vor {} [{}]",
            self.tag_name,
            format_time_nice(dif),
            arch
        )
    }
}

/// converts a given duration to a readable string
fn format_time_nice(time: chrono::Duration) -> String {
    if time.num_weeks() == 52 {
        format!("{} Jahr", (time.num_weeks() / 52) as i32)
    } else if time.num_weeks() > 103 {
        format!("{} Jahren", (time.num_weeks() / 52) as i32)
    } else if time.num_days() == 1 {
        format!("{} Tag", time.num_days())
    } else if time.num_days() > 1 {
        format!("{} Tagen", time.num_days())
    } else if time.num_hours() == 1 {
        format!("{} Stunde", time.num_hours())
    } else if time.num_hours() > 1 {
        format!("{} Stunden", time.num_hours())
    } else if time.num_minutes() == 1 {
        format!("{} Minute", time.num_minutes())
    } else if time.num_minutes() > 1 {
        format!("{} Minuten", time.num_minutes())
    } else {
        format!("{} Sekunden", time.num_seconds())
    }
}

#[cfg(test)]
mod tests {
    use crate::tags::{Error, Tags};
    #[test]
    fn test_check_repo() {
        assert_eq!(Tags::check_repo("nginx").unwrap(), "library/nginx");
        assert_eq!(Tags::check_repo("library/nginx").unwrap(), "library/nginx");
        assert_eq!(
            Tags::check_repo("rocketchat/rocket.chat").unwrap(),
            "rocketchat/rocket.chat"
        );
    }
}
