use nom::{
    branch::alt,
    bytes::complete::tag,
    combinator::{all_consuming, eof, map, opt},
    error::context,
    sequence::{pair, preceded, terminated, tuple},
    IResult,
};
use nom_regex::str::re_match;
use regex::Regex;
use std::collections::HashMap;

#[derive(Debug, PartialEq, Eq)]
pub struct Label {
    pub repository: String,
    pub package: String,
    pub target: String,
    pub exact: bool,
}

fn target(target: &str) -> IResult<&str, &str> {
    context(
        "target",
        re_match(Regex::new(r#"[a-zA-Z0-9!%@^_#"$&'()*\-+,;<=>?\[\]{|}~/. ]+"#).unwrap()),
    )(target)
}

fn package(pkg: &str) -> IResult<&str, &str> {
    context(
        "package",
        re_match(Regex::new(r"[A-Za–z0–9/\-.@_]+").unwrap()),
    )(pkg)
}

fn repository(repo: &str) -> IResult<&str, &str> {
    context(
        "repository",
        re_match(Regex::new(r"[A-Za–z0–9/\-._]+").unwrap()),
    )(repo)
}

fn root(input: &str) -> IResult<&str, &str> {
    alt((tag("//"), tag("!/")))(input)
}

fn nom_error(message: &str, typ: nom::error::ErrorKind) -> nom::Err<nom::error::Error<&str>> {
    nom::Err::Error(nom::error::Error::new(message, typ))
}

impl Label {
    /// Creates a new [`Label`].
    ///
    /// # Examples
    ///
    /// ```
    /// use mortar::label::Label;
    ///
    /// assert_eq!(Label::new("@package_name//abc:something", "abc", "."), Label {repository: "package_name".to_owned(), package: "/abc".to_owned(), target: "something".to_owned(), exact: false});
    /// ```
    /// ```
    /// use mortar::label::Label;
    ///
    /// assert_eq!(
    ///     Label::new("!something:abc", "test", "a_dir"),
    ///     Label {repository: "test".to_owned(), package: "a_dir/something".to_owned(), target: "abc".to_owned(), exact: true}
    /// )
    /// ```
    pub fn new<S: AsRef<str>>(
        label: S,
        current_repository: S,
        current_package: S,
    ) -> Result<Self, String> {
        Ok(Self::parse(
            label.as_ref(),
            current_repository.as_ref(),
            current_package.as_ref(),
        )
        .map_err(|e| e.to_string())?
        .1)
    }

    /// Same as [`Label::new`] except it returns a [`Result`].
    fn parse<'a>(
        label: &'a str,
        current_repository: &'a str,
        current_package: &'a str,
    ) -> IResult<&'a str, Self> {
        let hmap: HashMap<&str, &str> = terminated(
            alt((
                map(
                    all_consuming(preceded(
                        tag("@"),
                        tuple((
                            repository,
                            root,
                            alt((
                                map(package, |parsed_package| {
                                    HashMap::from([("package", parsed_package)])
                                }),
                                map(preceded(tag(":"), target), |parsed_target| {
                                    HashMap::from([("target", parsed_target)])
                                }),
                                map(
                                    pair(package, preceded(tag(":"), target)),
                                    |(parsed_package, parsed_target)| {
                                        HashMap::from([
                                            ("package", parsed_package),
                                            ("target", parsed_target),
                                        ])
                                    },
                                ),
                            )),
                        )),
                    )),
                    |(parsed_repository, parsed_separator, parsed_package_and_or_target)| {
                        let mut m = HashMap::from([
                            ("repository", parsed_repository),
                            ("separator", parsed_separator),
                        ]);
                        m.extend(parsed_package_and_or_target);
                        m
                    },
                ),
                map(
                    all_consuming(preceded(
                        tag("@"),
                        tuple((repository, root, package, opt(preceded(tag(":"), target)))),
                    )),
                    |(parsed_repository, parsed_separator, parsed_package, parsed_target)| {
                        let mut m = HashMap::from([
                            ("repository", parsed_repository),
                            ("separator", parsed_separator),
                            ("package", parsed_package),
                        ]);
                        parsed_target.map(|v| m.insert("target", v));
                        m
                    },
                ),
                map(
                    all_consuming(preceded(opt(tag(":")), target)),
                    |parsed_target| HashMap::from([("target", parsed_target)]),
                ),
                map(
                    all_consuming(tuple((root, package, opt(preceded(tag(":"), target))))),
                    |(parsed_separator, parsed_package, parsed_target)| {
                        let mut m = HashMap::from([
                            ("separator", parsed_separator),
                            ("package", parsed_package),
                        ]);
                        parsed_target.map(|v| m.insert("target", v));
                        m
                    },
                ),
            )),
            eof,
        )(label)?
        .1;

        println!("{:?}", hmap);

        let repo = hmap.get("repository").unwrap_or(&current_repository);
        let target: Result<&str, nom::Err<nom::error::Error<&str>>> = match hmap.get("target") {
            Some(v) => Ok(v),
            None => {
                if repo.len() == 0 || repo.split("/").count() == 0 {
                    Err(nom_error("Target must be explicitly specified as it cannot be inferred from an empty package.",
                                  nom::error::ErrorKind::Verify))
                } else {
                    Ok(&repo.split("/").last().unwrap())
                }
            }
        };

        Ok((
            "",
            Self {
                repository: repo.to_string(),
                package: hmap.get("package").unwrap_or(&current_package).to_string(),
                target: target?.to_string(),
                exact: hmap.get("separator").unwrap_or(&"//") == &"!/",
            },
        ))
    }
}

#[cfg(test)]
mod tests {
    use crate::label::Label;

    #[test]
    fn lone_target() {
        assert_eq!(
            Label::new(":a_target", "default_repo", "default_package").unwrap(),
            Label {
                repository: "default_repo".to_owned(),
                package: "default_package".to_owned(),
                target: "a_target".to_owned(),
                exact: false
            }
        );
    }

    #[test]
    fn relative_path() {
        assert_eq!(
            Label::new("//a_package:a_target", "default_repo", "current_package").unwrap(),
            Label {
                repository: "default_repo".to_owned(),
                package: "a_package".to_owned(),
                target: "a_target".to_owned(),
                exact: false
            }
        );
    }

    #[test]
    fn fully_qualifed_path() {
        assert_eq!(
            Label::new(
                "@another_repo//different_package:another_target",
                //"@another_package!/different_dir/../another_dir:another_file",
                "default_repo",
                "current_package"
            )
            .unwrap(),
            Label {
                repository: "another_repo".to_owned(),
                package: "another_package".to_owned(),
                target: "another_target".to_owned(),
                exact: true
            }
        );
    }
}
