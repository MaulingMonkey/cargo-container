//! Builds a cargo workspace

use super::*;
use std::process::Command;

pub fn from_args(rest: impl Iterator<Item = String>) -> Result<(), Error> {
    let status = Command::new("cargo")
        .args(&["build"])
        .args(rest)
        .status();

    if let Ok(exit) = status.as_ref() {
        if exit.success() {
            return Ok(());
        }
    }

    Err(Error::SubExec("cargo build [...]".to_string(), status))
}
