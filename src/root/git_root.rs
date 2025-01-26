use super::{current_dir::CurrentDir, ProjectRoot};
use std::{path::PathBuf, process::Command, str};

pub struct GitRoot {
    next: Option<CurrentDir>,
}

impl GitRoot {
    pub fn new() -> Self {
        Self {
            next: Some(CurrentDir::new()),
        }
    }
}

impl ProjectRoot for GitRoot {
    type NextHandler = CurrentDir;

    fn find(&self) -> Result<PathBuf, String> {
        let error_msg = "Tried searching for the root project through git".into();
        match Command::new("git")
            .args(["rev-parse", "--show-toplevel"])
            .output()
        {
            Ok(output) if output.status.success() => match str::from_utf8(&output.stdout) {
                Ok(stdout) => Ok(stdout.trim().into()),
                _ => Err(error_msg),
            },
            _ => Err(error_msg),
        }
    }

    fn next(&self) -> &Option<CurrentDir> {
        &self.next
    }
}
