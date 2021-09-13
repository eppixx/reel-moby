use crate::common;

#[derive(Debug, PartialEq)]
pub enum Error {
    Conversion,
    Empty,
    NoTagFound,
    InvalidChar,
    MisformedInput,
}

#[derive(Debug, PartialEq)]
pub enum Repo {
    WithServer(String, String, String),
    WithOrga(String, String),
    Project(String),
}

/// check if yaml line matches and returns the split of repo string and rest
pub fn match_yaml_image(input: &str) -> Option<(&str, &str)> {
    lazy_static::lazy_static! {
        static ref REGEX: regex::Regex = regex::Regex::new(r"^( +image *: *)([a-z0-9\./:]+)").unwrap();
    }
    let caps = match REGEX.captures(input) {
        Some(caps) => caps,
        None => return None,
    };

    Some((caps.get(1).unwrap().as_str(), caps.get(2).unwrap().as_str()))
}

pub fn split_repo(repo: &str) -> Result<Repo, Error> {
    let split_tag: Vec<&str> = repo.split(":").collect();
    if split_tag.len() == 2 && split_tag[0].len() != 0 && split_tag[1].len() != 0 {
        //
    }
    Ok(Repo::Project("".into()))
}

pub fn split_repo_without_tag(mut repo: &str) -> Result<Repo, Error> {
    repo.trim();
    let split_repo: Vec<&str> = repo.split("/").collect();
    match split_repo.len() {
        1 => {
            let regex = regex::Regex::new(r"[a-z0-9]+").unwrap();
            match regex.is_match(repo) {
                false => Err(Error::MisformedInput),
                true => Ok(Repo::Project(split_repo[0].into())),
            }
        }
        2 => {
            let regex = regex::Regex::new(r"[a-z0-9]+/[a-z0-9]+").unwrap();
            match regex.is_match(repo) {
                false => Err(Error::MisformedInput),
                true => Ok(Repo::WithOrga(split_repo[0].into(), split_repo[1].into())),
            }
        }
        3 => {
            let regex = regex::Regex::new(r"[a-z0-9\.]+/[a-z0-9]+/[a-z0-9]+").unwrap();
            match regex.is_match(repo) {
                false => Err(Error::MisformedInput),
                true => Ok(Repo::WithServer(
                    split_repo[0].into(),
                    split_repo[1].into(),
                    split_repo[2].into(),
                )),
            }
        }
        _ => Err(Error::MisformedInput),
    }
}

pub fn split_tag(repo: &str) -> Result<(&str, &str), Error> {
    let split_tag: Vec<&str> = repo.split(":").collect();
    if split_tag.len() == 2 && split_tag[0].len() != 0 && split_tag[1].len() != 0 {
        Ok((split_tag[0], split_tag[1]))
    } else {
        Err(Error::NoTagFound)
    }
}

// fn

pub fn extract(repo: &str) -> Result<(Option<&str>, Option<&str>, &str), Error> {
    if repo.len() == 0 {
        return Err(Error::Empty);
    }
    let regex = regex::Regex::new(r"([^/:]*?/)??([^/:]*?/)?([^/:]*):?(.*)?").unwrap();
    let caps = match regex.captures(repo) {
        None => return Err(Error::Conversion),
        Some(cap) => cap,
    };
    let server = match caps.get(1) {
        None => None,
        Some(cap) => Some(common::remove_last_char(cap.as_str())),
    };
    let orga = match caps.get(2) {
        None => None,
        Some(cap) => Some(common::remove_last_char(cap.as_str())),
    };

    Ok((server, orga, caps.get(3).unwrap().as_str()))
}

#[cfg(test)]
mod tests {
    use crate::repo;
    use crate::repo::{Error, Repo};

    // #[test]
    fn test_repo_regex() {
        assert_eq!(repo::extract(""), Err(repo::Error::Empty));
        assert_eq!(
            repo::extract("ghcr.io/library/nginx"),
            Ok((Some("ghcr.io"), Some("library"), "nginx"))
        );
        assert_eq!(
            repo::extract("library/nginx"),
            Ok((None, Some("library"), "nginx"))
        );
        assert_eq!(repo::extract("nginx"), Ok((None, None, "nginx")));
    }

    #[test]
    fn split_tag() {
        assert_eq!(repo::split_tag("nginx:v1"), Ok(("nginx", "v1")));
        assert_eq!(repo::split_tag("dsfsdf"), Err(repo::Error::NoTagFound));
        assert_eq!(repo::split_tag("nginx:"), Err(repo::Error::NoTagFound));
        assert_eq!(repo::split_tag(":v1"), Err(repo::Error::NoTagFound));
        assert_eq!(repo::split_tag(":"), Err(repo::Error::NoTagFound));
    }

    #[test]
    fn test_split_repo_without_tag() {
        use crate::repo::split_repo_without_tag as test_fn;
        assert_eq!(test_fn(""), Err(Error::MisformedInput));
        assert_eq!(test_fn("NGINX"), Err(Error::MisformedInput));
        assert_eq!(test_fn("nginx"), Ok(Repo::Project("nginx".into())));
        assert_eq!(
            test_fn("library/nginx"),
            Ok(Repo::WithOrga("library".into(), "nginx".into()))
        );
        assert_eq!(
            test_fn("ghcr.io/library/nginx"),
            Ok(Repo::WithServer(
                "ghcr.io".into(),
                "library".into(),
                "nginx".into(),
            ))
        );
    }

    #[test]
    fn test_match_yaml_image() {
        use crate::repo::match_yaml_image as test_fn;
        assert_eq!(test_fn(""), None);
        assert_eq!(test_fn("version: '2'"), None);
        assert_eq!(test_fn("image: "), None);
        assert_eq!(test_fn("  image: "), None);
        assert_eq!(test_fn("  image: nginx"), Some(("  image: ", "nginx")));
        assert_eq!(
            test_fn("  image: library/nginx"),
            Some(("  image: ", "library/nginx"))
        );
        assert_eq!(
            test_fn("  image: ghcr.io/library/nginx"),
            Some(("  image: ", "ghcr.io/library/nginx"))
        );
        assert_eq!(test_fn("#   image: nginx"), None);
        assert_eq!(
            test_fn("   image: nginx #comment"),
            Some(("   image: ", "nginx"))
        );
    }
}
