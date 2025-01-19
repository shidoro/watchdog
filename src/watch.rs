use crate::config::{Config, ExecPre, Extendable, When};
use notify::{
    event::{CreateKind, RemoveKind},
    Config as NotifyConfig, Event, EventKind, RecommendedWatcher, RecursiveMode, Result, Watcher,
};
use std::{
    path::PathBuf,
    process::{Child, Command},
    sync::mpsc::channel,
    thread,
    time::Duration,
};

pub fn watch(config: &mut Config) -> Result<()> {
    let mut child_proc = exec(config, false);

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

fn exec(config: &Config, restart: bool) -> Option<Child> {
    let _ = Command::new("clear").spawn().unwrap().wait();

    if let Some(exec_pre) = config.to_exec_pre() {
        match exec_pre.when() {
            When::Once if !restart => execute_pre(exec_pre),
            When::Always => execute_pre(exec_pre),
            _ => {}
        }
    }

    let exec = config.to_exec();
    let command = exec.command();
    let args = exec.args();
    let origin = exec.origin();
    println!(
        "executing command {:?} with args {:?} at origin {:?}",
        command, args, origin
    );
    Command::new(command)
        .args(args)
        .current_dir(origin)
        .spawn()
        .ok()
}

fn execute_pre(exec_pre: &ExecPre) {
    let commands = exec_pre.commands();
    commands.iter().for_each(|exec_pre_command| {
        let command = exec_pre_command.command();
        let args = exec_pre_command.args();
        let origin = exec_pre.origin();
        println!(
            "pre executing command {:?} with args {:?} at origin {:?}",
            command, args, origin
        );
        let _ = Command::new(command)
            .args(args)
            .current_dir(origin)
            .spawn()
            .map_err(|err| {
                format!(
                    "Something went wrong when executing command {:?} with args {:?}. {:?}",
                    command, args, err
                )
            })
            .unwrap()
            .wait();
    });
}

fn handler(event: Event, config: &mut Config, child_proc: &mut Option<Child>) {
    if let Some(mut child) = child_proc.take() {
        let _ = child.kill();
        let _ = child.wait();
    }

    if should_reload_config(&event) {
        match Config::new() {
            Ok(new_config) => *config = new_config,
            Err(err) => eprintln!("Error loading new config: {:?}", err),
        }
    }

    *child_proc = exec(config, true);
}

fn should_reload_config(event: &Event) -> bool {
    let watchdog = PathBuf::from("watchdog.toml");
    event
        .paths
        .iter()
        .any(|path| path.file_name().unwrap().eq(watchdog.file_name().unwrap()))
}

fn event_handler(event: Event, config: &mut Config, child_proc: &mut Option<Child>) {
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
        handler(event, config, child_proc);
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
