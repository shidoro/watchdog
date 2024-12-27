use notify::{Config, Event, RecommendedWatcher, RecursiveMode, Result, Watcher};
use std::{
    env::var,
    fs,
    io::{BufRead, BufReader},
    path::PathBuf,
    process::{Child, Command},
    sync::mpsc::channel,
};

const ROOT_PATH: &str = env!("CARGO_MANIFEST_DIR");
fn main() -> Result<()> {
    let (tx, rx) = channel();

    let config = Config::default();
    let mut watcher = RecommendedWatcher::new(tx, Config::with_compare_contents(config, true))?;

    let root_path = PathBuf::new().join(ROOT_PATH);
    watcher.watch(&root_path, RecursiveMode::Recursive)?;
    println!("created a watcher at root path: {root_path:?}");

    let is_child_proc = var("WATCHDOG_CHILD_PROC");
    let mut child_proc = None;
    if is_child_proc.is_err() {
        child_proc = start_app();
    }

    for res in rx {
        match res {
            Ok(event) => event_handler(event, &mut child_proc),
            Err(e) => eprintln!("Watch error: {e:?}"),
        }
    }

    Ok(())
}

fn start_app() -> Option<Child> {
    Command::new("cargo")
        .current_dir(ROOT_PATH)
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

fn event_handler(event: Event, proc: &mut Option<Child>) {
    let should_ignore = should_ignore_event(&event.paths);
    if !should_ignore {
        println!("do not ignore event: {event:?}");
        handler(event, proc);
    }
}

fn get_ignorable_paths() -> Vec<String> {
    let root = PathBuf::new().join(ROOT_PATH);
    let git = root.join(".git").to_str().unwrap().to_string();
    let mut ignorable_paths = extend_git_ignore();
    ignorable_paths.push(git);

    ignorable_paths
}

fn extend_git_ignore() -> Vec<String> {
    let root = PathBuf::new().join(ROOT_PATH);
    let path = root.join(".gitignore");
    let file =
        fs::File::open(&path).unwrap_or_else(|_| panic!("tried to open file at path {path:?}"));

    let reader = BufReader::new(file);
    let paths_to_ignore: Vec<String> = reader.lines().map(|line| line.unwrap()).collect();

    paths_to_ignore
}

fn should_ignore_event(paths: &[PathBuf]) -> bool {
    let paths_to_ignore = get_ignorable_paths();
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
