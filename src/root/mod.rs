mod current_dir;
mod git_root;

use lazy_static::lazy_static;
use std::path::PathBuf;

trait ProjectRoot {
    fn find(&self, root: &mut Root);
    fn next(&self) -> &Option<Box<dyn ProjectRoot>>;
}

#[derive(Debug, Default)]
pub struct Root {
    root: PathBuf,
    errors: Vec<String>,
}

impl Root {
    fn new<P: ProjectRoot>(beginning_of_chain: P) -> Result<Self, String> {
        let mut root = Self::default();
        beginning_of_chain.find(&mut root);

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
            Err(err) => panic!("Couldn't find the root of the project. Here is a list of attempted ways to find it: {}", err)
        }
    };
}
