use normalize_path::NormalizePath;
use std::path::PathBuf;

#[derive(Debug, PartialEq, Eq)]
pub struct Label {
    pub repository: String,
    pub package: String,
    pub target: String,
    pub exact: bool,
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
    pub fn new<S: AsRef<str>>(label: S, current_repository: S, current_package: S) -> Self {
        Self::parse(label, current_repository, current_package).unwrap()
    }

    /// Same as [`Label::new`] except it returns a [`Result`].
    pub fn parse<S: AsRef<str>>(label: S, current_repository: S, current_package: S) -> Result<Self, String> {
        if let Some(caps) = regex::Regex::new("^((?:@[a-zA-Z_-]+)?)([!/])(/[^:]+)?:?([^:/]+)$")
            .unwrap()
            .captures(label.as_ref())
        {
            Ok(
                Self {
                    repository: {
                        if caps[1].starts_with("@") {
                            caps[1][1..].into()
                        } else {
                            current_repository.as_ref().into()
                        }
                    },
                    package: {
                        if caps[3].len() > 0 {
                            PathBuf::from(caps[3].to_owned())
                                .normalize()
                                .to_str()
                                .unwrap()
                                .into()
                        } else {
                            current_package.as_ref().into()
                        }
                    },
                    target: caps[4].into(),
                    exact: &caps[2] == "!",
                }
            )
        } else {
            Err(format!("Invalid label {}", label.as_ref()))
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::label::Label;

    #[test]
    fn relative_path() {
        assert_eq!(
            Label::new("a_dir:a_file", "default_package", "cur_dir"),
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
            ),
            Label {
                repository: "another_package".to_owned(),
                package: "/another_dir".to_owned(),
                target: "another_file".to_owned(),
                exact: true
            }
        );
    }
}
