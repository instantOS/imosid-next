mod app;
mod dotwalker;
mod test;
use colored::Colorize;
use dotwalker::walkdots;
mod comment;
mod contentline;
mod files;
mod hashable;
mod section;
use std::path::PathBuf;

use crate::{
    files::{ApplyResult, Metafile, Specialfile},
    hashable::Hashable,
};

pub mod built_info {
    // The file has been placed there by the build script.
    include!(concat!(env!("OUT_DIR"), "/built.rs"));
}

// clap value parser does not distinguish between files and directories
macro_rules! check_file_arg {
    ($a:expr) => {
        if !$a.is_file() {
            eprintln!("{}", "file does not exist".red().bold());
            //TODO make this an error
            return Ok(());
        }
    };
}

macro_rules! specialfile {
    ($a:expr) => {
        match Specialfile::from_pathbuf($a) {
            Ok(file) => file,
            Err(_) => {
                eprintln!("could not open file {}", $a.to_str().unwrap().red());
                return Ok(());
            }
        };
    };
}

fn main() -> Result<(), std::io::Error> {
    let imosidapp = app::build_app();
    let matches = imosidapp.get_matches();

    match matches.subcommand() {
        // compile a file, making it an unmodified imosid file
        Some(("compile", compile_matches)) => {
            let filename = compile_matches.get_one::<PathBuf>("file").unwrap();
            check_file_arg!(filename);
            if *compile_matches.get_one("metafile").unwrap() {
                let mut newmetafile = Metafile::from(filename.to_path_buf());
                newmetafile.compile();
                newmetafile.write_to_file();
                println!("compiled {}", &filename.to_str().unwrap().bold());
                return Ok(());
            }
            let mut compfile = specialfile!(filename);
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
        Some(("check", check_matches)) => {
            let filename = check_matches.get_one::<PathBuf>("directory").unwrap();
            if !filename.is_dir() {
                eprintln!(
                    "{} is not a directory, only directories can be checked",
                    filename.to_str().unwrap().red()
                );
                return Ok(());
            }
            let mut anymodified = false;
            for entry in walkdots(filename) {
                let entrypath = entry.path().to_path_buf();
                let checkfile = match Specialfile::from_pathbuf(&entrypath) {
                    Ok(file) => file,
                    Err(_) => {
                        eprintln!("could not open file {}", entrypath.to_str().unwrap().red());
                        continue;
                    }
                };
                if checkfile.modified {
                    println!("{} {}", checkfile.filename.red().bold(), "modified".red());
                    anymodified = true;
                }
                let mut fileanonymous = true;
                if !checkfile.metafile.is_some() {
                    for i in checkfile.sections {
                        if !i.is_anonymous() {
                            fileanonymous = false;
                            break;
                        }
                    }
                    if fileanonymous {
                        println!(
                            "{} {}",
                            checkfile.filename.yellow().bold(),
                            "is unmanaged".yellow()
                        )
                    }
                }
            }
        }

        Some(("query", query_matches)) => {
            let filename = query_matches.get_one::<PathBuf>("file").unwrap();
            // this looks bad
            let sections = query_matches
                .get_many::<String>("section")
                .unwrap_or_default()
                .map(|v| v.as_str())
                .collect::<Vec<_>>();
            check_file_arg!(filename);

            let queryfile = specialfile!(filename);

            if queryfile.metafile.is_some() {
                todo!("add message for this");
                return Ok(());
            }
            for i in queryfile.sections {
                if i.is_anonymous() {
                    continue;
                }
                for j in &sections {
                    if i.name.clone().unwrap().eq(j) {
                        println!("{}", i.output(&queryfile.commentsign));
                    }
                }
            }
        }

        Some(("update", update_matches)) => {
            let filename = update_matches.get_one::<PathBuf>("file").unwrap();

            // this looks bad
            let sections = update_matches
                .get_many::<String>("section")
                .unwrap_or_default()
                .map(|v| v.as_str())
                .collect::<Vec<_>>();

            check_file_arg!(filename);

            let mut updatefile = specialfile!(filename);
            updatefile.update();

            match updatefile.metafile {
                Some(_) => {
                    eprintln!("cannot update metafile");
                    return Ok(());
                }
                None => {}
            }

            if sections.is_empty() {
                // update all sections
            }
        }
        Some(("delete", delete_matches)) => {
            let filename = delete_matches.get_one::<PathBuf>("file").unwrap();

            // this looks bad
            let sections = delete_matches
                .get_many::<String>("section")
                .unwrap_or_default()
                .map(|v| v.as_str())
                .collect::<Vec<_>>();

            check_file_arg!(filename);

            let mut deletefile = specialfile!(filename);

            for i in sections {
                if deletefile.deletesection(i) {
                    println!("deleted section {}", i.bold());
                } else {
                    println!("could not find section {}", i.red());
                }
            }
            deletefile.write_to_file();
        }

        Some(("apply", apply_matches)) => {
            let mut donesomething = false;
            let filename = apply_matches.get_one::<PathBuf>("file").unwrap();
            if filename.is_dir() {
                for entry in walkdots(filename)
                {
                    let entrypath = entry.path().to_path_buf();
                    let entrystring = entry.path().to_str();
                    let tmpsource = match Specialfile::from_pathbuf(&entry.path().to_path_buf()) {
                        Ok(file) => file,
                        Err(_) => {
                            eprintln!(
                                "could not open file {}",
                                &entry.path().to_str().unwrap().red()
                            );
                            continue;
                        }
                    };
                    match tmpsource.apply() {
                        ApplyResult::Changed => {
                            donesomething = true;
                        }
                        _ => {}
                    }
                }
                if !donesomething {
                    println!("{}", "nothing to do".bold());
                }
                return Ok(());
            } else if filename.is_file() {
                let tmpsource = specialfile!(filename);
                tmpsource.apply();
            } else {
                eprintln!("{}", "file does not exist".red().bold());
                return Ok(());
            }
        }
        Some(("info", info_matches)) => {
            let filename = info_matches.get_one::<PathBuf>("file").unwrap();
            check_file_arg!(filename);
            let infofile = Specialfile::from_pathbuf(filename)?;
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
        Some((&_, _)) => {
            return Ok(());
        }
        None => {
            return Ok(());
        }
    }
    return Ok(());
}
