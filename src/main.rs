use chrono::DateTime;
use serde::Deserialize;

mod ui;
mod widget;

#[derive(Deserialize)]
struct Image {
    architecture: String,
    os: String,
    size: i32,
    last_pulled: String,
    last_pushed: String,
}

#[derive(Deserialize)]
struct Result {
    images: Vec<Image>,
    last_updater_username: String,
    #[serde(rename(deserialize = "name"))]
    tag_name: String,
    last_updated: String,
}

#[derive(Deserialize)]
struct Tags {
    count: i32,
    next_page: Option<String>,
    prev_page: Option<String>,
    results: Vec<Result>,
}

fn main() {
    //docker hub exposes tags stored in json at the following url
    //https://hub.docker.com/v2/repositories/rocketchat/rocket.chat/tags

    //TODO fill them dynamic instead of hardcoded
    let group = "rocketchat";
    let repo = "rocket.chat";
    let request = format!(
        "https://hub.docker.com/v2/repositories/{}/{}/tags",
        group, repo
    );

    //get response
    let res = reqwest::blocking::get(request).unwrap();

    //convert it to json
    let raw = res.text().unwrap();
    let tags: Tags = serde_json::from_str(&raw).unwrap();

    let now = chrono::Utc::now();
    for result in tags.results {
        let rfc3339 = DateTime::parse_from_rfc3339(&result.last_updated).unwrap();
        let dif = now - rfc3339.with_timezone(&chrono::Utc);
        println!("{} vor {}", result.tag_name, format_time_nice(dif));
    }

    ui::Ui::new();
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
