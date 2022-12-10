mod app;
mod section;
mod comment;
use std::path::PathBuf;

fn main() {
    let tester = app::build_app();
    let matches = tester.get_matches();
    if let Some(matches) = matches.subcommand_matches("compile") {
        if let Some(name) = matches.get_one::<PathBuf>("file") {
            let mut name2 = name.clone();
            name2.push("thiswaspushed");
            if name2.exists() {
                println!("name {}", name2.to_str().unwrap());
            }
        } else {
            println!("no name");
        }
    }
}
