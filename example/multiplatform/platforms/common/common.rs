pub extern crate mmrbi;
use mmrbi::*;
use mmrbi::env::{req_var_str, req_var_path};
use std::path::PathBuf;



pub fn exec(tool: impl Tool, suffix: &str) {
    let state = State::get(suffix);
    let state = &state;
    match state.command.as_str() {
        "bench"     => tool.bench   (state),
        "build"     => tool.build   (state),
        "clean"     => tool.clean   (state),
        "doc"       => tool.doc     (state),
        "fetch"     => tool.fetch   (state),
        "generate"  => tool.generate(state),
        "run"       => tool.run     (state),
        "package"   => tool.package (state),
        "test"      => tool.test    (state),
        "update"    => tool.update  (state),
        _other      => exit::command_not_implemented(),
    }
}



pub trait Tool {
    fn bench    (&self, _state: &State) { exit::command_not_implemented() }
    fn build    (&self, _state: &State) { exit::command_not_implemented() }
    fn clean    (&self, _state: &State) { exit::command_not_implemented() }
    fn doc      (&self, _state: &State) { exit::command_not_implemented() }
    fn fetch    (&self, _state: &State) { exit::command_not_implemented() }
    fn generate (&self, _state: &State) { exit::command_not_implemented() }
    fn run      (&self, _state: &State) { exit::command_not_implemented() }
    fn package  (&self, _state: &State) { exit::command_not_implemented() }
    fn test     (&self, _state: &State) { exit::command_not_implemented() }
    fn update   (&self, _state: &State) { exit::command_not_implemented() }
}



pub struct State {
    pub command:    String,
    pub packages:   Vec<Package>,
    pub configs:    Vec<Config>,
}
impl State {
    fn get(suffix: &str) -> Self {
        let command     = env::var_str("CARGO_CONTAINER_COMMAND").unwrap_or_else(|err| fatal!("{}: did you not run this via `cargo container`?", err));
        let configs     = Config::list();
        let packages    = Package::list(suffix);
        Self { command, packages, configs }
    }
}



pub struct Config(String);
impl Config {
    pub fn list() -> Vec<Config> { req_var_str("CARGO_CONTAINER_CONFIGS").split(',').map(|s| Config(s.into())).collect() }
    pub fn name(&self) -> &str { &self.0 }
}


pub struct Package {
    // e.g. "alpha"
    name:           String,
    // e.g. "alpha-windows"
    name_suffix:    String,
}
impl Package {
    pub fn list(suffix: &str) -> Vec<Package> {
        req_var_str("CARGO_CONTAINER_PACKAGES").split(',').map(|p| Package {
            name_suffix:    format!("{}-{}", p, suffix),
            name:           p.into(),
        }).collect()
    }

    pub fn original_name(&self)     -> &str     { &self.name }
    pub fn original_path(&self)     -> PathBuf  { req_var_path(format!("CARGO_CONTAINER_PACKAGE_{}_PATH",           self.name)) }
    pub fn version(&self)           -> String   { req_var_str (format!("CARGO_CONTAINER_PACKAGE_{}_VERSION",        self.name)) }
    pub fn description(&self)       -> String   { req_var_str (format!("CARGO_CONTAINER_PACKAGE_{}_DESCRIPTION",    self.name)) }
    pub fn generated_name(&self)    -> &str     { &self.name_suffix }
    pub fn generated_path(&self)    -> PathBuf  { req_var_path("CARGO_CONTAINER_CRATES_DIR").join(&self.name) }
}



pub mod exit {
    pub fn errors()                     -> ! { std::process::exit(0xCCEEEE) } // E = E = Error
    pub fn warnings()                   -> ! { std::process::exit(0xCC3333) } // 3 = W = Warning
    pub fn command_not_implemented()    -> ! { std::process::exit(0xCC0C21) } // C21 = CNI = Command Not Implemented
    pub fn platform_not_implemented()   -> ! { std::process::exit(0xCC0921) } // 921 = PNI = Platform Not Implemented
}
