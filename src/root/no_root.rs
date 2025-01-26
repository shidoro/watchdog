use super::ProjectRoot;
use std::path::PathBuf;

impl ProjectRoot for () {
    type NextHandler = ();

    fn find(&self) -> Result<PathBuf, String> {
        Err("".into())
    }
    fn next(&self) -> &Option<Self::NextHandler> {
        &None
    }
}
