use regex::Regex;

use crate::error::Error;

#[derive(Debug, PartialEq)]
pub enum Repo {
    WithServer(String, String, String),
    WithOrga(String, String),
    Project(String),
}

/// check if yaml line matches and returns the split of repo string and rest
/// the first &str is the image tag
/// it will be used to not change the identation
/// the second &str will the the identifier for the image
pub fn match_yaml_image(input: &str) -> Result<(&str, &str), Error> {
    lazy_static::lazy_static! {
        static ref REGEX: Regex = Regex::new(r"^( +image *: *)([a-z0-9\-\./:]+)").unwrap();
    }
    let caps = match REGEX.captures(input) {
        Some(caps) => caps,
        None => return Err(Error::NoTagFound),
    };

    Ok((caps.get(1).unwrap().as_str(), caps.get(2).unwrap().as_str()))
}

/// takes the identifier and splits off the tag it exists
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

/// takes an identifier and changes it to a Repo enum
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
        let input: Vec<(&str, Result<Repo, Error>)> = vec![
            ("", Err(Error::MisformedInput)),
            ("NGINX", Err(Error::MisformedInput)),
            ("nginx", Ok(Repo::Project("nginx".into()))),
            (
                "library/nginx",
                Ok(Repo::WithOrga("library".into(), "nginx".into())),
            ),
            (
                "ghcr.io/library/nginx",
                Ok(Repo::WithServer(
                    "ghcr.io".into(),
                    "library".into(),
                    "nginx".into(),
                )),
            ),
            (
                "te-st/test-hypen",
                Ok(Repo::WithOrga("te-st".into(), "test-hypen".into())),
            ),
            (
                "test/test.dot",
                Ok(Repo::WithOrga("test".into(), "test.dot".into())),
            ),
        ];

        for i in input {
            assert_eq!(super::split_repo_without_tag(i.0), i.1);
        }
    }

    #[test]
    fn test_match_yaml_image() {
        let input: Vec<(&str, Result<(&str, &str), Error>)> = vec![
            ("", Err(Error::NoTagFound)),
            ("version: '2'", Err(Error::NoTagFound)),
            ("image: ", Err(Error::NoTagFound)),
            ("  image: ", Err(Error::NoTagFound)),
            ("  image: nginx", Ok(("  image: ", "nginx"))),
            ("  image: library/nginx", Ok(("  image: ", "library/nginx"))),
            (
                "  image: gchr.io/library/nginx",
                Ok(("  image: ", "gchr.io/library/nginx")),
            ),
            ("  image: nginx # comment", Ok(("  image: ", "nginx"))),
            ("  image: test-hyphen", Ok(("  image: ", "test-hyphen"))),
            ("  image: test.dot", Ok(("  image: ", "test.dot"))),
        ];

        for i in input {
            assert_eq!(super::match_yaml_image(i.0), i.1);
        }
    }

    #[test]
    fn test_split_tag_from_repo() {
        let input: Vec<(&str, Result<(&str, &str), super::Error>)> = vec![
            ("nginx", Ok(("nginx", ""))),
            ("library/nginx", Ok(("library/nginx", ""))),
            ("ghcr.io/library/nginx", Ok(("ghcr.io/library/nginx", ""))),
            ("nginx:", Ok(("nginx", ""))),
            ("nginx:1", Ok(("nginx", "1"))),
            ("nginx:latest", Ok(("nginx", "latest"))),
            ("hy-phen:latest", Ok(("hy-phen", "latest"))),
            ("test.dot:latest", Ok(("test.dot", "latest"))),
            (
                "woodpeckerci/woodpecker-server",
                Ok(("woodpeckerci/woodpecker-server", "")),
            ),
        ];

        for i in input {
            assert_eq!(super::split_tag_from_repo(i.0), i.1);
        }
    }
}
