#![deny(unsafe_code)]

#[macro_use]
extern crate serde_derive;

#[macro_use]
extern crate slog;

#[macro_use]
extern crate combine;

extern crate rmp_serde as rmps;

use structopt::StructOpt;

mod asyncresp;
mod blocking;
mod bloom;
mod database;
mod hashes;
mod keys;
mod lists;
mod logger;
#[macro_use]
mod macros;
mod data_structures;
mod misc;
mod ops;
mod server;
mod sets;
mod sorted_sets;
mod startup;
mod state;
mod types;

use self::database::{get_dump_file, load_state};
use self::logger::LOGGER;
// use self::server::server;
use self::startup::{startup_message, Config};
use self::server::socket_listener;
use self::database::save_state_interval;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let opt = Config::from_args();
    startup_message(&opt);
    info!(LOGGER, "Initializing State...");
    info!(LOGGER, "Opening Datafile...");
    let dump_file = get_dump_file(&opt);
    let state = load_state(dump_file.clone(), &opt)?;
    info!(LOGGER, "Starting Server...");
    tokio::spawn(save_state_interval(state.clone(), dump_file.clone()));
    socket_listener(state.clone(), dump_file.clone(), opt).await;
    Ok(())
}
