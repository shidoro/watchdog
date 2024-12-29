use config::{Config, Extendable};
use notify::{
    event::{CreateKind, ModifyKind, RemoveKind},
    Config as NotifyConfig, Error, Event, EventKind, RecommendedWatcher, RecursiveMode, Result,
    Watcher,
};
use std::{
    env::var,
    process::{Child, Command},
    sync::mpsc::channel,
};

mod config;

fn main() -> Result<()> {
    let config = config::load_config().map_err(|err| Error::generic(&format!("{err}")))?;

    match var("WATCHDOG_CHILD_PROC") {
        Ok(_) => run_child(),
        Err(_) => watchdog(&config),
    }
}

fn watchdog(config: &Config) -> Result<()> {
    let root_path = config.root();
    let (tx, rx) = channel();

    let mut child_proc = start_app();

    let notify_config = NotifyConfig::default();
    let mut watcher =
        RecommendedWatcher::new(tx, NotifyConfig::with_compare_contents(notify_config, true))?;

    watcher.watch(root_path, RecursiveMode::Recursive)?;

    for res in rx {
        match res {
            Ok(event) => event_handler(event, &mut child_proc, config),
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

fn event_handler(event: Event, proc: &mut Option<Child>, config: &Config) {
    let should_ignore = match &event.kind {
        EventKind::Create(create_kind) => {
            should_ignore_event(config, &event, create_kind == &CreateKind::Folder)
        }
        EventKind::Remove(remove_kind) => {
            should_ignore_event(config, &event, remove_kind == &RemoveKind::Folder)
        }
        EventKind::Modify(ModifyKind::Data(_)) | EventKind::Modify(ModifyKind::Name(_)) => {
            should_ignore_event(config, &event, false)
        }
        _ => true,
    };
    if !should_ignore {
        handler(event, proc);
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
