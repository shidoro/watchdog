use super::{hg_root::HgRoot, ProjectRoot};
use std::path::PathBuf;

pub struct GitRoot {
    next: Option<HgRoot>,
}

impl GitRoot {
    pub fn new() -> Self {
        Self {
            next: Some(HgRoot::new()),
        }
    }
}

impl ProjectRoot for GitRoot {
    type NextHandler = HgRoot;

    fn find(&self) -> Result<PathBuf, String> {
        let error_msg = "Tried searching for the root project through git";
        let args = ["rev-parse", "--show-toplevel"];

        self.command("git", &args, error_msg)
    }

    fn next(&self) -> &Option<HgRoot> {
        &self.next
    }
}
