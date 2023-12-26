use std::path::{Path, PathBuf};

pub mod r#ref;
use r#ref::{Mapping, Reference};

type EnvironmentID = uuid::Uuid;

/// # [[Environment]] - Command runner state
#[derive(PartialEq, Debug)]
pub struct Environment {
    pub id: EnvironmentID,
    pub loc: PathBuf,
    pub inputs: Vec<Mapping>,
}

impl Environment {
    pub fn new(location: PathBuf, inputs: Vec<Mapping>) -> Self {
        Self {
            id: EnvironmentID::new_v4(),
            loc: location,
            inputs: inputs,
        }
    }

    pub fn as_command(&self) -> Command {
        let mut command = "proot".to_string();

        // Paths may have quotes and/or spaces
        command.push_str(" -r \"");
        command.push_str(&self.loc.to_str().unwrap().replace('\"', "\\\""));
        command.push_str("\"");

        self.inputs.iter().for_each(|file| {
            // Ditto, but the quotes are already sanitized in `to_string()`.
            command.push_str(" -b \"");
            command.push_str(&file.to_string());
            command.push_str("\"");
        });

        command
    }

    pub fn get_reference(&self, path: &Path) -> Reference {
        Reference::new(Some(self.id), path)
    }
}

#[cfg(test)]
mod tests {
    use crate::env::{r#ref::Reference, Environment};
    use std::path::PathBuf;

    use super::r#ref::Mapping;

    fn create_env() -> Environment {
        Environment::new(
            PathBuf::from("/tmp/a \"b"),
            vec![
                Mapping::from_fs(&PathBuf::from("/tmp/b/3 3"), None),
                Mapping::from_fs(&PathBuf::from("/tmp/c/\"7"), None),
            ],
        )
    }

    #[test]
    fn init() {
        assert_eq!(create_env().as_command(), "proot -r \"/tmp/a \\\"b\" -b \"/tmp/b/3 3\" -b \"/tmp/c/\\\"7\"".to_string());
    }

    #[test]
    fn idk() {
        let env = create_env();

        assert_eq!(
            env.get_reference(&PathBuf::from("/smth")),
            Reference::new(Some(env.id), &PathBuf::from("/smth"))
        );
    }
}
