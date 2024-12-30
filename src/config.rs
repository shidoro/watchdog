use ignore::gitignore::{Gitignore, GitignoreBuilder};
use serde::de::Visitor;
use serde::Deserialize;
use std::{
    env::{current_dir, set_current_dir},
    error::Error,
    fs,
    io::{Error as IoError, ErrorKind as IoErrorKind},
    path::PathBuf,
    process::Command,
};
use std::{fmt::Debug, path::Path};

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

#[derive(Debug, Deserialize)]
pub struct Config {
    extend: Extend,
    exclude: Exclude,
    #[serde(default)]
    root: PathBuf,
    run: Run,
    build: Build,
}

impl Config {
    pub fn to_extend(&self) -> &Extend {
        &self.extend
    }

    pub fn to_exclude(&self) -> &Exclude {
        &self.exclude
    }

    pub fn root(&self) -> &PathBuf {
        &self.root
    }

    pub fn to_run(&self) -> &Run {
        &self.run
    }

    pub fn to_build(&self) -> &Build {
        &self.build
    }

    fn canonicalise(&mut self) {
        self.build.canonicalise(&self.root);
        self.run.canonicalise(&self.root);
    }

    fn setup(&mut self, root: PathBuf) -> Result<(), Box<dyn Error>> {
        let ignore_files: Vec<String> = self
            .exclude
            .files
            .iter()
            .map(|ignorable_path| ignorable_path.path.to_owned())
            .collect();

        self.root = root;
        let _ = set_current_dir(self.root());
        self.canonicalise();
        self.exclude.set_exclude_files(ignore_files);

        Ok(())
    }
}

#[derive(Debug, Deserialize)]
pub struct Extend {
    extendables: Vec<ExtendableType>,
}

impl Extend {
    pub fn to_extendables(&self) -> &Vec<ExtendableType> {
        &self.extendables
    }
}

#[derive(Debug, Deserialize)]
pub struct Exclude {
    files: Vec<IgnorablePath>,
    #[serde(default)]
    exclude_files: Vec<String>,
}

impl Exclude {
    pub fn to_exclude_files(&self) -> &Vec<String> {
        &self.exclude_files
    }

    pub fn set_exclude_files(&mut self, ignore_files: Vec<String>) {
        self.exclude_files = ignore_files;
    }
}

#[derive(Debug, Deserialize)]
struct IgnorablePath {
    path: String,
}

#[derive(Debug, Deserialize)]
pub struct Run {
    command: String,
    args: Vec<String>,
    #[serde(default)]
    precompile: bool,
    #[serde(default)]
    origin: PathBuf,
}

impl Run {
    pub fn command(&self) -> &str {
        &self.command
    }

    pub fn args(&self) -> &Vec<String> {
        &self.args
    }

    pub fn precompile(&self) -> bool {
        self.precompile
    }

    pub fn origin(&self) -> &PathBuf {
        &self.origin
    }

    fn canonicalise(&mut self, root: &Path) {
        let origin = fs::canonicalize(root.join(self.origin())).map_err(|err| {
            eprintln!(
                "Error while canonicalising run origin ({:?}): {err:?}",
                self.origin()
            );
            err
        });

        if let Ok(origin) = origin {
            self.origin = origin;
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct Build {
    command: String,
    args: Vec<String>,
    #[serde(default)]
    origin: PathBuf,
}

impl Build {
    pub fn command(&self) -> &str {
        &self.command
    }

    pub fn args(&self) -> &Vec<String> {
        &self.args
    }

    pub fn origin(&self) -> &PathBuf {
        &self.origin
    }

    fn canonicalise(&mut self, root: &Path) {
        let origin = fs::canonicalize(root.join(self.origin())).map_err(|err| {
            eprintln!(
                "Error while canonicalising build origin ({:?}): {err:?}",
                self.origin()
            );
            err
        });

        if let Ok(origin) = origin {
            self.origin = origin;
        }
    }
}

pub fn load_config() -> Result<Config, Box<dyn Error>> {
    let root = find_root()?;
    let contents = fs::read_to_string(root.join("watchdog.toml")).map_err(|err| {
        format!("An error occured while reading watchdog.toml config file: {err}")
    })?;
    let mut config: Config = toml::from_str(&contents).inspect_err(|err| {
        eprintln!(
            "An error occurred while deserialising watchdog.toml: {}",
            err.message()
        )
    })?;

    config.setup(root)?;
    Ok(config)
}

fn find_root() -> Result<PathBuf, Box<dyn Error>> {
    let mut root_path = None;
    if let Ok(git_root_path) = Command::new("git")
        .args(["rev-parse", "--show-toplevel"])
        .output()
    {
        root_path = Some(std::str::from_utf8(&git_root_path.stdout)?.to_owned())
    } else if let Ok(cargo_root_path) = current_dir() {
        root_path = cargo_root_path.to_str().map(|s| s.into());
    }

    let root_path = root_path.ok_or_else(|| {
        Box::new(IoError::new(
            IoErrorKind::NotFound,
            "Could not find the root project",
        ))
    })?;

    Ok(PathBuf::from(root_path.trim()))
}
