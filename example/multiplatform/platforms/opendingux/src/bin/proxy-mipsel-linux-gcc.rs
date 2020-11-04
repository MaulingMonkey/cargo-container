use platform_common::mmrbi::*;

use std::ffi::OsString;
use std::path::{Component, Path, PathBuf, Prefix};

fn main() {
    if cfg!(windows) {
        let distro = OsString::from("cargo-container-platforms-opendingux-1"); // TODO: read from env var
        let user   = OsString::from("opendingux"); // leave hardcoded?

        let mut cmd = Command::new("wsl");
        cmd.arg("--distribution").arg(distro);
        cmd.arg("--user").arg(user);
        cmd.arg("--").arg("/opt/gcw0-toolchain/usr/bin/mipsel-linux-gcc");

        let mut args = std::env::args_os();
        let _exe = args.next();
        for arg in args {
            if is_abs_win_path(Path::new(&arg)) {
                cmd.arg(wslify_path(Path::new(&arg)));
                continue;
            }

            match arg.into_string() {
                Err(s) => { cmd.arg(s); }, // not valid UTF8, pass through untranslated
                Ok(s) => {
                    if let Some(script) = s.strip_prefix("-Wl,--version-script=") {
                        cmd.arg(format!("-Wl,--version-script={}", wslify_path(Path::new(script)).display()));
                    } else {
                        cmd.arg(s);
                    }
                },
            }
        }

        //cmd.arg("-lpthread");
        cmd.arg("-ldl");        // dlsym, dlopen    (used by stdlib?)
        cmd.arg("-lgcc_eh");    // _Unwind_*        (used by stdlib?)
        //cmd.arg("-lm");
        cmd.status0().or_die();
    } else {
        let mut cmd = Command::new("/opt/gcw0-toolchain/usr/bin/mipsel-linux-g++");
        let mut args = std::env::args_os();
        let _exe = args.next();
        for arg in args { cmd.arg(arg); }
        cmd.status0().or_die();
    }
}

fn is_abs_win_path(path: impl AsRef<Path>) -> bool {
    match path.as_ref().components().next() {
        Some(Component::Prefix(pre)) => match pre.kind() {
            Prefix::VerbatimDisk(_) | Prefix::Disk(_) => true,
            _ => false,
        }
        _ => false,
    }
}

fn wslify_path(path: impl AsRef<Path>) -> PathBuf {
    let path = path.as_ref();
    assert!(path.is_absolute());
    //let path = path.canonicalize().unwrap_or_else(|err| fatal!("unable to WSLify path {}: canonicalization failed: {}", path.display(), err));
    let mut components = path.components();

    let mut o = OsString::from("/mnt/");
    let pre = components.next().unwrap_or_else(|| fatal!("unable to WSLify path {}: no components", path.display()));
    let pre = if let Component::Prefix(pre) = pre { pre } else { fatal!("unable to WSLify path {}: missing expected initial Component::Prefix", path.display()) };
    match pre.kind() {
        Prefix::Verbatim(_)         => fatal!("unable to WSLify path {}: verbatim prefix", path.display()),
        Prefix::VerbatimUNC(_, _)   => fatal!("unable to WSLify path {}: verbatim UNC prefix", path.display()),
        Prefix::DeviceNS(_)         => fatal!("unable to WSLify path {}: device namespace prefix", path.display()),
        Prefix::UNC(_, _)           => fatal!("unable to WSLify path {}: UNC prefix", path.display()),
        Prefix::VerbatimDisk(d) | Prefix::Disk(d) => {
            o.push(format!("{}", (d as char).to_ascii_lowercase()));
        },
    }
    if let Some(Component::RootDir) = components.next() {} else { fatal!("unable to WSLify path {}: expected Component::RootDir after prefix", path.display()) }

    for c in components {
        o.push("/");
        o.push(c);
    }

    PathBuf::from(o)
}

#[test] fn test_wslify_path() {
    assert_eq!(wslify_path(r"C:\Windows\System32"), Path::new("/mnt/c/Windows/System32"));
}
