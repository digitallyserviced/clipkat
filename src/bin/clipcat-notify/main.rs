use structopt::StructOpt;
#[macro_use]
extern crate serde;

#[macro_use]
extern crate snafu;

use std::sync::atomic;

mod command;
mod config;
mod error;

use self::command::Command;

pub static SHUTDOWN: atomic::AtomicBool = atomic::AtomicBool::new(false);

fn main() {
    let cmd = Command::from_args();
    if let Err(err) = cmd.run() {
        eprintln!("Error: {}", err);
        std::process::exit(1);
    }
}
