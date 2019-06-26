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
                .about("Create new profile")
                .arg(
                    Arg::with_name("profile")
                        .help("Profile name")
                        .required(true),
                ),
        )
        .subcommand(SubCommand::with_name("list").about("List all profiles"))
        .subcommand(
            SubCommand::with_name("open")
                .about("Open a profile")
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
                        .default_value("vim")
                        .takes_value(true),
                ),
        )
        .subcommand(
            SubCommand::with_name("remove").about("Remove profile").arg(
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
        .subcommand(SubCommand::with_name("status").about("Show current activate profile"))
        .get_matches();

    rnpmrc::run(matches)?;

    Ok(())
}
