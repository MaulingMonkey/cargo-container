use crate::*;

use mmrbi::*;

use std::collections::BTreeSet;
use std::fmt::Write as _;
use std::ffi::*;
use std::io::{self, BufRead, BufReader};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};



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
        "bench"                 => gen_then_fwd(&meta, args, "bench",   false, "Benchmarking"),
        "build" | "b"           => gen_then_fwd(&meta, args, "build",   false, "Building"),
        "check" | "c"           => check(&meta, args),
        "clean"                 => clean(&meta, args),
        "doc"                   => gen_then_fwd(&meta, args, "doc",     false, "Documenting"),
        "fetch"                 => fetch(&meta, args),
        "fuzz"                  => gen_then_fwd(&meta, args, "fuzz",    false, "Fuzzing"),
        "package"               => gen_then_fwd(&meta, args, "package", false, "Packaging"),
        "run" | "r"             => gen_then_fwd(&meta, args, "run",     false, "Running"), // XXX: Is this what we actually want?
        "setup"                 => setup(&meta, args),
        "test" | "t"            => gen_then_fwd(&meta, args, "test",    false, "Testing"),
        "update"                => gen_then_fwd(&meta, args, "update",  false, "Updating"),

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

fn check(meta: &ContainerToml, args: std::env::ArgsOs) {
    gen_then_fwd(&meta, args, "check", true, "Checking");
    Command::new("cargo").arg("check").status0().or_die();
}

fn clean(meta: &ContainerToml, args: std::env::ArgsOs) {
    // XXX: cleanup bin dir?
    let dot_container = meta.root_directory().join(".container");
    Command::new("cargo").arg("clean").args(args).current_dir(meta.root_directory()).status0().unwrap_or_else(|err| fatal!("`cargo clean` failed: {}", err));
    std::fs::remove_dir_all(&dot_container).unwrap_or_else(|err| fatal!("`cargo container clean` failed to delete `{}`: {}", dot_container.display(), err));
}

fn fetch(meta: &ContainerToml, args: std::env::ArgsOs) {
    gen_then_fwd(&meta, args, "fetch", true, "Fetching");
    Command::new("cargo").arg("fetch").status0().or_die();
}

fn setup(meta: &ContainerToml, args: std::env::ArgsOs) {
    gen_then_fwd(&meta, args, "setup", true, "Setup");
}

fn gen_then_fwd(meta: &ContainerToml, args: std::env::ArgsOs, command: &str, ok_none: bool, verbing: &str) {
    let args = Args::from(args);
    std::fs::remove_dir_all(".container/scripts/setup").unwrap_or_else(|err| if err.kind() != io::ErrorKind::NotFound { fatal!("unable to remove .container/scripts/setup: {}", err) });
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

    let mut apt_packages = BTreeSet::new();
    let mut sudos = Vec::new();

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
                cmd.env("CARGO_CONTAINER_COMMAND",      command);
                cmd.env("CARGO_CONTAINER_CRATES_DIR",   format!(".container/crates/{}", tool));
                cmd.env("CARGO_CONTAINER_ARCHES",       &arches);
                cmd.env("CARGO_CONTAINER_CONFIGS",      config);
                cmd.env("CARGO_CONTAINER_PACKAGES",     &crates_s);

                cmd.stdin(Stdio::null());
                cmd.stdout(Stdio::piped());
                cmd.stderr(Stdio::inherit());

                let prev_sudo_len = sudos.len();

                let mut child = cmd.spawn().unwrap_or_else(|err| fatal!("`{}` {} failed: {}", tool, command, err));
                let mut stdout = BufReader::new(child.stdout.take().unwrap());
                let mut line = String::new();
                loop {
                    line.clear();
                    match stdout.read_line(&mut line) {
                        Ok(0) => break,                                                 // EOF
                        Err(err) if err.kind() == io::ErrorKind::BrokenPipe => break,   // EPIPE
                        Err(err) => fatal!("error reading stdout from `{}` {}: {}", tool, command, err),
                        Ok(_) => {},
                    }
                    let line = line.trim_end_matches("\n").trim_end_matches("\r");
                    if let Some(cc) = unpre(line, "cargo-container:") {
                        if let Some(sudo) = unpre(cc, "sudo=") {
                            if prev_sudo_len == sudos.len() {
                                sudos.push(format!("{} requested by {} {}", if cfg!(windows) { "::" } else { "#" }, tool, command));
                            }
                            sudos.push(sudo.into());
                        } else if let Some(pkg) = unpre(cc, "apt-get-install=") {
                            apt_packages.insert(String::from(pkg));
                        } else if let Some(msg) = unpre(cc, "error=") {
                            error!(code: tool, "{}", msg);
                        } else if let Some(msg) = unpre(cc, "warning=") {
                            warning!(code: tool, "{}", msg);
                        } else if let Some(msg) = unpre(cc, "info=") {
                            info!(code: tool, "{}", msg);
                        } else {
                            warning!("unrecognized directive: {:?}", line);
                        }
                    } else {
                        println!("{}", line); // ...ignore?
                    }
                }

                if prev_sudo_len != sudos.len() {
                    sudos.push(String::new());
                }

                let status = child.wait().unwrap_or_else(|err| fatal!("`{}` {} failed: {}", tool, command, err));
                match status.code() {
                    Some(0x00) => builds = true, // success
                    Some(0xEE) => std::process::exit(1), // errors
                    Some(0x33) => builds = true, // warnings
                    Some(0xC1) => {}, // command not implemented
                    Some(0x91) => {}, // platform not implemented

                    Some(n) => fatal!("`{}` {} failed (exit code {})", tool, command, n),
                    None    => fatal!("`{}` {} failed (signal)", tool, command),
                }
                let stop = std::time::Instant::now();
                status!("Finished", "{} | {} | {} crates in {:.2}s", tool, config, crates.len(), (stop-start).as_secs_f32());
            }
        }
    }
    if !builds { fatal!("`{}`: matched no crate x tool combinations", command) }

    if !apt_packages.is_empty() {
        let mut install = String::from("apt-get install -y");
        for pkg in apt_packages.iter() {
            install.push_str(" ");
            install.push_str(pkg.as_str());
        }
        sudos.push(comment("requested by cargo-container for apt-get-install directives"));
        sudos.push(install);
        sudos.push(String::new());
    }

    if !sudos.is_empty() {
        if sudos.iter().any(|line| line.starts_with("apt-get ")) {
            sudos.insert(0, String::new());
            sudos.insert(0, "apt-get update".into());
            sudos.insert(0, comment("requested by cargo-container for apt-get commands"));
        }

        let allow_sudo = args.allow_sudo.unwrap_or_else(|| {
            // return false if command != "setup" with warning?
            if std::env::var_os("CI").is_some() {
                return true;
            }

            info!("tools wish to run the following commands as {}", if cfg!(windows) { "admin" } else { "root" });
            eprintln!();
            for line in sudos.iter() {
                if is_comment(line) {
                    //eprintln!("    \u{001B}[36;1m{}\u{001B}[0m", line); // green
                    eprintln!("    \u{001B}[90m{}\u{001B}[0m", line); // grey
                } else {
                    eprintln!("    {}", line);
                }
            }
            eprint!("would you like to run these commands? (N/y) ");

            let mut answer = String::new();
            match std::io::stdin().read_line(&mut answer) {
                Ok(_) => {},
                Err(err) if err.kind() == io::ErrorKind::BrokenPipe => {
                    eprintln!("N");
                    warning!("unable to read answer from stdin, assuming no.  use --allow-sudo to script, or --deny-sudo to supress this message");
                    return false;
                },
                Err(err) => {
                    fatal!("unable to read answer from stdin: {}", err);
                },
            }
            answer.make_ascii_lowercase();
            match answer.trim() {
                "y" | "ye" | "yes"  => true,
                _other              => false,
            }
        });

        if !allow_sudo {
            status!("Skipping", "admin tasks");
        } else {
            std::fs::create_dir_all(".container/scripts").unwrap_or_else(|err| fatal!("unable to create directory .container/scripts: {}", err));
            if cfg!(windows) {
                let mut script = String::new();
                // setlocal? pushd? ...?
                for line in sudos.iter() {
                    if is_comment(line) { continue }
                    writeln!(&mut script, "@echo on").unwrap();
                    writeln!(&mut script, "{}", line).unwrap();
                    writeln!(&mut script, "@if ERRORLEVEL 1 exit /b %ERRORLEVEL%").unwrap();
                }
                writeln!(&mut script, "pause").unwrap();
                let script_path = PathBuf::from(format!(".container/scripts/sudo-{}.cmd", std::process::id()));
                std::fs::write(&script_path, script).unwrap_or_else(|err| fatal!("unable to write {}: {}", script_path.display(), err));

                let mut params = OsString::from("/C \"call \"");
                params.push(script_path.as_os_str());
                params.push("\"\"\0");

                #[cfg(windows)] {
                    use winapi::um::errhandlingapi::GetLastError;
                    use winapi::um::handleapi::CloseHandle;
                    use winapi::um::shellapi::{ShellExecuteExW, SHELLEXECUTEINFOW, SEE_MASK_NOCLOSEPROCESS};
                    use winapi::um::synchapi::WaitForSingleObject;
                    use winapi::um::winbase::{INFINITE, WAIT_OBJECT_0};
                    use winapi::um::winuser::SW_HIDE;

                    use std::convert::TryInto;
                    use std::os::windows::ffi::OsStrExt;
                    use std::ptr::null_mut;

                    let params = params.encode_wide().collect::<Vec<_>>();

                    // CoInitializeEx(NULL, COINIT_APARTMENTTHREADED | COINIT_DISABLE_OLE1DDE)

                    // Don't use GetConsoleWindow here.  While it "works", when combined with VS Code,
                    // this results in a UAC prompt hiding in the background.  By using null instead,
                    // the UAC prompt comes nicely to the front where we can actually accept it.
                    let hwnd = std::ptr::null_mut();

                    let mut sei = SHELLEXECUTEINFOW {
                        cbSize:         std::mem::size_of::<SHELLEXECUTEINFOW>().try_into().unwrap(),
                        fMask:          SEE_MASK_NOCLOSEPROCESS,
                        hwnd,
                        lpVerb:         wchar::wch_c!("runas").as_ptr(), // "Launches an application as Administrator. User Account Control (UAC) will prompt the user for consent to run the application ..."
                        lpFile:         wchar::wch_c!("cmd.exe").as_ptr(),
                        lpParameters:   params.as_ptr(),
                        lpDirectory:    null_mut(), // "... If this value is NULL, the current working directory is used."
                        nShow:          SW_HIDE,
                        hInstApp:       null_mut(),
                        lpIDList:       null_mut(),
                        lpClass:        null_mut(),
                        hkeyClass:      null_mut(),
                        dwHotKey:       0,
                        hMonitor:       null_mut(),
                        hProcess:       null_mut(),
                    };

                    // https://docs.microsoft.com/en-us/windows/win32/api/shellapi/nf-shellapi-shellexecuteexw
                    let success = unsafe { ShellExecuteExW(&mut sei) };
                    if success == 0 {
                        let gle = unsafe { GetLastError() };
                        fatal!("`cmd /C \"call \"{}\"\"` failed: ShellExecuteExW failed with GetLastError() == 0x{:08x}", script_path.display(), gle);
                    }

                    status!("Running", "admin tasks");
                    // Sadly, we don't have any stdout.  We could setup some kind of pipe maybe...?
                    let wait = unsafe { WaitForSingleObject(sei.hProcess, INFINITE) };
                    assert_eq!(wait, WAIT_OBJECT_0);
                    unsafe { CloseHandle(sei.hProcess) };
                    status!("Finished", "admin tasks");
                }

                let _ = params;
            } else {
                let mut script = String::new();
                writeln!(&mut script, "set -e").unwrap();
                for line in sudos.iter() {
                    if is_comment(line) { continue }
                    let quot = format!("{:?}", line);
                    writeln!(&mut script, "echo \"   \u{001B}[32;1mExecuting\u{001B}[0m {}\"", &quot[1..quot.len()-1]).unwrap();
                    writeln!(&mut script, "{}", line).unwrap();
                }
                let script_path = PathBuf::from(format!(".container/scripts/sudo-{}.sh", std::process::id()));
                std::fs::write(&script_path, script).unwrap_or_else(|err| fatal!("unable to write {}: {}", script_path.display(), err));

                let mut sh = Command::new("sudo");
                sh.arg("sh");
                sh.arg(&script_path);
                let status = sh.status0();
                let _ = std::fs::remove_file(&script_path);
                status.unwrap_or_else(|err| fatal!("`sudo sh {}` failed: {}", script_path.display(), err));
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

fn unpre<'s>(s: &'s str, pre: &str) -> Option<&'s str> {
    if s.starts_with(pre) {
        Some(&s[pre.len()..])
    } else {
        None
    }
}

fn is_comment(line: &str) -> bool {
    let line = line.trim_start();
    line.is_empty() || line.starts_with(if cfg!(windows) { "::" } else { "#" })
}

fn comment(c: &str) -> String {
    format!("{} {}", if cfg!(windows) { "::" } else { "#" }, c)
}
