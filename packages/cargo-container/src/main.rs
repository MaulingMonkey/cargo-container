use cargo_container::*;
use cargo_metadata::*;
use std::ffi::OsString;
use std::process::{Command, exit};

fn main() {
    let metadata : Metadata = match MetadataCommand::new().exec() {
        Ok(m) => m,
        Err(err) => {
            eprintln!();
            eprintln!("{}", err);
            eprintln!();
            exit(1);
        },
    };

    // Skip: "cargo-container.exe", "container"
    let rest = || std::env::args().skip(2);

    // Order here is important and defines ccs script search order / priority.
    exec_cargo_container_script_package(&metadata, rest());
    exec_cargo_container_script_script(&metadata, rest());
    exec_builtin_default_script(metadata, rest());
}

/// If the cargo workspace contains a `cargo-container-script` package, execute that and exit.
fn exec_cargo_container_script_package(metadata: &Metadata, rest: impl Iterator<Item = String>) {
    let ccs_id = PackageId { repr: "cargo-container-script".to_string() };
    if !metadata.workspace_members.contains(&ccs_id) { return; }

    move || -> ! {
        // XXX: Consider using exec* on *nix for better debug/error forwarding.
        let status = Command::new("cargo")
            .args("run -p cargo-container-script --".split_ascii_whitespace())
            .args(rest)
            .status();

        match status {
            Ok(exit_status) => {
                match exit_status.code() {
                    Some(0) => exit(0),
                    Some(code) => {
                        eprintln!();
                        eprintln!("cargo-container-script failed with exit code: {:?}", code);
                        eprintln!();
                        exit(code);
                    },
                    None => {
                        eprintln!();
                        eprintln!("cargo-container-script failed: {:?}", exit_status);
                        eprintln!();
                        exit(1);
                    },
                }
            },
            Err(io) => {
                eprintln!();
                eprintln!("cargo-container-script exists, but failed to start \"cargo run -p cargo-container-script -- ...\":");
                eprintln!("    {:?}", io);
                eprintln!();
                exit(1);
            },
        }
    }();
}

/// If the cargo workspace is alongside a `cargo-container-script.rs` script, execute that and exit.
fn exec_cargo_container_script_script(metadata: &Metadata, rest: impl Iterator<Item = String>) {
    let ccs_rs = metadata.workspace_root.join("cargo-container-script.rs");
    if !ccs_rs.exists() { return; }

    move || -> ! {
        let deps_dir    = metadata.target_directory.join("debug/deps");
        let inc_dir     = metadata.target_directory.join("debug/incremental");
        let out_dir     = metadata.target_directory.join("debug/build/cargo-container-script");
        let script_exe  = out_dir.join("debug/build/cargo-container-script/cargo-container-script.exe");

        let status = Command::new("cargo")
            .args("rustc --edition=2018 --crate-name cargo-container-script".split_ascii_whitespace())
            .arg(ccs_rs)
            .args("--color always --crate-type bin --emit=dep-info,link -C debuginfo=2 --out-dir".split_ascii_whitespace())
            .arg(out_dir)
            .arg("-C").arg(os_join("incremental=", inc_dir))
            .arg("-C").arg(os_join("dependency=", deps_dir))
            .status();

        match status {
            Ok(exit_status) => {
                match exit_status.code() {
                    Some(0) => {},
                    Some(code) => {
                        eprintln!();
                        eprintln!("rustc cargo-container-script.rs [...] failed with exit code: {:?}", code);
                        eprintln!();
                        exit(code);
                    },
                    None => {
                        eprintln!();
                        eprintln!("rustc cargo-container-script.rs [...] failed: {:?}", exit_status);
                        eprintln!();
                        exit(1);
                    },
                }
            },
            Err(io) => {
                eprintln!();
                eprintln!("cargo-container-script.rs exists, but failed to start \"rustc cargo-container-script.rs [...]\":");
                eprintln!("    {:?}", io);
                eprintln!();
                exit(1);
            },
        }

        // XXX: Consider using exec* on *nix for better debug/error forwarding.
        let status = Command::new(script_exe)
            .args(rest)
            .status();

        match status {
            Ok(exit_status) => {
                match exit_status.code() {
                    Some(0) => exit(0),
                    Some(code) => {
                        eprintln!();
                        eprintln!("rustc cargo-container-script.rs [...] failed with exit code: {:?}", code);
                        eprintln!();
                        exit(code);
                    },
                    None => {
                        eprintln!();
                        eprintln!("rustc cargo-container-script.rs [...] failed: {:?}", exit_status);
                        eprintln!();
                        exit(1);
                    },
                }
            },
            Err(io) => {
                eprintln!();
                eprintln!("cargo-container-script.rs exists, but failed to start \"rustc cargo-container-script.rs [...]\":");
                eprintln!("    {:?}", io);
                eprintln!();
                exit(1);
            },
        }
    }();
}

/// If no script was found, try to do the right thing.
fn exec_builtin_default_script(_metadata: Metadata, rest: impl Iterator<Item = String>) -> ! {
    match actions::default::from_args(rest) {
        Ok(()) => {
            exit(0);
        },
        Err(err) => {
            eprintln!();
            eprintln!("{}", err);
            eprintln!();
            actions::help::usage();
            exit(1);
        },
    }
}

fn os_join(a: impl Into<OsString>, b: impl Into<OsString>) -> OsString {
    let mut r = a.into();
    r.push(b.into());
    r
}
