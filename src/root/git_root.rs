use super::{current_dir::CurrentDir, ProjectRoot, Root};
use std::{process::Command, str};

#[derive(Default)]
pub struct GitRoot {
    next: Option<Box<dyn ProjectRoot>>,
}

impl GitRoot {
    pub fn new() -> Self {
        Self {
            next: Some(Box::new(CurrentDir::new())),
        }
    }
}

impl ProjectRoot for GitRoot {
    fn find(&self, root: &mut Root) {
        let git_output = match Command::new("git")
            .args(["rev-parse", "--show-toplevel"])
            .output()
        {
            Ok(output) if output.status.success() => match str::from_utf8(&output.stdout) {
                Ok(stdout) => Some(stdout.trim().into()),
                _ => None,
            },
            _ => None,
        };

        match git_output {
            Some(output) => {
                root.errors.clear();
                root.root = output;
            }
            _ => {
                root.errors
                    .push("Tried searching for the root project through git".into());
                if let Some(next) = self.next() {
                    next.find(root)
                }
            }
        };
    }

    fn next(&self) -> &Option<Box<dyn ProjectRoot>> {
        &self.next
    }
}
