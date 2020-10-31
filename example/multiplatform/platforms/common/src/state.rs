use super::{Arches, Config, Package};

use mmrbi::*;



pub struct State {
    pub command:    String,
    pub packages:   Vec<Package>,
    pub configs:    Vec<Config>,
    pub arches:     Arches,
}

impl State {
    pub(crate) fn get(suffix: &str) -> Self {
        let command     = env::var_str("CARGO_CONTAINER_COMMAND").unwrap_or_else(|err| fatal!("{}: did you not run this via `cargo container`?", err));
        let configs     = Config::list();
        let packages    = Package::list(suffix);
        let arches      = Arches::get();
        Self { command, packages, configs, arches }
    }
}
