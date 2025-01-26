use super::{current_dir::CurrentDir, ProjectRoot};
use std::path::PathBuf;

pub struct HgRoot {
    next: Option<CurrentDir>,
}

impl HgRoot {
    pub fn new() -> Self {
        Self {
            next: Some(CurrentDir::new()),
        }
    }
}

impl ProjectRoot for HgRoot {
    type NextHandler = CurrentDir;

    fn find(&self) -> Result<PathBuf, String> {
        let error_msg = "Tried searching for the root project through hg";

        self.command("hg", &["root"], error_msg)
    }

    fn next(&self) -> &Option<CurrentDir> {
        &self.next
    }
}
