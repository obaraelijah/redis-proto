use crate::logger::LOGGER;
use crate::startup::Config;
use crate::types::{Dumpfile, StateRef, StateStoreRef};
use directories::ProjectDirs;
use parking_lot::Mutex;
use std::error::Error;
use std::fs::{File, OpenOptions};
use std::io::{Seek,SeekFrom};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Duration;
use tokio::{task, time::Interval};
use rmp_serde as rmps;
use slog::error;


/// Convenience macro to panic with error messages.
macro_rules! fatal_panic {
    ($msg:expr) => {{
        error!(LOGGER, "{}", $msg);
        println!("{}", $msg);
        panic!("Fatal Error, cannot continue...");
    }};
    ($msg:expr, $err:expr) => {{
        error!(LOGGER, "{} {}", $msg, $err);
        println!("{}", $msg);
        panic!("Fatal Error, cannot continue...");
    }};
}


/// Dump the current state to the dump_file
fn dump_state(state: StateStoreRef, dump_file: &mut File) -> Result<(), Box<dyn Error>> {
    dump_file.seek(SeekFrom::Start(0))?;
    rmps::encode::write(dump_file, &state)
        .map_err(|e| fatal_panic!("Could not write state!", e.to_string()))
        .unwrap();
    Ok(())
}

/// Load state from the dump_file
fn load_state() -> {

}