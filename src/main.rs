use std::env;
use std::process;

fn main() {
    let config = rnpmrc::Config::new(env::args()).unwrap_or_else(|err| {
        eprintln!("Problem parsing arguments: {}", err);
        process::exit(1);
    });

    rnpmrc::run(&config).unwrap_or_else(|err| {
        eprintln!("{}", err);
        process::exit(1);
    });
}
