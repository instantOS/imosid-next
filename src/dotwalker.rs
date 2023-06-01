pub(crate) use std::path::PathBuf;

use walkdir::WalkDir;

pub fn walkdots(path: &PathBuf) -> impl Iterator<Item = walkdir::DirEntry> {
    // TODO: how does ripgrep handle this?
    let walker = WalkDir::new(path)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| {
            let path = e.path();
            let entrystring = path.to_str().unwrap();
            !entrystring.ends_with(".imosid.toml")
                && !entrystring.contains("/.git/")
                && path.to_path_buf().is_file()
        });
    return walker;
}
