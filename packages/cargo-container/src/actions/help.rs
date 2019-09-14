//! Print help text

use super::*;

/// Print usage help text
pub fn usage() {
    // XXX: This should be dynamically driven in case you want to add/remove your own subcommands
    println!("Usage: cargo container [subcommand] [..args..]");
    println!("");
    println!("Subcommands:");
    println!("    help      Print this help information");
    println!("    build     Invoke cargo build, or appropriate platform specific wrapper commands");
    println!("    test      Invoke cargo test, or appropriate platform specific wrapper commands");
    println!("");
}

pub fn from_args(_rest: impl Iterator<Item = String>) -> Result<(), Error> {
    Ok(usage())
}
