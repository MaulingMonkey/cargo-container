use crate::*;

use mmrbi::*;

use std::collections::BTreeSet;
use std::ffi::*;
use std::io;
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

    let cmd = args.next().unwrap_or_else(|| fatal!("expected subcommand"));
    let cmd = cmd.to_string_lossy();
    let cmd = &*cmd;

    match cmd {
        // Metadata Commands
        "help"                  => help(args),
        "version"               => version(args),

        // General Commands
        "bench"                 => gen_then_fwd(&meta, args, cmd, false, "Benchmarking"),
        "build" | "b"           => gen_then_fwd(&meta, args, cmd, false, "Building"),
        "check" | "c"           =>{gen_then_fwd(&meta, args, cmd, true,  "Checking"); Command::new("cargo").arg("check").status0().or_die(); },
        "clean"                 => clean(&meta, args),
        "doc"                   => gen_then_fwd(&meta, args, cmd, false, "Documenting"),
        "fetch"                 =>{gen_then_fwd(&meta, args, cmd, true,  "Fetching"); Command::new("cargo").arg("fetch").status0().or_die(); }
        "fuzz"                  => gen_then_fwd(&meta, args, cmd, false, "Fuzzing"),
        "package"               => gen_then_fwd(&meta, args, cmd, false, "Packaging"),
        "run" | "r"             => gen_then_fwd(&meta, args, cmd, false, "Running"), // XXX: Is this what we actually want?
        "test" | "t"            => gen_then_fwd(&meta, args, cmd, false, "Testing"),
        "update"                => gen_then_fwd(&meta, args, cmd, false, "Updating"),

        // NYI commands
        "generate-lockfile"     => fatal!("not yet implemented: {}", cmd),
        "vendor"                => fatal!("not yet implemented: {}", cmd),
        "init"                  => fatal!("not yet implemented: {}", cmd),
        "install"               => fatal!("not yet implemented: {}", cmd),
        "new"                   => fatal!("not yet implemented: {}", cmd),
        "uninstall"             => fatal!("not yet implemented: {}", cmd),
        "publish"               => fatal!("not yet implemented: {}", cmd),
        other                   => fatal!("unrecognized subcommand: {}", other),
    }
}

fn help(_args: std::env::ArgsOs) {
    let _ = print_usage(&mut std::io::stdout().lock());
}

fn version(_args: std::env::ArgsOs) {
    println!("{} {}", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"));
}

fn print_usage(o: &mut impl io::Write) -> io::Result<()> {
    writeln!(o, "    Usage:")?;
    writeln!(o, "cargo container [subcommand] ...flags...")?;
    writeln!(o)?;
    writeln!(o, "    Subcommands:")?;
    writeln!(o, "build | b  \"Prepare workspace\" and use `tools` to build the crates")?;
    writeln!(o, "bench      \"Prepare workspace\" and use `tools` to benchmark the crates")?;
    writeln!(o, "check | c  \"Prepare workspace\" and use `tools` to verify the crates compile")?;
    writeln!(o, "clean      Attempt to get rid of generated files")?;
    writeln!(o, "doc        \"Prepare workspace\" and use `tools` to document the crates")?;
    writeln!(o, "fetch      \"Prepare workspace\" and use `tools` to fetch the crates + `cargo fetch`")?;
    writeln!(o, "fuzz       \"Prepare workspace\" and use `tools` to fuzz-test the crates")?;
    writeln!(o, "package    \"Prepare workspace\" and use `tools` to package the crates")?;
    writeln!(o, "run   | r  \"Prepare workspace\" and use `tools` to run the crates")?;
    writeln!(o, "test       \"Prepare workspace\" and use `tools` to test the crates")?;
    writeln!(o, "update     \"Prepare workspace\" and use `tools` to update dependencies")?;
    writeln!(o)?;
    writeln!(o, "    \"Prepare workspace\" generally means:")?;
    writeln!(o, "1. Find a `Container.toml` defining the workspace root")?;
    writeln!(o, "2. Install any `[local-install]` dependencies")?;
    writeln!(o, "3. Enumerate all `tools` found in `[[build]]`s to generate crates")?;
    writeln!(o, "4. Generate a `Cargo.toml` alongside `Container.toml` consuming said crates")?;
    writeln!(o)?;
    writeln!(o, "    Flags:")?;
    writeln!(o, "--arch     <arch | *>      Specify an architecture to target instead of using the default of 'whatever is native'")?;
    writeln!(o, "--config   <config | *>    Specify a configuration to target instead of using the default of 'debug'")?;
    writeln!(o, "--crate    <crate>         Specify a specific crate to build/run/package instead of selecting all available crates")?;
    writeln!(o, "--tool     <tool>          Specify a specific tool to use instead of selecting all available tools")?;
    writeln!(o)?;
    Ok(())
}

#[derive(Default)]
struct Args {
    pub arches:     BTreeSet<String>,
    pub configs:    BTreeSet<String>,
    pub crates:     BTreeSet<String>,
    pub tools:      BTreeSet<String>,
}

fn add_arg(o: &mut BTreeSet<String>, flag: &str, param: &str, args: &mut std::env::ArgsOs) {
    let next = args.next().unwrap_or_else(|| fatal!("expected {} after {}", param, flag)).to_string_lossy().into_owned();
    if let Some(prev) = o.replace(next) {
        warning!("{} {} was already specified", flag, prev);
    }
}

impl Args {
    pub fn from(mut args: std::env::ArgsOs) -> Self {
        let mut o = Self::default();
        while let Some(arg) = args.next() {
            let arg = arg.to_string_lossy();
            match &*arg {
                flag @ "--arch"     => add_arg(&mut o.arches,   flag, "architecture", &mut args),
                flag @ "--config"   => add_arg(&mut o.configs,  flag, "configuration", &mut args),
                flag @ "--crate"    => add_arg(&mut o.crates,   flag, "crate", &mut args),
                flag @ "--tool"     => add_arg(&mut o.tools,    flag, "tool", &mut args),

                flag if flag.starts_with("-") => fatal!("unrecognized flag: {}", flag),
                other => fatal!("unrecognized argument: {}", other),
            }
        }
        if o.configs.is_empty() { o.configs.insert(String::from("debug")); }
        o
    }
}

fn gen_then_fwd(meta: &ContainerToml, args: std::env::ArgsOs, cmd: &str, ok_none: bool, verbing: &str) {
    let args = Args::from(args);
    generate::dot_container(meta);
    generate::workspace_toml(meta);
    local_install(meta);
    generate::crates(meta);

    let path = prepend_paths(Some(Path::new("bin").canonicalize().unwrap().cleanup()));
    let arches = args.arches.iter().map(|s| s.as_str()).collect::<Vec<_>>().join(",");

    for c in args.crates.iter() {
        if !meta.builds.iter().any(|b| b.crates.iter().any(|c2| c.as_str() == c2)) {
            warning!("`--crate {}` is not part of any `[[build]]`'s crates", c);
        }
    }

    for t in args.tools.iter() {
        if !meta.builds.iter().any(|b| b.tools.iter().any(|t2| t.as_str() == t2)) {
            warning!("`--tool {}` is not part of any `[[build]]`'s tools", t);
        }
    }

    let mut builds = ok_none;
    for build in meta.builds.iter() {
        let crates = build.crates.iter().map(|c| c.as_str()).filter(|c| args.crates.is_empty() || args.crates.contains(*c)).collect::<Vec<_>>();
        if crates.is_empty() { continue }
        let crates_s = crates.join(",");
        for tool in build.tools.iter() {
            if !args.tools.is_empty() && !args.tools.contains(tool.as_str()) { continue }
            for config in args.configs.iter() {
                let start = std::time::Instant::now();
                eprintln!();
                status!(verbing, "{} | {} | {} crates", tool, config, crates.len());
                let mut cmd = Command::new(tool.as_str());
                cmd.env("PATH",                         &path);
                cmd.env("CARGO_CONTAINER_COMMAND",      "build");
                cmd.env("CARGO_CONTAINER_CRATES_DIR",   format!(".container/crates/{}", tool));
                cmd.env("CARGO_CONTAINER_ARCHES",       &arches);
                cmd.env("CARGO_CONTAINER_CONFIGS",      config);
                cmd.env("CARGO_CONTAINER_PACKAGES",     &crates_s);
                let status = cmd.status().unwrap_or_else(|err| fatal!("`{}` build failed: {}", tool, err));

                match status.code() {
                    Some(0x000000)  => builds = true, // success
                    Some(0xCCEEEE)  => std::process::exit(1), // errors
                    Some(0xCC3333)  => builds = true, // warnings
                    Some(0xCC0C21)  => {}, // command not implemented
                    Some(0xCC0921)  => {}, // platform not implemented

                    Some(n) if (0xCC0000 ..= 0xCCFFFF).contains(&n) => fatal!("`{}` unrecognized exit code 0x{:x} - is your `cargo container` up-to-date?", tool, n),
                    Some(n)                                         => fatal!("`{}` build failed (exit code {})", tool, n),
                    None                                            => fatal!("`{}` build failed (signal)", tool),
                }
                let stop = std::time::Instant::now();
                status!("Finished", "{} | {} | {} crates in {:.2}s", tool, config, crates.len(), (stop-start).as_secs_f32());
            }
        }
    }
    if !builds { fatal!("`{}`: matched no crate x tool combinations", cmd) }
}

fn clean(meta: &ContainerToml, args: std::env::ArgsOs) {
    // XXX: cleanup bin dir?
    let dot_container = meta.root_directory().join(".container");
    Command::new("cargo").arg("clean").args(args).current_dir(meta.root_directory()).status0().unwrap_or_else(|err| fatal!("`cargo clean` failed: {}", err));
    std::fs::remove_dir_all(&dot_container).unwrap_or_else(|err| fatal!("`cargo container clean` failed to delete `{}`: {}", dot_container.display(), err));
}

fn local_install(meta: &ContainerToml) {
    if meta.local_install.is_empty() { return }
    cargo_local_install::run_from_strs(vec![
        OsStr::new("--no-path-warning"),
        //OsStr::new("--root"), meta.root_directory().join(".container").as_os_str(),
    ].into_iter()).unwrap_or_else(|err| fatal!("cargo-local-install failed: {}", err));
}
