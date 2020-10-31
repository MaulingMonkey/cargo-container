use mmrbi::env::*;



#[derive(Debug)]
pub struct Config(String);

impl Config {
    pub fn list() -> Vec<Config> { req_var_str("CARGO_CONTAINER_CONFIGS").split(',').map(|s| Config(s.into())).collect() }
    pub fn name(&self) -> &str { &self.0 }
}
