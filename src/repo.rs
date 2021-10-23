use std::fmt;

use regex::Regex;

// use crate::common;

#[derive(Debug, PartialEq)]
pub enum Error {
    // Conversion,
    // Empty,
    NoTagFound,
    // InvalidChar,
    MisformedInput,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            // Error::Conversion => write!(f, "Conversion error"),
            // Error::Empty => write!(f, "Input is empty"),
            Error::NoTagFound => write!(f, "Expected a tag"),
            // Error::InvalidChar => write!(f, "Invalid character found"),
            Error::MisformedInput => write!(f, "Unexpected input"),
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum Repo {
    WithServer(String, String, String),
    WithOrga(String, String),
    Project(String),
}

/// check if yaml line matches and returns the split of repo string and rest
pub fn match_yaml_image(input: &str) -> Result<(&str, &str), Error> {
    lazy_static::lazy_static! {
        static ref REGEX: Regex = Regex::new(r"^( +image *: *)([a-z0-9\./:]+)").unwrap();
    }
    let caps = match REGEX.captures(input) {
        Some(caps) => caps,
        None => return Err(Error::NoTagFound),
    };

    Ok((caps.get(1).unwrap().as_str(), caps.get(2).unwrap().as_str()))
}

pub fn split_tag_from_repo(input: &str) -> Result<(&str, &str), Error> {
    lazy_static::lazy_static! {
        static ref REGEX: Regex = Regex::new(r"^([a-z0-9\./[^:]]*):?([a-z0-9._\-]*)").unwrap();
    }
    let (front, back) = match REGEX.captures(input) {
        None => return Err(Error::MisformedInput),
        Some(caps) => {
            let front = match caps.get(1) {
                None => return Err(Error::MisformedInput),
                Some(cap) => cap.as_str(),
            };
            let back = match caps.get(2) {
                None => "",
                Some(cap) => cap.as_str(),
            };
            (front, back)
        }
    };

    Ok((front, back))
}

pub fn split_repo_without_tag(repo: &str) -> Result<Repo, Error> {
    let repo = repo.trim();
    let split_repo: Vec<&str> = repo.split('/').collect();
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

#[cfg(test)]
mod tests {
    use crate::repo::{Error, Repo};

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
        assert_eq!(test_fn(""), Err(Error::NoTagFound));
        assert_eq!(test_fn("version: '2'"), Err(Error::NoTagFound));
        assert_eq!(test_fn("image: "), Err(Error::NoTagFound));
        assert_eq!(test_fn("  image: "), Err(Error::NoTagFound));
        assert_eq!(test_fn("  image: nginx"), Ok(("  image: ", "nginx")));
        assert_eq!(
            test_fn("  image: library/nginx"),
            Ok(("  image: ", "library/nginx"))
        );
        assert_eq!(
            test_fn("  image: ghcr.io/library/nginx"),
            Ok(("  image: ", "ghcr.io/library/nginx"))
        );
        assert_eq!(test_fn("#   image: nginx"), Err(Error::NoTagFound));
        assert_eq!(
            test_fn("   image: nginx #comment"),
            Ok(("   image: ", "nginx"))
        );
    }

    #[test]
    fn test_split_tag_from_repo() {
        use crate::repo::split_tag_from_repo as test_fn;
        assert_eq!(test_fn("nginx"), Ok(("nginx", "")));
        assert_eq!(test_fn("library/nginx"), Ok(("library/nginx", "")));
        assert_eq!(
            test_fn("ghcr.io/library/nginx"),
            Ok(("ghcr.io/library/nginx", ""))
        );
        assert_eq!(test_fn("nginx:"), Ok(("nginx", "")));
        assert_eq!(test_fn("nginx:1"), Ok(("nginx", "1")));
        assert_eq!(test_fn("nginx:latest"), Ok(("nginx", "latest")));
    }
}
