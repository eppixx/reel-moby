mod dockerhub;
mod ghcr;

use std::fmt;

use chrono::DateTime;

use crate::repo;

#[derive(Debug, PartialEq)]
pub enum Error {
    /// couldn't fetch json with reqwest
    Fetching(String),
    /// a serde error
    Converting(String),
    /// invalid repos show a valid json with 0 tags
    NoTagsFound,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::Fetching(s) => write!(f, "Fetching error: {}", s),
            Error::Converting(s) => write!(f, "Converting error: {}", s),
            Error::NoTagsFound => write!(f, "Given Repo has 0 tags. Is it valid?"),
        }
    }
}

#[derive(Clone, PartialEq)]
pub struct TagDetails {
    pub arch: Option<String>,
    pub os: Option<String>,
    pub size: Option<usize>,
}

impl fmt::Display for TagDetails {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let size = match self.size {
            None => "".to_string(),
            Some(s) => (s / 1024 / 1024).to_string(),
        };
        write!(
            f,
            "{}|{}MB",
            self.arch.as_ref().unwrap_or(&"".to_string()),
            size
        )
    }
}

#[derive(Clone)]
pub struct Tag {
    name: String,
    details: Vec<TagDetails>,
    last_updated: Option<String>,
}

impl Tag {
    pub fn get_name(&self) -> &str {
        &self.name
    }

    pub fn get_name_with_details(&self) -> String {
        //architecture infos
        let mut arch = String::new();
        for image in self.details.iter().take(1) {
            arch.push_str(&format!("{}", image));
        }
        for image in self.details.iter().skip(1) {
            arch.push_str(&format!(", {}", image));
        }

        let dif = match &self.last_updated {
            None => "".to_string(),
            Some(last_updated) => {
                let now = chrono::Utc::now();
                let rfc3339 = DateTime::parse_from_rfc3339(last_updated).unwrap();
                let dif = now - rfc3339.with_timezone(&chrono::Utc);
                format!(" vor {}", format_time_nice(dif))
            }
        };

        if dif.is_empty() {}
        format!("{}{}", self.name, dif)
    }

    pub fn get_details(&self) -> &Vec<TagDetails> {
        &self.details
    }
}

pub struct Repo {
    tags: Vec<Tag>,
    next_page: Option<String>,
}

impl Repo {
    pub fn new(repo: &str) -> Result<Self, Error> {
        use crate::repo::Repo;
        let (registry, repo) = match crate::repo::split_repo_without_tag(repo) {
            Ok(Repo::WithServer(reg, org, pro)) => (Some(reg), format!("{}/{}", org, pro)),
            Ok(Repo::WithOrga(org, pro)) => (None, format!("{}/{}", org, pro)),
            Ok(Repo::Project(pro)) => (None, format!("library/{}", pro)),
            Err(e) => return Err(Error::Converting(format!("{}", e))),
        };

        if registry.unwrap_or_default() == "ghcr.io" {
            ghcr::Ghcr::create_repo(&repo)
        } else {
            dockerhub::DockerHub::create_repo(&repo)
        }
    }

    pub fn with_url(url: &str) -> Result<Self, Error> {
        //TODO fix for other registries
        dockerhub::DockerHub::with_url(url)
    }

    pub fn get_tags(&self) -> &Vec<Tag> {
        &self.tags
    }

    pub fn next_page(&self) -> Option<Self> {
        match &self.next_page {
            Some(url) => match Self::with_url(url) {
                Ok(tags) => Some(tags),
                Err(_) => None,
            },
            None => None,
        }
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

#[cfg(test)]
mod tests {
    #[test]
    fn test_check_repo() {
        assert_eq!(super::check_repo("nginx").unwrap(), "library/nginx");
        assert_eq!(super::check_repo("library/nginx").unwrap(), "library/nginx");
        assert_eq!(
            super::check_repo("rocketchat/rocket.chat").unwrap(),
            "rocketchat/rocket.chat"
        );
    }
}
