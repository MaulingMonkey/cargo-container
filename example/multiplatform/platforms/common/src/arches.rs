use mmrbi::env::*;

use std::collections::BTreeSet;



#[derive(Debug)]
pub struct Arches(BTreeSet<String>);

impl Arches {
    pub fn get() -> Self {
        Self(match opt_var_lossy("CARGO_CONTAINER_ARCHES") {
            None                => Default::default(),
            Some(s) if s == ""  => Default::default(),
            Some(s)             => s.split(',').map(String::from).collect()
        })
    }

    pub fn contains(&self, arch: &str) -> Option<bool> {
        if      self.0.contains("*")    { Some(true) }
        else if self.0.is_empty()       { None }
        else                            { Some(self.0.contains(arch)) }
    }
}
