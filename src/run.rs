use crate::*;

use mmrbi::*;

use std::ffi::*;
use std::path::Path;
use std::process::Command;



pub fn run() {
    let mut args = std::env::args_os();
    let _exe = args.next();
    if args.next().as_ref().and_then(|s| s.to_str()) != Some("container") {
        args = std::env::args_os();
        let _exe = args.next();
    }

    let meta = ContainerToml::from_current_dir().unwrap_or_else(|err| fatal!("{}", err));
    std::env::set_current_dir(meta.root_directory()).unwrap();

    generate::workspace_toml(&meta);

    match args.next() {
        Some(cmd) => {
            let cmd = cmd.to_string_lossy();
            let cmd = &*cmd;
            match cmd {
                "help"                  => fatal!("not yet implemented: {}", cmd),
                "version"               => fatal!("not yet implemented: {}", cmd),
        
                // Build Commands
                "bench"                 => fatal!("not yet implemented: {}", cmd),
                "build" | "b"           => build(&meta, args),
                "check" | "c"           => fatal!("not yet implemented: {}", cmd),
                "clean"                 => clean(&meta, args),
                "doc"                   => fatal!("not yet implemented: {}", cmd),
                "fetch"                 => fatal!("not yet implemented: {}", cmd),
                // fix
                "run" | "r"             => fatal!("not yet implemented: {}", cmd),
                // rustc
                // rustdoc
                "test" | "t"            => test(&meta, args),
        
                // Manifest Commands
                "generate-lockfile"     => fatal!("not yet implemented: {}", cmd),
                //    locate-project
                //    metadata
                //    pkgid
                //    tree
                "update"                => fatal!("not yet implemented: {}", cmd),
                "vendor"                => fatal!("not yet implemented: {}", cmd),
                //    verify-project
        
                // Package Commands
                "init"                  => fatal!("not yet implemented: {}", cmd),
                "install"               => fatal!("not yet implemented: {}", cmd),
                "new"                   => fatal!("not yet implemented: {}", cmd),
                // search
                "uninstall"             => fatal!("not yet implemented: {}", cmd),
        
                // Publishing Commands
                //  login
                //  owner
                "package"               => fatal!("not yet implemented: {}", cmd),
                "publish"               => fatal!("not yet implemented: {}", cmd),
                //  yank
        
                // Misc.
                other                   => fatal!("unrecognized subcommand: {}", other),
            }
        },
        None => fatal!("expected subcommand"),
    }
}

fn build(meta: &ContainerToml, args: std::env::ArgsOs) {
    generate::dot_container(meta);
    generate::workspace_toml(meta);
    local_install(meta);
    generate::crates(meta);
    let _ = args; // XXX: so much

    let path = prepend_paths(Some(Path::new("bin").canonicalize().unwrap().cleanup()));

    for build in meta.builds.iter() {
        for tool in build.tools.iter() {
            for config in ["debug", "release"].iter().cloned() { // XXX
                eprintln!();
                status!("Building", "{} | {} | {} crates", tool, config, build.crates.len());
                let mut cmd = Command::new(tool.as_str());
                cmd.env("PATH",                         &path);
                cmd.env("CARGO_CONTAINER_COMMAND",      "build");
                cmd.env("CARGO_CONTAINER_CRATES_DIR",   format!(".container/crates/{}", tool));
                cmd.env("CARGO_CONTAINER_CONFIGS",      config);
                cmd.env("CARGO_CONTAINER_PACKAGES",     build.crates.iter().map(|a| a.as_str()).collect::<Vec<_>>().join(","));
                cmd.status0().unwrap_or_else(|err| fatal!("`{}` build failed: {}", tool, err));
            }
        }
    }
}

fn clean(meta: &ContainerToml, args: std::env::ArgsOs) {
    // XXX: cleanup bin dir?
    let dot_container = meta.root_directory().join(".container");
    Command::new("cargo").arg("clean").args(args).current_dir(meta.root_directory()).status0().unwrap_or_else(|err| fatal!("`cargo clean` failed: {}", err));
    std::fs::remove_dir_all(&dot_container).unwrap_or_else(|err| fatal!("`cargo container clean` failed to delete `{}`: {}", dot_container.display(), err));
}

fn test(meta: &ContainerToml, args: std::env::ArgsOs) {
    generate::dot_container(meta);
    generate::workspace_toml(meta);
    local_install(meta);
    generate::crates(meta);
    let _ = args; // XXX: so much

    let path = prepend_paths(Some(Path::new("bin").canonicalize().unwrap().cleanup()));

    for build in meta.builds.iter() {
        for tool in build.tools.iter() {
            for config in ["debug", "release"].iter().cloned() { // XXX
                eprintln!();
                status!("Testing", "{} | {} | {} crates", tool, config, build.crates.len());
                let mut cmd = Command::new(tool.as_str());
                cmd.env("PATH",                         &path);
                cmd.env("CARGO_CONTAINER_COMMAND",      "test");
                cmd.env("CARGO_CONTAINER_CRATES_DIR",   format!(".container/crates/{}", tool));
                cmd.env("CARGO_CONTAINER_CONFIGS",      config);
                cmd.env("CARGO_CONTAINER_PACKAGES",     build.crates.iter().map(|a| a.as_str()).collect::<Vec<_>>().join(","));
                cmd.status0().unwrap_or_else(|err| fatal!("`{}` test failed: {}", tool, err));
            }
        }
    }
}

fn local_install(meta: &ContainerToml) {
    if meta.local_install.is_empty() { return }
    cargo_local_install::run_from_strs(vec![
        OsStr::new("--no-path-warning"),
        //OsStr::new("--root"), meta.root_directory().join(".container").as_os_str(),
    ].into_iter()).unwrap_or_else(|err| fatal!("cargo-local-install failed: {}", err));
}
