pub(crate) use std::path::PathBuf;

use walkdir::WalkDir;

pub fn walkdots(path: &PathBuf) -> impl Iterator<Item = walkdir::DirEntry> {
    let walker = WalkDir::new(path)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| {
            let entrystring = e.path().to_str().unwrap();
            !entrystring.ends_with(".imosid.toml")
                && !entrystring.contains("/.git/")
                && e.path().to_path_buf().is_file()
        });
    return walker;
}
