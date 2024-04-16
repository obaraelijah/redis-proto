use redis_proto::database::{get_dump_file, load_state, save_state_interval};
use redis_proto::logger::LOGGER;
use redis_proto::startup::{startup_message, Config};
use redis_proto::server::socket_listener;

use slog::{info, warn};
use structopt::StructOpt;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Get the args
    let opt = Config::from_args();
    // print the fancy logo
    startup_message(&opt);
    // Get the database file, making folders if necessary.
    info!(LOGGER, "Initializing state...");
    let dump_file = get_dump_file(&opt);
    //  Load database state if it exists.
    info!(LOGGER, "Opening Datafile...");
    let state = load_state(dump_file.clone(), &opt)?;
    // Spawn the save occassionally service
    info!(LOGGER, "Starting Server...");
    if !opt.memory_only {
        info!(LOGGER, "Spawning database saving task...");
        tokio::spawn(save_state_interval(state.clone(), dump_file.clone()));
    } else {
        warn!(
            LOGGER,
            "Database is in memory-only mode. STATE WILL NOT BE SAVED!"
        );
    }

    socket_listener(state.clone(), dump_file.clone(), opt).await;
    Ok(())
}
