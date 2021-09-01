use std::fmt;

use chrono::DateTime;
use serde::Deserialize;

#[derive(Deserialize)]
struct ImageDetails {
    architecture: String,
    os: String,
    size: i32,
    last_pulled: String,
    last_pushed: String,
}

#[derive(Deserialize)]
pub struct Images {
    images: Vec<ImageDetails>,
    last_updater_username: String,
    #[serde(rename(deserialize = "name"))]
    pub tag_name: String,
    last_updated: String,
}

#[derive(Deserialize)]
pub struct Tags {
    count: i32,
    next_page: Option<String>,
    prev_page: Option<String>,
    pub results: Vec<Images>,
}

#[derive(Debug)]
pub enum Error {
    InvalidCharacter(char),
    Fetching(String),
    Converting(String),
}

impl Tags {
    pub fn new(repo: String) -> Result<Self, Error> {
        // let repo = Self::check_repo(repo)?;
        let request = format!("https://hub.docker.com/v2/repositories/{}/tags", repo);

        //get response
        let res = match reqwest::blocking::get(request) {
            Ok(result) => result,
            Err(_) => return Err(Error::Fetching(String::from("reqwest error"))),
        };

        //convert it to json
        let raw = res.text().unwrap();
        let tags: Self = match serde_json::from_str(&raw) {
            Ok(result) => result,
            Err(_) => return Err(Error::Converting(String::from("invalid json"))),
        };

        Ok(tags)
    }

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
}

impl fmt::Display for Images {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let now = chrono::Utc::now();
        let rfc3339 = DateTime::parse_from_rfc3339(&self.last_updated).unwrap();
        let dif = now - rfc3339.with_timezone(&chrono::Utc);
        write!(f, "{} vor {}", self.tag_name, format_time_nice(dif))
    }
}

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
