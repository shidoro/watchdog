use clap::{builder::ArgPredicate, Args, Parser, ValueEnum};
use serde::Deserialize;

#[derive(Parser, Debug)]
#[command(version, about("A lightweight, language-agnostic file watcher that can execute commands when your code changes"))]
pub struct ArgsOpt {
    #[command(flatten)]
    exec: Option<ArgsOptExec>,

    #[command(flatten)]
    exec_pre: Option<ArgsOptExecPre>,

    #[arg(
        short('x'),
        long,
        value_name("PATH"),
        help("a list of paths you'd like to exclude from watching e.g watchdog -r \"cargo run\" --exclude-files \".git\"")
    )]
    exclude: Option<Vec<String>>,

    #[command(flatten)]
    extend: Option<ArgsOptExtend>,
}

impl ArgsOpt {
    pub fn take_exec(&mut self) -> Option<ArgsOptExec> {
        self.exec.take()
    }

    pub fn take_exec_pre(&mut self) -> Option<ArgsOptExecPre> {
        self.exec_pre.take()
    }

    pub fn take_exclude(&mut self) -> Option<Vec<String>> {
        self.exclude.take()
    }

    pub fn take_extend(&mut self) -> Option<ArgsOptExtend> {
        self.extend.take()
    }
}

#[derive(Args, Clone, Debug)]
pub struct ArgsOptExec {
    #[arg(
        short,
        long,
        value_name("COMMAND"),
        required(false),
        help("command to run e.g watchdog --exec \"cargo run\"")
    )]
    pub exec: Option<String>,

    #[arg(
        short,
        long = "exec-origin",
        value_name("PATH"),
        help("a path relative to the root project to where to run exec")
    )]
    pub origin: Option<String>,
}

impl ArgsOptExec {
    pub fn take_exec(&mut self) -> Option<String> {
        self.exec.take()
    }

    pub fn take_origin(&mut self) -> Option<String> {
        self.origin.take()
    }
}

#[derive(Args, Clone, Debug)]
pub struct ArgsOptExecPre {
    #[arg(
        short = 'E',
        long,
        value_name("COMMAND"),
        help("list of commands to sequentially execute before exec e.g watchdog -e \"cargo-run\" --exec-pre \"cargo build\"")
    )]
    exec_pre: Option<Vec<String>>,

    #[arg(short, long, help("when should the exec-pre commands run"))]
    when: Option<ArgsOptWhen>,

    #[arg(
        short = 'O',
        long = "exec-pre-origin",
        value_name("PATH"),
        help("a path relative to the root project to where to run exec-pre")
    )]
    origin_pre: Option<String>,
}

impl ArgsOptExecPre {
    pub fn take_exec_pre(&mut self) -> Option<Vec<String>> {
        self.exec_pre.take()
    }

    pub fn take_when(&mut self) -> Option<ArgsOptWhen> {
        self.when.take()
    }

    pub fn take_origin_pre(&mut self) -> Option<String> {
        self.origin_pre.take()
    }
}

#[derive(ValueEnum, Copy, Clone, Debug, Deserialize)]
pub enum ArgsOptWhen {
    #[serde(rename = "once")]
    Once,
    #[serde(rename = "always")]
    Always,
}

#[derive(Args, Clone, Debug)]
pub struct ArgsOptExtend {
    #[arg(
        short('X'),
        long,
        value_name("PATH"),
        group("extend-files"),
        help("a list of files to extend e.g watchdog -r \"cargo run\" --extend-files .gitignore -t git")
    )]
    extend: Option<Vec<String>>,

    #[arg(
        short('t'),
        long,
        default_value_if("extend-files", ArgPredicate::IsPresent, "git"),
        help("the type of ignore files you want to extend")
    )]
    extendable_type: Option<Vec<ArgsOptExtendableType>>,
}

#[derive(ValueEnum, Copy, Clone, Debug, Default, Deserialize)]
pub enum ArgsOptExtendableType {
    #[default]
    Git,
}

impl ArgsOptExtend {
    pub fn take_extend(&mut self) -> Option<Vec<String>> {
        self.extend.take()
    }

    pub fn take_extendable_type(&mut self) -> Option<Vec<ArgsOptExtendableType>> {
        self.extendable_type.take()
    }
}
