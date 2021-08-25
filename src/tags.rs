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
struct Images {
    images: Vec<ImageDetails>,
    last_updater_username: String,
    #[serde(rename(deserialize = "name"))]
    tag_name: String,
    last_updated: String,
}

#[derive(Deserialize)]
struct TagList {
    count: i32,
    next_page: Option<String>,
    prev_page: Option<String>,
    results: Vec<Images>,
}

pub enum Error {
    InvalidCharacter(char),
    Fetching(String),
    Converting(String),
}

pub struct Tags {}

impl Tags {
    pub fn get_tags(repo: String) -> Result<Vec<String>, Error> {
        let request = format!("https://hub.docker.com/v2/repositories/{}/tags", repo);

        //check for right set of characters
        if request.bytes().any(|c| !c.is_ascii()) {
            return Err(Error::InvalidCharacter('a'));
        }

        //get response
        let res = match reqwest::blocking::get(request) {
            Ok(result) => result,
            Err(_) => return Err(Error::Fetching(String::from("reqwest error"))),
        };

        //convert it to json
        let raw = res.text().unwrap();
        let tags: TagList = match serde_json::from_str(&raw) {
            Ok(result) => result,
            Err(_) => return Err(Error::Converting(String::from("invalid json"))),
        };

        let now = chrono::Utc::now();

        Ok(tags
            .results
            .iter()
            .map(|r| {
                let rfc3339 = DateTime::parse_from_rfc3339(&r.last_updated).unwrap();
                let dif = now - rfc3339.with_timezone(&chrono::Utc);
                format!("{} vor {}", r.tag_name, Self::format_time_nice(dif))
            })
            .collect())
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
}
