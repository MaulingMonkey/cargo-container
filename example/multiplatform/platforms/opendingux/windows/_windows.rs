// TODO: upstream this?

#[cfg(windows)] use bstr::BString;
#[cfg(windows)] mod bstr;
#[cfg(windows)] use var::Variant;
#[cfg(windows)] mod var;

pub mod features;

pub use winrt::windows::*;
