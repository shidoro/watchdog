use config::{load_config, Config, Extendable};
use notify::{
    event::{CreateKind, RemoveKind},
    Config as NotifyConfig, Error, Event, EventKind, RecommendedWatcher, RecursiveMode, Result,
    Watcher,
};
use std::{
    process::{Child, Command},
    sync::mpsc::channel,
    thread,
    time::Duration,
};

mod config;

fn main() -> Result<()> {
    let config = load_config().map_err(|err| Error::generic(&format!("{err}")))?;

    watchdog(&config)
}

fn watchdog(config: &Config) -> Result<()> {
    let mut child_proc = watch(config, false);

    let (tx, rx) = channel();
    let notify_config = NotifyConfig::default();
    let mut watcher =
        RecommendedWatcher::new(tx, NotifyConfig::with_compare_contents(notify_config, true))?;

    let root = config.root();
    watcher.watch(root, RecursiveMode::Recursive)?;

    loop {
        // there is an assumption here that when performing an operation such as creating/removing
        // or saving a file, the first event will always contain the path we're interested in
        let res = rx.recv();
        match res {
            Ok(Ok(event)) => event_handler(event, config, &mut child_proc),
            err => eprintln!("Watch error: {err:?}"),
        }

        // there's usually more events happening in the background e.g. git updating some internal files
        // there's no need to react on those events since the process has just been reloaded
        // so discard all of them; it's highly unlikely to make two changes in under half a second
        {
            thread::sleep(Duration::from_millis(500));
            while rx.try_recv().is_ok() {}
        }
    }
}

fn watch(config: &Config, restart: bool) -> Option<Child> {
    let _ = Command::new("clear").spawn().unwrap().wait();
    let run = config.to_run();
    if run.precompile() {
        println!("{}...", if restart { "Recompiling" } else { "Compiling" });
        let build = config.to_build();
        let _ = Command::new(build.command())
            .args(build.args())
            .current_dir(build.origin())
            .spawn()
            .expect("Something went wrong when building cargo")
            .wait();
    }

    println!("{}...", if restart { "Restart" } else { "Start" });
    Command::new(run.command())
        .args(run.args())
        .current_dir(run.origin())
        .spawn()
        .ok()
}

fn handler(config: &Config, child_proc: &mut Option<Child>) {
    if let Some(mut child) = child_proc.take() {
        let _ = child.kill();
        let _ = child.wait();
    }

    *child_proc = watch(config, true);
}

fn event_handler(event: Event, config: &Config, child_proc: &mut Option<Child>) {
    let should_ignore = match &event.kind {
        EventKind::Create(create_kind) => {
            should_ignore_event(config, &event, create_kind == &CreateKind::Folder)
        }
        EventKind::Remove(remove_kind) => {
            should_ignore_event(config, &event, remove_kind == &RemoveKind::Folder)
        }
        EventKind::Modify(_) => should_ignore_event(config, &event, false),
        _ => true,
    };
    if !should_ignore {
        handler(config, child_proc);
    }
}

fn should_ignore_event(config: &Config, event: &Event, is_dir: bool) -> bool {
    let paths = &event.paths;
    let paths_to_ignore = config.to_exclude().to_exclude_files();
    let extendables = &config.to_extend().to_extendables();
    paths.iter().all(|path| {
        for path_to_ignore in paths_to_ignore {
            if path.to_str().unwrap().contains(path_to_ignore) {
                return true;
            }
        }
        extendables
            .iter()
            .any(|extendable| extendable.matcher(path, is_dir))
    })
}
