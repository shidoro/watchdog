use crate::{
    config::{Config, Extend, ExtendableType},
    find_root,
};
use ignore::gitignore::GitignoreBuilder;

pub fn ignore_files(config: Config) -> Vec<String> {
    let mut paths_to_ignore: Vec<String> = Vec::new();
    config
        .to_exclude()
        .files()
        .iter()
        .for_each(|ignorable_path| paths_to_ignore.push(ignorable_path.path().to_owned()));

    // config
    //     .to_extend()
    //     .files()
    //     .iter()
    //     .for_each(|extendable_path| paths_to_ignore.push(extendable_path.path().to_owned()));

    extend(config.to_extend());

    paths_to_ignore
}

fn extend(extend: &Extend) {
    let extendable_path = extend.files();
    extendable_path
        .iter()
        .for_each(|path| match path.extendable_type() {
            ExtendableType::Git => extend_git_ignore(path.path()),
        });
}

fn extend_git_ignore(path: &String) {
    let root = find_root().unwrap();
    let mut builder = GitignoreBuilder::new(&root);
    builder.add(root.join(path));
    let gitignore = builder.build().unwrap();

    let path = root.join(path);
    let matched = gitignore.matched(root.join("src/config.rs"), false);
    println!("{:?} at path {:?}", matched.is_ignore(), root.join("src"));
}
