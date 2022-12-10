use clap::{arg, command, value_parser, Arg, ArgAction, ColorChoice, Command};
use std::path::PathBuf;

pub fn build_app() -> Command {
    command!()
        .color(ColorChoice::Always)
        .subcommand_required(true)
        .about("instant manager of sections in dotfiles")
        .author("paperbenni <paperbenni@gmail.com>")
        .arg(arg!([name] "Optional Name to operate on"))
        .arg(arg!(-d --debug "debugging?"))
        .arg(
            arg!(-c --config <FILE> "sets a custom config file")
                .required(false)
                .value_parser(value_parser!(PathBuf)),
        )
        .subcommand(
            Command::new("test")
                .about("testing stuff")
                .arg(arg!(-l --list "list test values").action(ArgAction::SetTrue)),
        )
        .subcommand(
            Command::new("compile").about("compile file").arg(
                Arg::new("file")
                    .value_parser(value_parser!(PathBuf))
                    .required(true)
                    .help("file to compile"),
            ),
        )
        .subcommand(
            Command::new("update")
                .about("update sections from sources")
                .arg(
                    Arg::new("file")
                        .help("file to update")
                        .required(true)
                        .value_parser(value_parser!(PathBuf)),
                )
                .arg(
                    Arg::new("print")
                        .help("only print result, do not write to file")
                        .required(false)
                        .action(ArgAction::SetTrue),
                )
                .arg(
                    Arg::new("section")
                        .help("only update section, default is all")
                        .required(false)
                        .action(ArgAction::Append),
                ),
        )
        .subcommand(
            Command::new("query")
                .about("print section from file")
                .arg(
                    Arg::new("section")
                        .action(ArgAction::Append)
                        .required(true)
                        .value_parser(value_parser!(String))
                        .help("section(s) to include in output"),
                )
                .arg(
                    Arg::new("file")
                        .required(true)
                        .help("file to search through")
                        .value_parser(value_parser!(PathBuf)),
                ),
        )
        .subcommand(
            Command::new("info")
                .about("list imosid metadate in file")
                .arg(
                    Arg::new("file")
                        .required(true)
                        .help("file to get info for")
                        .value_parser(value_parser!(PathBuf)),
                ),
        )
        .subcommand(
            Command::new("apply")
                .about("apply source to target marked in the file")
                .arg(
                    Arg::new("file")
                        .required(true)
                        .help("file to apply")
                        .value_parser(value_parser!(PathBuf)),
                ),
        )
        .subcommand(
            Command::new("check")
                .about("check directory for modified files")
                .arg(
                    Arg::new("directory")
                        .required(true)
                        .value_parser(value_parser!(PathBuf)),
                ),
        )
}
