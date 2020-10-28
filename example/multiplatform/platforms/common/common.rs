pub extern crate mmrbi;
use mmrbi::env::{req_var_str, req_var_path};
use std::path::PathBuf;



pub struct Config(String);
impl Config {
    pub fn list() -> Vec<Config> { req_var_str("CARGO_CONTAINER_CONFIGS").split(',').map(|s| Config(s.into())).collect() }
    pub fn name(&self) -> &str { &self.0 }
}


pub struct Package(String);
impl Package {
    pub fn list() -> Vec<Package> { req_var_str("CARGO_CONTAINER_PACKAGES").split(',').map(|p| Package(p.into())).collect() }
    pub fn original_name(&self)     -> &str     { &self.0 }
    pub fn original_path(&self)     -> PathBuf  { req_var_path(format!("CARGO_CONTAINER_PACKAGE_{}_PATH",           self.0)) }
    pub fn version(&self)           -> String   { req_var_str (format!("CARGO_CONTAINER_PACKAGE_{}_VERSION",        self.0)) }
    pub fn description(&self)       -> String   { req_var_str (format!("CARGO_CONTAINER_PACKAGE_{}_DESCRIPTION",    self.0)) }
    pub fn generated_path(&self)    -> PathBuf  { req_var_path("CARGO_CONTAINER_CRATES_DIR").join(&self.0) }
}
