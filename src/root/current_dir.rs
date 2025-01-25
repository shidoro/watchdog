use super::{ProjectRoot, Root};
use std::env::current_dir;

#[derive(Default)]
pub struct CurrentDir {
    next: Option<Box<dyn ProjectRoot>>,
}

impl CurrentDir {
    pub fn new() -> Self {
        Self { next: None }
    }
}

impl ProjectRoot for CurrentDir {
    fn find(&self, root: &mut Root) {
        match current_dir() {
            Ok(cur_dir) => {
                root.errors.clear();
                root.root = cur_dir;
            }
            _ => {
                root.errors.push(
                    "Tried searching for the root project through the current working directory"
                        .into(),
                );
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
