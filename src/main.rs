use std::env;
use std::process;

//const PATH: &str = ".rnpm";

fn main() {
    let config = rnpm::Config::new(env::args()).unwrap_or_else(|err| {
        eprintln!("Problem parsing arguments: {}", err);
        process::exit(1);
    });

    rnpm::run(&config).unwrap_or_else(|err| {
        eprintln!("{}", err);
        process::exit(1);
    });
}
