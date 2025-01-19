mod args_opt;
mod file_opt;

use args_opt::{
    ArgsOpt, ArgsOptExec, ArgsOptExecPre, ArgsOptExtend, ArgsOptExtendableType, ArgsOptWhen,
};
use clap::Parser;
use file_opt::GitignoreSerde;
pub use file_opt::{
    Extendable, ExtendableType, FileOpt, FileOptExclude, FileOptExec, FileOptExecPre,
    FileOptExtend, FileOptWhen,
};
use ignore::gitignore::GitignoreBuilder;
use std::{
    env::current_dir,
    error::Error,
    fs,
    io::{Error as IoError, ErrorKind as IoErrorKind},
    path::PathBuf,
    process::Command,
    str::FromStr,
};
use std::{fmt::Debug, path::Path};

#[derive(Debug, Default)]
pub struct Config {
    exec: Exec,
    exec_pre: Option<ExecPre>,
    exclude: Exclude,
    extend: Extend,
    root: PathBuf,
}

impl Config {
    pub fn new() -> Result<Self, Box<dyn Error>> {
        let mut config = Self::default();
        let root = find_root()?;
        config.root = root;

        let file_opt = FileOpt::parse();
        let args_opt = ArgsOpt::parse();

        let mut config = config.merge(file_opt, args_opt);
        config.canonicalise();
        Ok(config)
    }

    fn merge(mut self, mut file_opt: FileOpt, mut args_opt: ArgsOpt) -> Self {
        self.merge_exec(file_opt.take_exec(), args_opt.take_exec());
        self.merge_exec_pre(file_opt.take_exec_pre(), args_opt.take_exec_pre());
        self.merge_exclude(file_opt.take_exclude(), args_opt.take_exclude());
        self.merge_extend(file_opt.take_extend(), args_opt.take_extend());

        self.canonicalise();

        self
    }

    fn merge_exec(&mut self, file_exec: Option<FileOptExec>, args_exec: Option<ArgsOptExec>) {
        match (file_exec, args_exec) {
            (Some(file_exec), Some(args_exec)) => {
                self.exec.merge_file_exec(file_exec);
                self.exec.merge_args_exec(args_exec);
            }
            (None, Some(args_exec)) => self.exec.merge_args_exec(args_exec),
            (Some(file_exec), None) => self.exec.merge_file_exec(file_exec),
            (None, None) => self.exec = Exec::default(),
        }
    }

    fn merge_exec_pre(
        &mut self,
        file_exec_pre: Option<FileOptExecPre>,
        args_exec_pre: Option<ArgsOptExecPre>,
    ) {
        let mut exec_pre = ExecPre::default();
        match (file_exec_pre, args_exec_pre) {
            (Some(file_exec_pre), Some(args_exec_pre)) => {
                exec_pre.merge_file_exec_pre(file_exec_pre);
                exec_pre.merge_args_exec_pre(args_exec_pre);
            }
            (None, Some(args_exec_pre)) => {
                ExecPre::merge_args_exec_pre(&mut exec_pre, args_exec_pre);
            }
            (Some(file_exec_pre), None) => {
                ExecPre::merge_file_exec_pre(&mut exec_pre, file_exec_pre);
            }
            (None, None) => return self.exec_pre = None,
        }

        self.exec_pre = Some(exec_pre);
    }

    fn merge_exclude(
        &mut self,
        file_exclude: Option<FileOptExclude>,
        args_exclude: Option<Vec<String>>,
    ) {
        match (file_exclude, args_exclude) {
            (Some(file_exclude), Some(args_exclude)) => {
                self.exclude.merge_file_exclude(file_exclude);
                self.exclude.merge_args_exclude(args_exclude);
            }
            (None, Some(args_exclude)) => self.exclude.merge_args_exclude(args_exclude),
            (Some(file_exclude), None) => self.exclude.merge_file_exclude(file_exclude),
            (None, None) => self.exclude = Exclude::default(),
        }
    }

    fn merge_extend(
        &mut self,
        file_extend: Option<FileOptExtend>,
        args_extend: Option<ArgsOptExtend>,
    ) {
        match (file_extend, args_extend) {
            (Some(file_extend), Some(args_extend)) => {
                self.extend.merge_file_extend(file_extend);
                self.extend.merge_args_extend(args_extend);
            }
            (None, Some(args_extend)) => self.extend.merge_args_extend(args_extend),
            (Some(file_extend), None) => self.extend.merge_file_extend(file_extend),
            (None, None) => self.extend = Extend::default(),
        }
    }

    pub fn to_extend(&self) -> &Extend {
        &self.extend
    }

    pub fn to_exclude(&self) -> &Exclude {
        &self.exclude
    }

    pub fn root(&self) -> &PathBuf {
        &self.root
    }

    pub fn to_exec(&self) -> &Exec {
        &self.exec
    }

    pub fn to_exec_pre(&self) -> &Option<ExecPre> {
        &self.exec_pre
    }

    fn canonicalise(&mut self) {
        if let Some(exec_pre) = self.exec_pre.as_mut() {
            exec_pre.canonicalise(&self.root);
        }
        self.exec.canonicalise(&self.root);
    }
}

#[derive(Debug, Default)]
pub struct Exec {
    command: String,
    args: Vec<String>,
    origin: PathBuf,
}

impl Exec {
    fn merge_file_exec(&mut self, mut file_exec: FileOptExec) {
        if let Some(command) = file_exec.take_command() {
            self.command = command;
        }
        if let Some(args) = file_exec.take_args() {
            self.args = args;
        }
        if let Some(origin) = file_exec.take_origin() {
            self.origin = origin;
        }
    }

    fn merge_args_exec(&mut self, mut args_exec: ArgsOptExec) {
        if let Some(command) = args_exec.take_exec() {
            let (command, args) = parse_command_string(command);
            if let Some(command) = command {
                self.command = command;
            }
            self.args = args;
        }
        if let Some(origin) = args_exec.take_origin() {
            let origin = PathBuf::from_str(&origin).unwrap();
            self.origin = origin;
        }
    }

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
                "Error while canonicalising exec origin ({:?}): {err:?}",
                self.origin()
            );
            err
        });

        if let Ok(origin) = origin {
            self.origin = origin;
        }
    }
}

#[derive(Debug, Default)]
pub struct ExecPre {
    origin: PathBuf,
    when: When,
    commands: Vec<ExecPreCommand>,
}

impl ExecPre {
    fn merge_file_exec_pre(&mut self, mut file_exec_pre: FileOptExecPre) {
        if let Some(commands) = file_exec_pre.take_commands() {
            self.commands = commands
                .into_iter()
                .map(|mut file_opt_exec_pre_command| {
                    let mut exec_pre_command = ExecPreCommand::default();
                    if let Some(command) = file_opt_exec_pre_command.take_command() {
                        exec_pre_command.command = command;
                    }

                    if let Some(args) = file_opt_exec_pre_command.take_args() {
                        exec_pre_command.args = args;
                    }

                    exec_pre_command
                })
                .collect();
        }
        if let Some(when) = file_exec_pre.take_when() {
            self.when = match when {
                FileOptWhen::Once => When::Once,
                FileOptWhen::Always => When::Always,
            };
        }
        if let Some(origin) = file_exec_pre.take_origin() {
            self.origin = origin;
        }
    }

    fn merge_args_exec_pre(&mut self, mut args_exec_pre: ArgsOptExecPre) {
        if let Some(commands) = args_exec_pre.take_exec_pre() {
            let commands: Result<Vec<ExecPreCommand>, _> =
                commands.into_iter().map(String::try_into).collect();

            if let Ok(commands) = commands {
                self.commands = commands;
            }
        }

        if let Some(when) = args_exec_pre.take_when() {
            self.when = match when {
                ArgsOptWhen::Once => When::Once,
                ArgsOptWhen::Always => When::Always,
            }
        }

        if let Some(origin) = args_exec_pre.take_origin_pre() {
            let origin = PathBuf::from_str(&origin).unwrap();
            self.origin = origin;
        }
    }

    pub fn when(&self) -> &When {
        &self.when
    }

    pub fn commands(&self) -> &Vec<ExecPreCommand> {
        &self.commands
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

#[derive(Debug, Default)]
pub enum When {
    Once,
    #[default]
    Always,
}

#[derive(Debug, Default)]
pub struct ExecPreCommand {
    command: String,
    args: Vec<String>,
}

impl TryFrom<String> for ExecPreCommand {
    type Error = String;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        let mut exec_pre_cmd = Self::default();

        if value.is_empty() {
            return Err("empty string".into());
        }

        let (command, args) = parse_command_string(value);

        if let Some(command) = command {
            exec_pre_cmd.command = command;
        }
        exec_pre_cmd.args = args;

        Ok(exec_pre_cmd)
    }
}

impl ExecPreCommand {
    pub fn command(&self) -> &str {
        &self.command
    }

    pub fn args(&self) -> &Vec<String> {
        &self.args
    }
}

#[derive(Debug, Default)]
pub struct Exclude {
    exclude_files: Vec<String>,
}

impl Exclude {
    fn merge_file_exclude(&mut self, file_exclude: FileOptExclude) {
        self.exclude_files = file_exclude.take_exclude_files();
    }

    fn merge_args_exclude(&mut self, args_exclude: Vec<String>) {
        self.exclude_files = args_exclude;
    }

    pub fn to_exclude_files(&self) -> &Vec<String> {
        &self.exclude_files
    }
}

#[derive(Debug, Default)]
pub struct Extend {
    extendables: Vec<ExtendableType>,
}

impl Extend {
    fn merge_file_extend(&mut self, file_extend: FileOptExtend) {
        self.extendables = file_extend.take_extendables();
    }

    fn merge_args_extend(&mut self, args_extend: ArgsOptExtend) {
        if let Ok(Extend { extendables }) = TryInto::<Extend>::try_into(args_extend) {
            self.extendables = extendables
        }
    }

    pub fn to_extendables(&self) -> &Vec<ExtendableType> {
        &self.extendables
    }
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

// FIXME: doesn't work with string commands like `watchdog --exec "cmd --my-var='space separated variable'"`
fn parse_command_string(command: String) -> (Option<String>, Vec<String>) {
    if command.is_empty() {
        return (None, Vec::new());
    }
    let mut tokens = command.split_whitespace();
    let command = tokens.next().unwrap().into();
    let args = tokens.map(String::from).collect();

    (Some(command), args)
}

impl TryFrom<ArgsOptExtend> for Extend {
    type Error = String;

    fn try_from(mut value: ArgsOptExtend) -> Result<Self, Self::Error> {
        if let (Some(opt_extendable_types), Some(opt_paths)) =
            (value.take_extendable_type(), value.take_extend())
        {
            let extendables = opt_paths
                .into_iter()
                .zip(opt_extendable_types)
                .map(
                    |(opt_path, opt_extendable_type)| match opt_extendable_type {
                        ArgsOptExtendableType::Git => {
                            let root = find_root().unwrap_or_default();
                            let mut builder = GitignoreBuilder::new(&root);
                            let err = builder.add(root.join(opt_path));
                            if let Some(err) = err {
                                return Err(err.to_string());
                            }
                            let git = builder
                                .build()
                                .map_err(|err| format!("Failed to build Gitignore: {err}"))?;

                            Ok(ExtendableType::Git(GitignoreSerde(git)))
                        }
                    },
                )
                .collect::<Result<Vec<ExtendableType>, String>>()?;

            return Ok(Self { extendables });
        }

        Err("Could not convert OptExtendFiles into Extend".into())
    }
}
