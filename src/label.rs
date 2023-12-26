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
use std::{collections::HashMap, path::PathBuf, str::FromStr};

#[derive(Debug, PartialEq, Eq)]
pub struct Label {
    pub repository: String,
    pub package: String,
    pub target: String,
}

fn nom_error(message: &str, typ: nom::error::ErrorKind) -> nom::Err<nom::error::Error<&str>> {
    nom::Err::Error(nom::error::Error::new(message, typ))
}

fn fre_match(re: &fancy_regex::Regex) -> impl Fn(&str) -> IResult<&str, &str> + '_ {
    move |i: &str| {
        if let Ok(Some(mat)) = re.find(i) {
            Ok((&i[mat.end()..], &i[mat.start()..mat.end()]))
        } else {
            Err(nom_error(
                "Didn't match",
                nom::error::ErrorKind::RegexpMatch,
            ))
        }
    }
}

fn target(target: &str) -> IResult<&str, &str> {
    context(
        "target",
        re_match(Regex::new(r#"^[a-zA-Z0-9!%@^_#"$&'()*\-+,;<=>?\[\]{|}~/. ]+"#).unwrap()),
    )(target)
}

fn package(pkg: &str) -> IResult<&str, &str> {
    context(
        "package",
        fre_match(&fancy_regex::Regex::new(r"^[^\n$:]+?(?=:(?:.|\s)+$|$)").unwrap()),
    )(pkg)
}

fn repository(repo: &str) -> IResult<&str, &str> {
    context(
        "repository",
        re_match(Regex::new(r"^[-A-Za-z0-9\/._]+").unwrap()),
    )(repo)
}

fn root(input: &str) -> IResult<&str, &str> {
    alt((tag("//"), tag("!/")))(input)
}

impl Label {
    /// Creates a new [`Label`].
    ///
    /// # Examples
    ///
    /// ```
    /// use mortar::label::Label;
    ///
    /// assert_eq!(Label::new("@package_name//abc:something", "abc", "."), Ok(Label {repository: "package_name".to_owned(), package: "/abc".to_owned(), target: "something".to_owned(), exact: false}));
    /// ```
    /// ```
    /// use mortar::label::Label;
    ///
    /// assert_eq!(
    ///     Label::new("something:abc", "test", "a_dir"),
    ///     Ok(Label {repository: "test".to_owned(), package: "a_dir/something".to_owned(), target: "abc".to_owned(), exact: false})
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
                    all_consuming(tuple((opt(root), package, opt(preceded(tag(":"), target))))),
                    |(parsed_separator, parsed_package, parsed_target)| {
                        let mut m = HashMap::from([("package", parsed_package)]);

                        if let Some(sep) = parsed_separator {
                            m.insert("separator", sep);
                        }

                        parsed_target.map(|v| m.insert("target", v));
                        m
                    },
                ),
                map(
                    all_consuming(preceded(opt(tag(":")), target)),
                    |parsed_target| HashMap::from([("target", parsed_target)]),
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
                package: if hmap.contains_key("separator") && hmap.contains_key("package") {
                    hmap.get("package").unwrap_or(&current_package).to_string()
                } else if hmap.contains_key("package") {
                    hmap.get("package")
                        .map(|pkg| {
                            if current_package == "." {
                                return pkg.to_string();
                            }

                            let mut new_package = PathBuf::from_str(current_package).unwrap();
                            new_package.push(pkg);
                            new_package.to_str().unwrap().to_string()
                        })
                        .unwrap_or(current_package.to_owned())
                } else {
                    current_package.to_owned()
                },
                target: target?.to_string(),
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
            }
        );
    }

    #[test]
    fn fully_qualifed_path() {
        assert_eq!(
            Label::new(
                "@another_repo//different_package:another_target",
                "default_repo",
                "current_package"
            )
            .unwrap(),
            Label {
                repository: "another_repo".to_owned(),
                package: "another_package".to_owned(),
                target: "another_target".to_owned(),
            }
        );
    }
}
