mod app;
use colored::Colorize;
mod comment;
mod contentline;
mod files;
mod hashable;
mod section;
use std::path::PathBuf;

use crate::{
    files::{Metafile, Specialfile},
    hashable::Hashable,
};

pub mod built_info {
    // The file has been placed there by the build script.
    include!(concat!(env!("OUT_DIR"), "/built.rs"));
}

fn main() -> Result<(), std::io::Error> {
    let tester = app::build_app();
    let matches = tester.get_matches();
    if let Some(matches) = matches.subcommand_matches("compile") {
        let filename = matches.get_one::<PathBuf>("file").unwrap();
        if *matches.get_one("metafile").expect("could not open file") {
            let mut newmetafile = Metafile::from(filename.to_path_buf());
            newmetafile.compile();
            newmetafile.write_to_file();
            println!("compiled {}", &filename.to_str().unwrap().bold());
            return Ok(());
        }
        if !filename.is_file() {
            //TODO make this an error message
            return Ok(());
        }


    }

    return Ok(());
}
