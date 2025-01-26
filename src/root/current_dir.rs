use super::ProjectRoot;
use std::{env::current_dir, path::PathBuf};

pub struct CurrentDir {
    next: Option<()>,
}

impl CurrentDir {
    pub fn new() -> Self {
        Self { next: Some(()) }
    }
}

impl ProjectRoot for CurrentDir {
    type NextHandler = ();

    fn find(&self) -> Result<PathBuf, String> {
        current_dir().map_err(|_| {
            "Tried searching for the root project through the current working directory".into()
        })
    }

    fn next(&self) -> &Option<()> {
        &self.next
    }
}
