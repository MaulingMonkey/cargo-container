use mmrbi::*;

use std::collections::BTreeSet;

#[derive(Default)]
pub struct Args {
    pub arches:     BTreeSet<String>,
    pub configs:    BTreeSet<String>,
    pub crates:     BTreeSet<String>,
    pub tools:      BTreeSet<String>,
    pub allow_sudo: Option<bool>,
}

impl Args {
    pub fn from(mut args: std::env::ArgsOs) -> Self {
        let mut o = Self::default();
        while let Some(arg) = args.next() {
            let arg = arg.to_string_lossy();
            match &*arg {
                flag @ "--arch"     => add_arg(&mut o.arches,   flag, "architecture",   &mut args),
                flag @ "--config"   => add_arg(&mut o.configs,  flag, "configuration",  &mut args),
                flag @ "--crate"    => add_arg(&mut o.crates,   flag, "crate",          &mut args),
                flag @ "--tool"     => add_arg(&mut o.tools,    flag, "tool",           &mut args),
                "--allow-sudo"      => o.allow_sudo = Some(true),
                "--deny-sudo"       => o.allow_sudo = Some(false),

                flag if flag.starts_with("-") => fatal!("unrecognized flag: {}", flag),
                other => fatal!("unrecognized argument: {}", other),
            }
        }
        if o.configs.is_empty() { o.configs.insert(String::from("debug")); }
        o
    }
}

fn add_arg(o: &mut BTreeSet<String>, flag: &str, param: &str, args: &mut std::env::ArgsOs) {
    let next = args.next().unwrap_or_else(|| fatal!("expected {} after {}", param, flag)).to_string_lossy().into_owned();
    if let Some(prev) = o.replace(next) {
        warning!("{} {} was already specified", flag, prev);
    }
}
