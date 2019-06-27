#![forbid(unsafe_code)]
#[macro_use]
extern crate clap;
extern crate dirs;
extern crate exitfailure;
extern crate failure;

use clap::{App, Arg, SubCommand};
use exitfailure::ExitFailure;

fn main() -> Result<(), ExitFailure> {
    let matches = App::new("rnpmrc")
        .about("A simple tool to manage multiple .npmrc files")
        .version(crate_version!())
        .author(crate_authors!())
        .subcommand(
            SubCommand::with_name("create")
                .about("Creates new profile")
                .arg(
                    Arg::with_name("profile")
                        .help("Profile name")
                        .required(true),
                ),
        )
        .subcommand(SubCommand::with_name("list").about("Lists all profiles"))
        .subcommand(
            SubCommand::with_name("open")
                .about("Opens a profile")
                .arg(
                    Arg::with_name("profile")
                        .help("Profile name")
                        .required(true),
                )
                .arg(
                    Arg::with_name("editor")
                        .short("e")
                        .long("editor")
                        .help("Editor to open file")
                        .value_name("EDITOR")
                        .default_value("vi")
                        .takes_value(true),
                ),
        )
        .subcommand(
            SubCommand::with_name("remove")
                .about("Removes profile")
                .arg(
                    Arg::with_name("profile")
                        .help("Profile name")
                        .required(true),
                ),
        )
        .subcommand(
            SubCommand::with_name("activate")
                .about("Activate profile")
                .arg(
                    Arg::with_name("profile")
                        .help("Profile name")
                        .required(true),
                ),
        )
        .subcommand(SubCommand::with_name("status").about("Shows current activate profile"))
        .subcommand(
            SubCommand::with_name("backup")
                .about("Creates a profile from .npmrc file")
                .arg(
                    Arg::with_name("profile")
                        .help("Profile name")
                        .required(true),
                ),
        )
        .get_matches();

    rnpmrc::run(&matches)?;

    Ok(())
}
