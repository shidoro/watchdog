use core::str;
use notify::{Config, Error, Event, RecommendedWatcher, RecursiveMode, Result, Watcher};
use std::{
    env::{current_dir, var},
    fs,
    io::{BufRead, BufReader},
    path::{Path, PathBuf},
    process::{Child, Command},
    sync::mpsc::channel,
};

mod config;
mod ignore;

fn main() -> Result<()> {
    let root = find_root()?;
    let config =
        config::load_config(&root.join("watchdog.toml")).unwrap_or(config::Config::empty());

    let ignore_files = ignore::ignore_files(config);
    println!("files to ignore: {ignore_files:?}");
    match var("WATCHDOG_CHILD_PROC") {
        Ok(_) => run_child(),
        Err(_) => watchdog(&root),
    }
}

fn find_root() -> Result<PathBuf> {
    let mut root_path = String::new();
    if let Ok(git_root_path) = Command::new("git")
        .args(["rev-parse", "--show-toplevel"])
        .output()
    {
        root_path += str::from_utf8(&git_root_path.stdout).unwrap().trim();
    } else if let Ok(cargo_root_path) = current_dir() {
        root_path += cargo_root_path.to_str().unwrap();
    }

    if root_path.is_empty() {
        return Err(Error::generic("Could not find the root project"));
    }

    Ok(PathBuf::from(root_path))
}

fn watchdog(root_path: &PathBuf) -> Result<()> {
    let (tx, rx) = channel();

    let mut child_proc = start_app();

    let config = Config::default();
    let mut watcher = RecommendedWatcher::new(tx, Config::with_compare_contents(config, true))?;

    watcher.watch(root_path, RecursiveMode::Recursive)?;
    println!("Started watching directory {root_path:?}");

    for res in rx {
        match res {
            Ok(event) => event_handler(event, &mut child_proc, root_path),
            Err(e) => eprintln!("Watch error: {e:?}"),
        }
    }

    Ok(())
}

fn run_child() -> Result<()> {
    Ok(())
}

fn start_app() -> Option<Child> {
    Command::new("cargo")
        .env("WATCHDOG_CHILD_PROC", "1")
        .arg("run")
        .spawn()
        .ok()
}

fn handler(_: Event, proc: &mut Option<Child>) {
    if let Some(mut child) = proc.take() {
        let _ = child.kill();
        let _ = child.wait();
    }

    *proc = start_app();
}

fn event_handler(event: Event, proc: &mut Option<Child>, root_path: &Path) {
    let should_ignore = should_ignore_event(root_path, &event.paths);
    if !should_ignore {
        handler(event, proc);
    }
}

fn get_ignorable_paths(root_path: &Path) -> Vec<String> {
    let git = root_path.join(".git").to_str().unwrap().to_string();
    let mut ignorable_paths = extend_git_ignore(root_path);
    ignorable_paths.push(git);

    ignorable_paths
}

fn extend_git_ignore(root_path: &Path) -> Vec<String> {
    let path = root_path.join(".gitignore");
    let exists = fs::exists(&path);
    if exists.is_err() || !exists.unwrap() {
        return vec![];
    }
    let file =
        fs::File::open(&path).unwrap_or_else(|_| panic!("tried to open file at path {path:?}"));

    let reader = BufReader::new(file);
    let paths_to_ignore: Vec<String> = reader.lines().map(|line| line.unwrap()).collect();

    paths_to_ignore
}

fn should_ignore_event(root_path: &Path, paths: &[PathBuf]) -> bool {
    let paths_to_ignore = get_ignorable_paths(root_path);
    paths.iter().all(|path| {
        let path = path.to_str().unwrap();
        for path_to_ignore in &paths_to_ignore {
            if path.contains(path_to_ignore) {
                return true;
            }
        }
        false
    })
}
