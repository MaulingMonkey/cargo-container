// TODO: upstream this?

#[cfg(windows)] use bstr::BString;
#[cfg(windows)] mod bstr;
#[cfg(windows)] use var::Variant;
#[cfg(windows)] mod var;
#[cfg(windows)] pub(crate) mod wsl;
mod ver;
pub use ver::version;

pub mod features;
