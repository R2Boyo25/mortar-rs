use std::{
    path::{Path, PathBuf},
    process::Command,
};

pub mod r#ref;
use r#ref::{Mapping, Reference};

type EnvironmentID = uuid::Uuid;

/// # [[Environment]] - Command runner state using Proot
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

    /// Returns a list of commands needed to initialize the environment.
    /// The last command will always be the proot command.
    pub fn as_commands(&self) -> Vec<Command> {
        let mut init_commands: Vec<Command> = vec![];
        let mut args: Vec<String> = vec![];
        let mut command = Command::new("proot");

        self.inputs.iter().for_each(|file| {
            // If a mapping is read only, it must be mounted with `bindfs,` not `proot.`
            if file.read_only {
                init_commands.push(file.as_bind(&self.loc));
            } else {
                args.push("-b".to_string());
                args.push(file.to_string());
            }
        });

        command
            .arg("-r")
            .arg(self.loc.to_str().unwrap())
            .arg("-w")
            .arg("/")
            .args(args);

        init_commands.push(command);
        init_commands
    }

    /// Returns a reference to a file in this environment.
    pub fn get_reference(&self, path: &Path) -> Reference {
        Reference::new(Some(self.id), path)
    }

    /// Returns a list of commands needed to run a command in the environment.
    pub fn run_command(&self, command: &mut Command) -> Vec<Command> {
        let mut commands = self.as_commands();

        commands
            .last_mut()
            .unwrap()
            .arg(command.get_program())
            .args(command.get_args());

        commands
    }
}

#[cfg(test)]
mod tests {
    use super::r#ref::Mapping;
    use crate::env::{r#ref::Reference, Environment};
    use std::{path::PathBuf, process::Command};

    /// Creates the common environment for the rest of testing.
    fn create_env() -> Environment {
        Environment::new(
            PathBuf::from("/tmp/a b"),
            vec![
                Mapping::from_fs(&PathBuf::from("/tmp/b/3 3"), None, false),
                Mapping::from_fs(&PathBuf::from("/tmp/c/7"), None, true),
            ],
        )
    }

    #[test]
    fn as_commands() {
        assert_eq!(
            create_env()
                .as_commands()
                .iter()
                .map(|v| format!("{:?}", v))
                .collect::<Vec<_>>(),
            vec![
                "\"bindfs\" \"--no-allow-other\" \"-r\" \"/tmp/c/7\" \"/tmp/c/7\"".to_string(),
                "\"proot\" \"-r\" \"/tmp/a b\" \"-w\" \"/\" \"-b\" \"/tmp/b/3 3\"".to_string(),
            ]
        );
    }

    #[test]
    fn get_reference() {
        let env = create_env();

        assert_eq!(
            env.get_reference(&PathBuf::from("/smth")),
            Reference::new(Some(env.id), &PathBuf::from("/smth"))
        );
    }

    #[test]
    fn run_command() {
        let a = format!(
            "{:?}",
            Environment::new(
                PathBuf::from("/tmp/a"),
                vec![Mapping::from_fs(&PathBuf::from("/nix"), None, false)]
            )
            .run_command(Command::new("echo").arg("Hello, World!"))
            .last()
            .unwrap()
        );

        assert_eq!(
            a,
            "\"proot\" \"-r\" \"/tmp/a\" \"-w\" \"/\" \"-b\" \"/nix\" \"echo\" \"Hello, World!\""
        )
    }
}
