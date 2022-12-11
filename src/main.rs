mod app;
use colored::Colorize;
mod comment;
mod contentline;
mod files;
mod hashable;
mod section;
use std::{fmt::format, path::PathBuf};

use crate::{
    files::{Metafile, Specialfile},
    hashable::Hashable,
};

pub mod built_info {
    // The file has been placed there by the build script.
    include!(concat!(env!("OUT_DIR"), "/built.rs"));
}

macro_rules! check_file_arg {
    ($a:expr) => {
        if !$a.is_file() {
            eprintln!("{}", "file does not exist".red().bold());
            //TODO make this an error
            return Ok(());
        }
    };
}

fn main() -> Result<(), std::io::Error> {
    let tester = app::build_app();
    let matches = tester.get_matches();

    // compile a file, making it an unmodified imosid file
    if let Some(matches) = matches.subcommand_matches("compile") {
        let filename = matches.get_one::<PathBuf>("file").unwrap();
        check_file_arg!(filename);
        if *matches.get_one("metafile").unwrap() {
            let mut newmetafile = Metafile::from(filename.to_path_buf());
            newmetafile.compile();
            newmetafile.write_to_file();
            println!("compiled {}", &filename.to_str().unwrap().bold());
            return Ok(());
        }
        let mut compfile = match Specialfile::from(filename) {
            Ok(file) => file,
            Err(_) => {
                eprintln!("{}", "could not open file".red());
                return Ok(());
            }
        };
        if compfile.compile() {
            compfile.write_to_file();
            println!("compiled {}", filename.to_str().unwrap().bold());
        } else {
            println!(
                "{} already compiled, no change",
                filename.to_str().unwrap().bold().green()
            );
        }
    }

    if let Some(matches) = matches.subcommand_matches("info") {
        let filename = matches.get_one::<PathBuf>("file").unwrap();
        check_file_arg!(filename);
        let infofile = Specialfile::from(filename)?;
        match &infofile.metafile {
            None => {
                println!("comment syntax: {}", &infofile.commentsign);
                for i in infofile.sections {
                    if !i.name.is_some() {
                        continue;
                    }
                    let outstr = format!(
                        "{}-{}: {} | {}{}",
                        i.startline,
                        i.endline,
                        i.name.unwrap(),
                        if i.modified {
                            "modified".red().bold()
                        } else {
                            "ok".green().bold()
                        },
                        match i.source {
                            Some(s) => {
                                format!(" | source {}", &s)
                            }
                            None => {
                                "".to_string()
                            }
                        }
                    );
                    println!("{}", outstr);
                }
            }
            Some(metafile) => {
                println!("metafile hash: {}", &metafile.hash);
                println!(
                    "{}",
                    if metafile.modified {
                        "modified".red()
                    } else {
                        "unmodified".green()
                    }
                )
            }
        }
        if let Some(permissions) = infofile.permissions {
            println!(
                "target file permissions: {}",
                &permissions.to_string().bold()
            )
        }
        if let Some(target) = infofile.targetfile {
            println!("target: {}", &target);
        }

        if infofile.modified {
            std::process::exit(1);
        }
    }

    return Ok(());
}
