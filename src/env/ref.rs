use crate::env::EnvironmentID;
use std::{
    path::{Path, PathBuf},
    process::Command,
};

/// References reference locations within [[Environment]]s
///
/// If [[self.env]] is [[None]], the Reference is directly to a FS location.
#[derive(PartialEq, Debug)]
pub struct Reference {
    pub env: Option<EnvironmentID>,
    pub file: PathBuf,
}

impl Reference {
    pub fn new(env: Option<EnvironmentID>, file: &Path) -> Self {
        Self {
            env,
            file: file.to_owned(),
        }
    }

    /// Gets the real FS path for a file, given the env
    pub fn real_path(&self) -> String {
        if let Some(env) = self.env {
            todo!();
        } else {
            self.file.to_str().unwrap().to_string()
        }
    }
}

/// Mappings map an input Reference to an output location
///
/// If [[self.alias]] is [[None]], then the alias is just [[self.from.file]].
#[derive(PartialEq, Debug)]
pub struct Mapping {
    pub from: Reference,
    pub alias: Option<PathBuf>,
    pub read_only: bool,
}

impl Mapping {
    pub fn to_string(&self) -> String {
        // Handle paths with colons.
        let mut tmp = self.from.real_path().replace(':', "\\:");

        if let Some(alias) = self.alias.clone() {
            tmp.push_str(":");
            tmp.push_str(alias.to_str().unwrap());
        }

        tmp
    }

    /// Shortcut for creating Mappings from the filesystem.
    pub fn from_fs(from: &Path, alias: Option<&Path>, read_only: bool) -> Self {
        Self {
            from: Reference::new(None, from),
            alias: alias.map(|v| v.to_path_buf()),
            read_only,
        }
    }

    /// Shortcut to create a mapping from a [[Reference]]. This is the [[new()]] method.
    pub fn from_reference(from: Reference, alias: Option<&Path>, read_only: bool) -> Self {
        Self {
            from,
            alias: alias.map(|v| v.to_path_buf()),
            read_only,
        }
    }

    /// Converts this mapping into a read-only bindfs mount command.
    pub fn as_bind(&self, out_dir: &Path) -> Command {
        let mut mount_path = out_dir.to_path_buf();

        mount_path.push(if let Some(alias) = self.alias.clone() {
            alias
        } else {
            self.from.file.clone()
        });

        let mut com = Command::new("bindfs");

        com.arg("--no-allow-other")
            .arg("-r")
            .arg(self.from.real_path())
            .arg(mount_path);

        com
    }
}

#[cfg(test)]
pub mod tests {
    use super::Mapping;
    use std::path::PathBuf;

    #[test]
    pub fn yes() {
        assert_eq!(
            Mapping::from_fs(&PathBuf::from("/a:a"), Some(&PathBuf::from("/b")), true).to_string(),
            "/a\\:a:/b"
        );
    }

    #[test]
    pub fn get_binding() {
        assert_eq!(
            format!(
                "{:?}",
                Mapping::from_fs(&PathBuf::from("/a:a"), Some(&PathBuf::from("/b")), true)
                    .as_bind(&PathBuf::from("/test"))
            ),
            "\"bindfs\" \"--no-allow-other\" \"-r\" \"/a:a\" \"/b\""
        );
    }
}
