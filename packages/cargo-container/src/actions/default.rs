//! Not a subcommand/action in and of itself, but rather describes the default behavior of cargo-container.

use super::*;

/// Select a default build action from command line arguments.
/// 
/// **NOTE**:  Don't just pass `std::env::args()` !  You're expected to remove:
/// 1. The executable name (typically `args[0]`)
/// 2. For cargo subcommands, the cargo subcommand name (typically `args[1]`)
/// 
/// # Examples
/// 
/// ```no_run
/// use cargo_container as container;
/// 
/// // standalone-bin.rs
/// container::actions::default::from_args(std::env::args().skip(1));
/// // skip(1): skips "standalone-bin.exe"
/// 
/// // cargo-container.rs
/// container::actions::default::from_args(std::env::args().skip(2));
/// // skip(2): skips "cargo-container.exe", "container"
/// ```
pub fn from_args(mut args: impl Iterator<Item = String>) -> Result<(), Error> {
    let subcommand = args.next();
    match subcommand.as_ref().map(|s| s.as_str()) {
        Some("build")   => actions::build::from_args(args),
        Some("help")    => actions::help::from_args(args),
        Some("test")    => actions::test::from_args(args),
        Some(_)         => Err(Error::NoSuchSubcommand(subcommand.unwrap())),
        None            => Ok(actions::help::usage()),
    }
}
