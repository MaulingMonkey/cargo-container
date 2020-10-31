use mmrbi::env::*;

use std::path::PathBuf;



#[derive(Debug)]
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
