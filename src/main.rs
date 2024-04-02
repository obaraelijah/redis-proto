use redis_proto::logger::LOGGER;
use redis_proto::startup::{startup_message, Config};

use slog::info;
use structopt::StructOpt;

#[tokio::main]
async fn main() {
    // Get the args
    let opt = Config::from_args();
    // print the fancy logo
    startup_message(&opt);
    // database state
    info!(LOGGER, "Initializing state...");
    todo!()
}
