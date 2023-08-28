use normalize_path::NormalizePath;
use std::path::PathBuf;

#[derive(Debug, PartialEq, Eq)]
pub struct Path {
    pub package: String,
    pub dir: String,
    pub target: String,
    pub exact: bool,
}

impl Path {
    /// Creates a new [`Path`].
    ///
    /// # Examples
    ///
    /// ```
    /// use mortar::path::Path;
    ///
    /// assert_eq!(Path::new("@package_name//abc:something", "abc", "."), Path {package: "package_name".to_owned(), dir: "/abc".to_owned(), target: "something".to_owned(), exact: false});
    /// ```
    /// ```
    /// use mortar::path::Path;
    ///
    /// assert_eq!(
    ///     Path::new("!something:abc", "test", "a_dir"),
    ///     Path {package: "test".to_owned(), dir: "a_dir/something".to_owned(), target: "abc".to_owned(), exact: true}
    /// )
    /// ```
    pub fn new<S: AsRef<str>>(path: S, cur_package: S, cur_dir: S) -> Self {
        Self::parse(path, cur_package, cur_dir).unwrap()
    }

    /// Same as [`Path::new`] except it returns a [`Result`].
    pub fn parse<S: AsRef<str>>(path: S, cur_package: S, cur_dir: S) -> Result<Self, &'static str> {
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
    use crate::path::Path;

    #[test]
    fn relative_path() {
        assert_eq!(
            Path::new("a_dir:a_file", "default_package", "cur_dir"),
            Path {
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
            Path::new(
                "@another_package!/different_dir/../another_dir:another_file",
                "default_package",
                "cur_dir"
            ),
            Path {
                package: "another_package".to_owned(),
                dir: "/another_dir".to_owned(),
                target: "another_file".to_owned(),
                exact: true
            }
        );
    }
}
