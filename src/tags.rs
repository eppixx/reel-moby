use std::fmt;

use chrono::DateTime;
use serde::Deserialize;

#[derive(Deserialize)]
struct ImageDetails {
    architecture: String,
    os: String,
    size: usize,
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

#[derive(Debug)]
pub enum Error {
    /// repo string contains an illegal character
    InvalidCharacter(char),
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
            Error::InvalidCharacter(c) => write!(f, "Invalid Character: {}", c),
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
    pub fn check_repo(mut name: String) -> Result<String, Error> {
        //check for right set of characters
        if name.bytes().any(|c| !c.is_ascii()) {
            return Err(Error::InvalidCharacter('a'));
        }

        //check if need to inject "library" of given repo
        let regex = regex::Regex::new(r".*/.*").unwrap();
        if !regex.is_match(&name) {
            name.insert_str(0, "library/");
        }
        Ok(name)
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
        let now = chrono::Utc::now();
        let rfc3339 = DateTime::parse_from_rfc3339(&self.last_updated).unwrap();
        let dif = now - rfc3339.with_timezone(&chrono::Utc);
        write!(f, "{} vor {}", self.tag_name, format_time_nice(dif))
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
    use crate::tags;
    #[test]
    fn test_check_repo() {
        let check_eq = |s, s2| {
            assert_eq!(&tags::Tags::check_repo(String::from(s)).unwrap(), s2);
        };
        let check_neq = |s, s2| {
            assert_ne!(&tags::Tags::check_repo(String::from(s)).unwrap(), s2);
        };
        let check_err = |s: &str| {
            assert_eq!(tags::Tags::check_repo(String::from(s)).is_err(), true);
        };

        check_eq("nginx", "library/nginx");
        check_neq("nginx", "nginx");
        check_eq("rocketchat/rocket.chat", "rocketchat/rocket.chat");
        check_eq("mysql", "library/mysql");
        check_neq("mysql", "mysql");
        check_err("nginxä");
        check_err("nginx²");
        check_eq("selim13/automysqlbackup", "selim13/automysqlbackup");
    }
}
