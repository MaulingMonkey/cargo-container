pub extern crate cargo_local_install;
pub extern crate mmrbi;

use sha2::Digest;

use mmrbi::*;
use mmrbi::env::*;

use std::collections::BTreeSet;
use std::fmt::{Write as _ };
use std::io::{Read, Write};
use std::path::{Path, PathBuf};



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
        "setup"     => tool.setup   (state),
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
    fn setup    (&self, _state: &State) { exit::command_not_implemented() }
    fn test     (&self, _state: &State) { exit::command_not_implemented() }
    fn update   (&self, _state: &State) { exit::command_not_implemented() }
}



pub struct State {
    pub command:    String,
    pub packages:   Vec<Package>,
    pub configs:    Vec<Config>,
    pub arches:     Arches,
}
impl State {
    fn get(suffix: &str) -> Self {
        let command     = env::var_str("CARGO_CONTAINER_COMMAND").unwrap_or_else(|err| fatal!("{}: did you not run this via `cargo container`?", err));
        let configs     = Config::list();
        let packages    = Package::list(suffix);
        let arches      = Arches::get();
        Self { command, packages, configs, arches }
    }
}



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



#[derive(Debug)]
pub struct Config(String);
impl Config {
    pub fn list() -> Vec<Config> { req_var_str("CARGO_CONTAINER_CONFIGS").split(',').map(|s| Config(s.into())).collect() }
    pub fn name(&self) -> &str { &self.0 }
}


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



pub mod exit {
    pub fn errors()                     -> ! { std::process::exit(0xEE) } // EE = EE = Errors
    pub fn warnings()                   -> ! { std::process::exit(0x33) } // 33 = WW = Warnings
    pub fn command_not_implemented()    -> ! { std::process::exit(0xC1) } // C1 = CnI = Command Not Implemented
    pub fn platform_not_implemented()   -> ! { std::process::exit(0x91) } // 91 = PnI = Platform Not Implemented
}



pub struct Download {
    pub name:   &'static str,
    pub url:    &'static str,
    pub sha256: &'static str,
}

impl Download {
    pub fn download(&self) -> impl AsRef<[u8]> {
        status!("Downloading", "{} ({})", self.name, self.url);
        let download = reqwest::blocking::Client::builder()
            .user_agent("github.com/MaulingMonkey/cargo-container/example/multiplatform/platforms/common")
            .build().or_die()
            .get(self.url)
            .send().or_die()
            .bytes().or_die();

        let mut hasher = sha2::Sha256::new();
        hasher.update(download.as_ref());
        let mut hash = String::new();
        for b in hasher.finalize().into_iter() {
            let _ = write!(&mut hash, "{:02X}", b);
        }

        if self.sha256 != hash {
            fatal!("expected hash {}\r\nbut got hash {}", self.sha256, hash);
        }

        download
    }

    pub fn download_gunzip(&self) -> impl AsRef<[u8]> {
        let mut o = Vec::new();
        libflate::gzip::Decoder::new(self.download().as_ref())
            .unwrap_or_else(|err| fatal!("failed to gunzip: {}", err))
            .read_to_end(&mut o)
            .unwrap_or_else(|err| fatal!("failed to gunzip: {}", err));
        o
    }

    pub fn download_gunzip_to(&self, to: impl AsRef<Path>, _unix_mode: u32) {
        let mut o = std::fs::File::create(to.as_ref()).unwrap_or_else(|err| fatal!("failed to create {}: {}", to.as_ref().display(), err));
        o.write_all(self.download_gunzip().as_ref()).unwrap_or_else(|err| fatal!("failed to write to {}: {}", to.as_ref().display(), err));
        #[cfg(unix)] {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = o.metadata().unwrap_or_else(|err| fatal!("failed to get permissions for {}: {}", to.as_ref().display(), err)).permissions();
            perms.set_mode(_unix_mode);
            o.set_permissions(perms).unwrap_or_else(|err| fatal!("failed to set permissions for {}: {}", to.as_ref().display(), err));
        }
    }

    pub fn download_gunzip_untar_entry_to(&self, entry: impl AsRef<Path>, to: impl AsRef<Path>, _unix_mode: u32) {
        let mut tar = tar::Archive::new(std::io::Cursor::new(self.download_gunzip()));
        for e in tar.entries().unwrap_or_else(|err| fatal!("failed to read tar entries: {}", err)) {
            let mut e = e.unwrap_or_else(|err| fatal!("failed to read tar entry: {}", err));
            let path = e.path().unwrap_or_else(|err| fatal!("failed to read tar entry path: {}", err));
            if path == entry.as_ref() {
                e.unpack(to.as_ref()).unwrap_or_else(|err| fatal!("failed to unpack {} to {}: {}", entry.as_ref().display(), to.as_ref().display(), err));
                #[cfg(unix)] {
                    use std::os::unix::fs::PermissionsExt;
                    let mut perms = to.as_ref().metadata().unwrap_or_else(|err| fatal!("failed to get permissions for {}: {}", to.as_ref().display(), err)).permissions();
                    perms.set_mode(_unix_mode);
                    std::fs::set_permissions(to.as_ref(), perms).unwrap_or_else(|err| fatal!("failed to set permissions for {}: {}", to.as_ref().display(), err));
                }
                return;
            }
        }
        fatal!("unable to find {} in archive", entry.as_ref().display());
    }
}
