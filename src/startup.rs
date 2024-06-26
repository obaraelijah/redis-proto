use slog::info;
use structopt::StructOpt;

use crate::logger::LOGGER;
use std::path::PathBuf;

#[derive(Debug, StructOpt)]
#[structopt(
    name = "redis-proto",
    about = "A multi-threaded implementation of redis written in rust 🦀"
)]
pub struct Config {
    /// Database Dump File Directory Location
    #[structopt(short = "d", long = "dump-file", parse(from_os_str))]
    pub data_dir: Option<PathBuf>,
    /// Don't show the starting graphic
    #[structopt(short = "g", long = "no-graphic")]
    pub dont_show_graphic: bool,
    #[structopt(short = "s", long = "ops-until-save", default_value = "10000")]
    pub ops_until_save: u64,
    #[structopt(short = "p", long = "port", default_value = "6379")]
    pub port: u64,
    /// Run in memory only mode. Don't save database state to disk
    #[structopt(short = "m", long = "memory-only")]
    pub memory_only: bool,
    #[structopt(short = "f", long = "scripts-dir")]
    pub scripts_dir: Option<std::path::PathBuf>,
}

pub fn startup_message(config: &Config) {
    if !config.dont_show_graphic {
        info!(
            LOGGER,
            r#"
 ___               _              ___                _          
 |  _`\            ( ) _          (  _`\             ( )_        
 | (_) )   __     _| |(_)  ___    | |_) ) _ __   _   | ,_)   _   
 | ,  /  /'__`\ /'_` || |/',__)   | ,__/'( '__)/'_`\ | |   /'_`\ 
 | |\ \ (  ___/( (_| || |\__, \   | |    | |  ( (_) )| |_ ( (_) )
 (_) (_)`\____)`\__,_)(_)(____/   (_)    (_)  `\___/'`\__)`\___/'
                                                                                                                                
        "#
        );
    }
    info!(LOGGER, "Redis Proto starting...");
}
