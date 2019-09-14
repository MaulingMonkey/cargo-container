use std::fmt::{self, Debug, Display, Formatter};
use std::io;
use std::process::ExitStatus;

pub enum Error {
    SubExec(String, io::Result<ExitStatus>),
    NotYetImplemented(String),
    NoSuchSubcommand(String),

    #[doc(hidden)] _NonExhaustive,
}

impl std::error::Error for Error {}

impl Display for Error {
    fn fmt(&self, fmt: &mut Formatter) -> fmt::Result {
        match self {
            Error::SubExec(command, exit_status)    => write!(fmt, "Failed to invoke subcommand\nCommand:   {}\nExit status:  {:?}", command, exit_status),
            Error::NotYetImplemented(what)          => write!(fmt, "Not yet implemented: {}", what),
            Error::NoSuchSubcommand(subcommand)     => write!(fmt, "Not such subcommand: {}", subcommand),
            Error::_NonExhaustive                   => panic!("Do not use Error::_NonExhaustive"),
        }
    }
}

impl Debug for Error {
    fn fmt(&self, fmt: &mut Formatter) -> fmt::Result {
        Display::fmt(self, fmt)
    }
}
