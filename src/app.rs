use crate::config;
use crate::config::Config;
use clap::{App, AppSettings, Arg, SubCommand};

pub fn run() -> Option<Config> {
    let matches = App::new("heimdall")
        .setting(AppSettings::ArgRequiredElseHelp)
        .version(env!("CARGO_PKG_VERSION"))
        .author(env!("CARGO_PKG_AUTHORS"))
        .about(env!("CARGO_PKG_DESCRIPTION"))                          
        .subcommand(SubCommand::with_name("default")
            .about("Create a default config file")
            .version(env!("CARGO_PKG_VERSION"))
            .arg(Arg::with_name("FILE_NAME")
                .help("Name of the default config file")
                .required(true)
                .index(1)))
        .subcommand(SubCommand::with_name("run")
            .about("Run from a given config file")
            .version(env!("CARGO_PKG_VERSION"))
            .arg(Arg::with_name("FILE_NAME")
                .help("Name of the config file to run from")
                .required(true)
                .index(1)))
        .get_matches();
    
    if let Some(matches) = matches.subcommand_matches("default") {
        let file = matches.value_of("FILE_NAME").unwrap();
        if let Err(err) = config::write_default(&file) {
            println!("Could not write default config! {}", err);
            return None;
        }
    }

    if let Some(matches) = matches.subcommand_matches("run") {
        let file = matches.value_of("FILE_NAME").unwrap();
        match config::load(&file) {
            Ok(config) => return Some(config),
            Err(err) => {
                println!("Error loading config file: '{}'! {}", &file, err);
                return None;
            }
        }
    }
    println!("Nothing to do ...");
    None
}
