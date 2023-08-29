use nom::{IResult, error::context, bytes::complete::tag, combinator::{opt, eof}, branch::alt, sequence::{preceded, tuple}};
use nom_regex::str::re_match;
use regex::Regex;

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
        preceded(
            opt(tag(":")),
            re_match(Regex::new(r#"[a-zA-Z0-9!%@^_#"$&'()*\-+,;<=>?\[\]{|}~/.]+"#).unwrap())
        )
    )(target)
}

fn package(pkg: &str) -> IResult<&str, (Option<&str>, &str)> {
    context(
        "package",
        tuple((
            opt(alt((
                tag("!/"),
                tag("//")
            ))),
            re_match(Regex::new(r"[A-Za–z0–9/\-.@_]+").unwrap())
        ))
    )(pkg)
}

fn repository(repo: &str) -> IResult<&str, &str> {
    context(
        "repository",
        preceded(
            tag("@"),
            re_match(Regex::new(r"[A-Za–z0–9/\-._]+").unwrap())
        )
    )(repo)
}

fn nom_error(message: &str, typ: nom::error::ErrorKind) -> nom::Err<nom::error::Error<&str>> {
    nom::Err::Error(
        nom::error::Error::new(
            message,
            typ
        )
    )
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
    pub fn new<S: AsRef<str>>(label: S, current_repository: S, current_package: S) -> Result<Self, String> {
        Ok(
            Self::parse(label.as_ref(), current_repository.as_ref(), current_package.as_ref()).map_err(|e| e.to_string())?.1
        )
    }

    /// Same as [`Label::new`] except it returns a [`Result`].
    fn parse<'a>(label: &'a str, current_repository: &'a str, current_package: &'a str) -> IResult<&'a str, Self> {       
        let (i, repository) = opt(repository)(label)?;
        let (i, separator_package) = opt(package)(i)?;
        let (i, target) = opt(target)(i)?;
        let _ = eof(i)?;

        if None == separator_package && None == target {
            return Err(
                nom_error(
                    "Package or target must be specified.",
                    nom::error::ErrorKind::Verify
                )
            )
        }
        

        let target = target.unwrap_or(
            match separator_package.unwrap_or((None, current_package)).1.split("/").last() {
                Some(pkg_name) => pkg_name,
                None => return Err(
                    nom_error(
                        "Cannot be missing target if package is not specified and the current package is the root of a repository.",
                        nom::error::ErrorKind::Verify
                    )
                )
            }
        );

        let exact = separator_package.unwrap_or((
            None,
            ""
        )).0.unwrap_or("") == "!/";
        
        Ok(
            (
                i,
                Self {
                    repository: repository.unwrap_or(current_repository).into(),
                    package: separator_package.unwrap_or((None, current_package)).1.into(),
                    target: target.into(),
                    exact: exact
                }
            )
        )
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
            Label::new("a_dir:a_file", "default_package", "cur_dir").unwrap(),
            Label {
                repository: "default_package".to_owned(),
                package: "cur_dir/a_dir".to_owned(),
                target: "a_file".to_owned(),
                exact: false
            }
        );
    }

    #[test]
    fn fully_qualifed_path() {
        assert_eq!(
            Label::new(
                "@another_package!/different_dir/../another_dir:another_file",
                "default_package",
                "cur_dir"
            ).unwrap(),
            Label {
                repository: "another_package".to_owned(),
                package: "/another_dir".to_owned(),
                target: "another_file".to_owned(),
                exact: true
            }
        );
    }
}
