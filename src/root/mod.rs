mod current_dir;
mod git_root;
mod no_root;

use lazy_static::lazy_static;
use std::path::PathBuf;

trait ProjectRoot {
    type NextHandler: ProjectRoot;

    fn handle(&self, root: &mut Root) {
        match self.find() {
            Ok(path) => {
                root.errors.clear();
                root.root = path;
            }
            Err(err) => {
                root.errors.push(err);
                if let Some(next) = self.next() {
                    next.handle(root);
                }
            }
        }
    }

    fn find(&self) -> Result<PathBuf, String>;
    fn next(&self) -> &Option<Self::NextHandler>;
}

#[derive(Debug, Default)]
pub struct Root {
    root: PathBuf,
    errors: Vec<String>,
}

impl Root {
    fn new<P: ProjectRoot>(beginning_of_chain: P) -> Result<Self, String> {
        let mut root = Self::default();
        beginning_of_chain.handle(&mut root);

        if !root.errors.is_empty() {
            return Err(root.errors.join("\n"));
        }

        Ok(root)
    }

    pub fn root(&self) -> &PathBuf {
        &self.root
    }
}

lazy_static! {
    pub static ref ROOT: Root = {
        match Root::new(git_root::GitRoot::new()) {
            Ok(root) => root,
            Err(err) => panic!("Couldn't find the root of the project. Here is a list of attempted ways to find it:\n{}", err)
        }
    };
}
