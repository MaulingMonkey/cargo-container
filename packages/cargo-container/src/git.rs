//! Utilities for managing git integration.

use std::fs::{File};
use std::mem;
use std::path::Path;
use std::io::{self, BufRead, BufReader};


/// A `.git` directory.
pub struct DotDirectory<'a>(&'a Path);

impl DotDirectory<'_> {
    pub fn read_config(&self) -> io::Result<Config> {
        Config::read(&self.0.join("config"))
    }
}

/// A parsed `.git/config` [file](https://git-scm.com/docs/git-config#FILES).
/// 
/// See [git-config](https://git-scm.com/docs/git-config)
pub struct Config {
    pub remotes: Vec<Remote>,
}

impl Config {
    /// Parse a `.git/config` file.
    pub fn parse(input: impl BufRead) -> io::Result<Self> {
        let mut state = ConfigParserState::None;
        let mut config = Self {
            remotes: Vec::new()
        };

        for (line_index, line) in input.lines().enumerate() {
            let line = line?;
            if line.starts_with("[") && line.ends_with("]") {
                state.commit(&mut config);
                state.new_section(line);
            } else if line.starts_with("\t") {
                state.add_section(line);
            } else {
                for ch in line.chars() {
                    if !ch.is_ascii_whitespace() {
                        return Err(io::Error::new(io::ErrorKind::InvalidData, format!(".git/config:{} didn't parse", line_index + 1)));
                    }
                }
            }
        }

        state.commit(&mut config);
        Ok(config)
    }

    /// Read a `.git/config` file from disk.
    pub fn read(path: &impl AsRef<Path>) -> io::Result<Self> {
        Self::parse(BufReader::new(File::open(path)?))
    }
}

/// A `.git/config` file remote entry:
/// 
/// ```text
/// [remote "{name}"]
/// url    = {url}
/// fetch  = {fetch}
/// ```
/// 
/// See [git-config remote.<name>.*](https://git-scm.com/docs/git-config#Documentation/git-config.txt-remoteltnamegturl)
#[derive(Clone, Default)]
pub struct Remote {
    pub name:                   String,
    pub url:                    Option<String>, // url
    pub pushurl:                Option<String>, // url
    pub proxy:                  Option<String>, // url
    pub proxy_auth_methods:     Option<String>,
    pub fetch:                  Option<String>, // ref spec
    pub push:                   Option<String>, // ref spec
    pub mirror:                 Option<bool>,
    pub skip_default_update:    Option<bool>,
    pub skip_fetch_all:         Option<bool>,
    pub receivepack:            Option<String>,
    pub uploadpack:             Option<String>,
    pub tag_opt:                Option<String>,
    pub vcs:                    Option<String>, // git-remote-<vcs>
    pub prune:                  Option<bool>,
    pub prune_tags:             Option<bool>,

    _not_exhaustive:            (),
}



enum ConfigParserState {
    None,
    ParsingIgnored,
    ParsingRemote(Remote),
}

impl ConfigParserState {
    pub fn commit(&mut self, config: &mut Config) {
        match &mut mem::replace(self, ConfigParserState::None) {
            ConfigParserState::None => {},
            ConfigParserState::ParsingIgnored => {},
            ConfigParserState::ParsingRemote(remote) => {
                config.remotes.push(mem::replace(remote, Remote::default()));
            },
        };
    }

    pub fn new_section(&mut self, line: String) {
        debug_assert!(self.is_none());
        if let Some(remote) = match_section(line.as_str(), "[remote \"", "\"]") {
            *self = ConfigParserState::ParsingRemote(Remote {
                name: remote.to_string(),
                ..Remote::default()
            });
        } else {
            *self = ConfigParserState::ParsingIgnored;
        }
    }

    pub fn add_section(&mut self, line: String) {
        debug_assert!(!self.is_none());
        match self {
            ConfigParserState::ParsingRemote(remote) => {
                debug_assert!(line.starts_with("\t"));
                let line = &line[1..];
                let eq = if let Some(eq) = line.find("=") { eq } else { return; };
                let (key, value) = line.split_at(eq);
                let key     = key.trim();
                let value   = value[1..].trim();

                match key {
                    "url"                   => remote.url                   = Some(value.to_string()),
                    "pushurl"               => remote.pushurl               = Some(value.to_string()),
                    "proxy"                 => remote.proxy                 = Some(value.to_string()),
                    "proxyAuthMethod"       => remote.proxy_auth_methods    = Some(value.to_string()),
                    "fetch"                 => remote.fetch                 = Some(value.to_string()),
                    "push"                  => remote.push                  = Some(value.to_string()),
                    "mirror"                => remote.mirror                = Some(value == "true"),
                    "skipDefaultUpdate"     => remote.skip_default_update   = Some(value == "true"),
                    "skipFetchAll"          => remote.skip_fetch_all        = Some(value == "true"),
                    "receivepack"           => remote.receivepack           = Some(value.to_string()),
                    "uploadpack"            => remote.uploadpack            = Some(value.to_string()),
                    "tagOpt"                => remote.tag_opt               = Some(value.to_string()),
                    "vcs"                   => remote.vcs                   = Some(value.to_string()),
                    "prune"                 => remote.prune                 = Some(value == "true"),
                    "pruneTags"             => remote.prune_tags            = Some(value == "true"),
                    _ => {}, // not yet supported
                }
            },
            _ => {},
        }
    }

    fn is_none(&self) -> bool {
        match self {
            ConfigParserState::None => true,
            _ => false,
        }
    }
}

impl Drop for ConfigParserState {
    fn drop(&mut self) {
        debug_assert!(self.is_none());
    }
}

fn match_section<'a>(line: &'a str, prefix: &str, postfix: &str) -> Option<&'a str> {
    if line.starts_with(prefix) && line.ends_with(postfix) {
        Some(&line[prefix.len()..(line.len() - postfix.len())])
    } else {
        None
    }
}

#[test]
fn test_read_config() {
    let git = DotDirectory(Path::new(".git"));
    if !git.0.exists() { return; } // ???!?!?!!!? (.zip download?)

    let config = git.read_config().unwrap();
    assert!(config.remotes.len() > 0);
    for remote in config.remotes.iter() {
        match &remote.url {
            Some(url) => assert!(
                url.starts_with("git@github.com:") || url.starts_with("https://github.com/"),
                "A remote to MaulingMonkey's upstream github exists, right?  For bugfixes at least?"
            ),
            None => panic!("All git remotes for this repository should probably have a url"),
        }
    }
}
