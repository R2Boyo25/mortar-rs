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
    /// assert_eq!(Label::new("@package_name//abc:something", "abc", "."), Label {package: "package_name".to_owned(), dir: "/abc".to_owned(), target: "something".to_owned(), exact: false});
    /// ```
    /// ```
    /// use mortar::label::Label;
    ///
    /// assert_eq!(
    ///     Label::new("!something:abc", "test", "a_dir"),
    ///     Label {package: "test".to_owned(), dir: "a_dir/something".to_owned(), target: "abc".to_owned(), exact: true}
    /// )
    /// ```
    pub fn new<S: AsRef<str>>(label: S, current_repository: S, current_package: S) -> Self {
        Self::parse(path, cur_package, cur_dir).unwrap()
    }

    /// Same as [`Label::new`] except it returns a [`Result`].
    pub fn parse<S: AsRef<str>>(label: S, current_repository: S, current_package: S) -> Result<Self, &'static str> {
        if let Some(caps) = regex::Regex::new("^((?:@[a-zA-Z_-]+)?)([!/])(/[^:]+):([^:/]+)$")
            .unwrap()
            .captures(path.as_ref())
        {
            Ok(Self {
                package: {
                    if caps[1].starts_with("@") {
                        caps[1][1..].to_owned()
                    } else {
                        cur_package.as_ref().to_owned()
                    }
                },
                dir: {
                    if caps[3].len() > 0 {
                        PathBuf::from(caps[3].to_owned())
                            .normalize()
                            .to_str()
                            .unwrap()
                            .to_owned()
                    } else {
                        "/".to_owned()
                    }
                },
                target: { caps[4].to_owned() },
                exact: &caps[2] == "!",
            })
        } else if let Some(caps) = regex::Regex::new("^(!?)((?:(?:.?/)?[^:]+)?):([^:/]+)$")
            .unwrap()
            .captures(path.as_ref())
        {
            println!("oi");

            let mut pb = PathBuf::new();

            println!("{}", &caps[2]);

            cur_dir
                .as_ref()
                .to_owned()
                .split("/")
                .map(|x| pb.push(x))
                .for_each(drop);
            caps[2]
                .to_owned()
                .split("/")
                .map(|x| pb.push(x))
                .for_each(drop);

            Ok(Self {
                package: cur_package.as_ref().to_owned(),
                dir: pb.normalize().to_str().unwrap().to_owned(),
                target: caps[3].to_owned(),
                exact: &caps[1] == "!",
            })
        } else {
            Err("WTF MAN, THAT'S INVALID.")
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
                package: "default_package".to_owned(),
                dir: "cur_dir/a_dir".to_owned(),
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
                package: "another_package".to_owned(),
                dir: "/another_dir".to_owned(),
                target: "another_file".to_owned(),
                exact: true
            }
        );
    }
}
