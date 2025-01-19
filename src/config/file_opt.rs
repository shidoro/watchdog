use super::find_root;
use ignore::gitignore::{Gitignore, GitignoreBuilder};
use serde::de::Visitor;
use serde::Deserialize;
use std::{fmt::Debug, path::Path};
use std::{fs, path::PathBuf};

#[derive(Debug, Default, Deserialize)]
pub struct FileOpt {
    #[serde(default)]
    exec: Option<FileOptExec>,
    #[serde(default)]
    exec_pre: Option<FileOptExecPre>,
    #[serde(default)]
    exclude: Option<FileOptExclude>,
    #[serde(default)]
    extend: Option<FileOptExtend>,
}

impl FileOpt {
    pub fn parse() -> Self {
        let root = find_root();
        if root.is_err() {
            return Self::default();
        }

        let root = root.unwrap();
        let contents = fs::read_to_string(root.join("watchdog.toml")).map_err(|err| {
            format!("An error occured while reading watchdog.toml config file: {err}")
        });

        if contents.is_err() {
            return Self::default();
        }

        let file_opt: Result<FileOpt, _> = toml::from_str(&contents.unwrap()).inspect_err(|err| {
            eprintln!(
                "An error occurred while deserialising watchdog.toml: {}",
                err.message()
            )
        });

        if file_opt.is_err() {
            return Self::default();
        }

        file_opt.unwrap()
    }

    pub fn take_exec(&mut self) -> Option<FileOptExec> {
        self.exec.take()
    }

    pub fn take_exec_pre(&mut self) -> Option<FileOptExecPre> {
        self.exec_pre.take()
    }

    pub fn take_exclude(&mut self) -> Option<FileOptExclude> {
        self.exclude.take()
    }

    pub fn take_extend(&mut self) -> Option<FileOptExtend> {
        self.extend.take()
    }
}

#[derive(Debug, Deserialize)]
pub struct FileOptExec {
    command: Option<String>,
    args: Option<Vec<String>>,
    origin: Option<PathBuf>,
}

impl FileOptExec {
    pub fn take_command(&mut self) -> Option<String> {
        self.command.take()
    }

    pub fn take_args(&mut self) -> Option<Vec<String>> {
        self.args.take()
    }

    pub fn take_origin(&mut self) -> Option<PathBuf> {
        self.origin.take()
    }
}

#[derive(Debug, Default, Deserialize)]
pub struct FileOptExecPre {
    #[serde(default)]
    origin: Option<PathBuf>,
    #[serde(default)]
    when: Option<FileOptWhen>,
    commands: Option<Vec<FileOptExecPreCommand>>,
}

impl FileOptExecPre {
    pub fn take_when(&mut self) -> Option<FileOptWhen> {
        self.when.take()
    }

    pub fn take_commands(&mut self) -> Option<Vec<FileOptExecPreCommand>> {
        self.commands.take()
    }

    pub fn take_origin(&mut self) -> Option<PathBuf> {
        self.origin.take()
    }
}

#[derive(Debug, Deserialize)]
pub enum FileOptWhen {
    #[serde(rename = "once")]
    Once,
    #[serde(rename = "always")]
    Always,
}

#[derive(Debug, Default, Deserialize)]
pub struct FileOptExecPreCommand {
    command: Option<String>,
    args: Option<Vec<String>>,
}

impl FileOptExecPreCommand {
    pub fn take_command(&mut self) -> Option<String> {
        self.command.take()
    }

    pub fn take_args(&mut self) -> Option<Vec<String>> {
        self.args.take()
    }
}

#[derive(Debug, Default, Deserialize)]
pub struct FileOptExclude {
    files: Vec<IgnorablePath>,
}

impl FileOptExclude {
    pub fn take_exclude_files(self) -> Vec<String> {
        self.files
            .into_iter()
            .map(IgnorablePath::take_path)
            .collect()
    }
}

#[derive(Debug, Deserialize)]
struct IgnorablePath {
    path: String,
}

impl IgnorablePath {
    fn take_path(self) -> String {
        self.path
    }
}

pub trait Extendable: Debug {
    fn matcher(&self, path: &Path, is_dir: bool) -> bool;
}

#[derive(Debug, Deserialize)]
#[serde(tag = "extendable_type", content = "path")]
pub enum ExtendableType {
    #[serde(rename = "git")]
    Git(GitignoreSerde),
}

impl Extendable for ExtendableType {
    fn matcher(&self, path: &Path, is_dir: bool) -> bool {
        match self {
            ExtendableType::Git(wrapper) => wrapper
                .0
                .matched_path_or_any_parents(path, is_dir)
                .is_ignore(),
        }
    }
}

#[derive(Debug, Default, Deserialize)]
pub struct FileOptExtend {
    extendables: Vec<ExtendableType>,
}

impl FileOptExtend {
    pub fn take_extendables(self) -> Vec<ExtendableType> {
        self.extendables
    }
}

#[derive(Debug)]
pub struct GitignoreSerde(pub Gitignore);

impl<'de> serde::Deserialize<'de> for GitignoreSerde {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct GitignorePathVisitor;

        impl Visitor<'_> for GitignorePathVisitor {
            type Value = GitignoreSerde;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("a path string to a .gitignore file")
            }

            fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                let root = find_root().unwrap_or_default();
                let mut builder = GitignoreBuilder::new(&root);
                let err = builder.add(root.join(v));
                if err.is_some() {
                    eprintln!("Something went wrong with adding '{v}' to the path {root:?}. GitignoreBuilder will now be empty.\nError: {err:?}")
                }
                let git = builder
                    .build()
                    .map_err(|err| E::custom(format!("Failed to build Gitignore: {err}")))?;

                Ok(GitignoreSerde(git))
            }
        }

        deserializer.deserialize_str(GitignorePathVisitor)
    }
}
