#![cfg_attr(unix, allow(dead_code))]
#![cfg_attr(unix, allow(unused_imports))]

#[path="windows/_windows.rs"] mod windows;

use platform_common::*;

use mmrbi::*;
use mmrbi::fs::write_if_modified_with as wimw;

use std::io::Write;
use std::path::PathBuf;



const XARGO_VERSION : &'static str = "0.3.22";

const DISTRO_ID : &'static str = "cargo-container-platforms-opendingux-1";
const PFNS : &'static [&'static str] = &[
    "CanonicalGroupLimited.Ubuntu16.04onWindows_79rhkp1fndgsc", // Ubuntu 16.04 via https://aka.ms/wsl-ubuntu-1604
    "CanonicalGroupLimited.Ubuntu18.04onWindows_79rhkp1fndgsc", // Ubuntu 18.04 via https://aka.ms/wsl-ubuntu-1804
    "CanonicalGroupLimited.Ubuntu20.04onWindows_79rhkp1fndgsc", // Ubuntu 20.04 via ???
    "CanonicalGroupLimited.UbuntuonWindows_79rhkp1fndgsc",      // Ubuntu (20.04)
];

const GCW0_TOOLCHAIN_ENTRIES : usize = 30591 + 2160; // files + dirs for tar progress
const GCW0_TOOLCHAIN : Download = Download {
    name:   "opendingux gcw0 toolchain (2014-08-20)",
    url:    "http://www.gcw-zero.com/files/opendingux-gcw0-toolchain.2014-08-20.tar.bz2",
    //url:    concat!("file:///C:/Users/", env!("USERNAME"), "/Downloads/opendingux-gcw0-toolchain.2014-08-20.tar.bz2"),
    sha256: "3632C85F48879108D4349570DED60D87D7A324850B81D683D031E4EE112BAAA0",
};

const UBUNTU_PFN : &'static str = "CanonicalGroupLimited.Ubuntu16.04onWindows_79rhkp1fndgsc";
const UBUNTU_APPX : Download = Download {
    name:   "ubuntu 16.04",
    url:    "https://aka.ms/wsl-ubuntu-1604",
    //url:    concat!("file:///C:/Users/", env!("USERNAME"), "/Downloads/Ubuntu_1604.2019.523.0_x64.appx"),
    sha256: "55782021E04F52D071814FDAD02CD37448C7A49F2EF5A66899F3D5BC7D79859F",
};

fn distros() -> impl Iterator<Item = &'static str> {
    if env::has_var("CI") {&[
        DISTRO_ID,
        // For CI, consider just using whatever VMs are already installed, if any.
        "Ubuntu",
        "Ubuntu-16.04", // Appveyor
        "Ubuntu-18.04", // Appveyor
        "Ubuntu-20.04", // Appveyor
        "openSUSE-42",  // Appveyor
    ][..]} else {&[
        DISTRO_ID,
    ][..]}.iter().copied()
}

fn supported(warn: bool) -> bool {
    let mut supported = true;

    // rustc version
    let rustc = rustc::version().or_die();
    let nightly = rustc.version.pre.iter().map(|s| s.to_string()).collect::<Vec<_>>() == ["nightly"];
    if !nightly {
        if warn { warning!("requires nightly rustc, on rustc {}", rustc.version); }
        supported = false;
    }

    if cfg!(windows) {
        match windows::version() {
            None => {
                warning!("unable to determine windows version, may fail");
            },
            Some(ver) => {
                // https://docs.microsoft.com/en-us/windows/wsl/release-notes
                //  Build 18362:  Introduces WSL 2 (required - WSL1 doesn't like gcw0's 32-bit linux binaries)
                //  Build 18305:  Introduces `wsl --import` (could workaround with WslLaunch, but it's super painful)
                //  Build 17763:  Appveyor's "Visual Studio 2019" image as of 11/2/2020
                //  Build 17763:  Github Action's "windows-2019" image as of 11/2/2020
                let req = "windows 10 build 18362";
                if ver < (10, 0, 0, 0) {
                    if !warn {
                        // squelched
                    } else if ver.1 == 0 {
                        warning!("requires {}, on windows {}", req, ver.0);
                    } else {
                        warning!("requires {}, on windows {}.{}", req, ver.0, ver.1);
                    }
                    supported = false;
                } else if ver < (10, 0, 18362, 0) {
                    if warn { warning!("requires {}, on windows 10 build {}", req, ver.2); }
                    supported = false;
                }
            },
        }
    } else if cfg!(target_os = "linux") {
        if warn { warning!("linux host not yet implemented"); }
        supported = false;
    } else {
        if warn { warning!("host platform not supported"); }
        supported = false;
    }

    supported
}



fn main() { platform_common::exec(Tool, "opendingux") }

struct Tool;
impl platform_common::Tool for Tool {
    fn setup(&self, _state: &State) {
        if !supported(true) { return }

        windows::features::require("Microsoft-Windows-Subsystem-Linux");    // WSL 1
        windows::features::require("VirtualMachinePlatform");               // WSL 2 - required to run gcw0's 32-bit elf binaries

        if cfg!(target_os = "windows") {
            #[cfg(windows)] {
                let wsl = wslapi::Library::new().unwrap_or_else(|err| fatal!(
                    "unable to check/install a distro for opendingux builds: WSL not available ({})!  You may need to install WSL, or restart if you recently did.",
                    err
                ));
                let distro = windows::wsl::ensure_distro_installed(&wsl);
                windows::wsl::ensure_root_stuff(&wsl, distro);
                windows::wsl::ensure_gcw0_installed(&wsl, distro);
            }
            install_xargo();
        } else if cfg!(target_os = "linux") {
            install_xargo();
            // TODO: toolchain junk
        } else {
            // OS X?
            // ...?
        }
    }

    fn generate(&self, state: &State) {
        if !supported(false) { return }

        for package in state.packages.iter() {
            let out_dir = package.generated_path();
            std::fs::create_dir_all(&out_dir).unwrap_or_else(|err| fatal!("unable to create `{}`: {}", out_dir.display(), err));

            // XXX: Allow merging Xargo.toml files from multiple sources
            wimw("Xargo.toml", |o|{
                writeln!(o, "# AUTOGENERATED BY {}", env!("CARGO_PKG_NAME"))?;
                writeln!(o, "[target.mipsel-gcw0-linux-uclibc.dependencies.std]")?;
                writeln!(o, "features = []")?;
                Ok(())
            }).or_die();

            // See https://github.com/MaulingMonkey/rust-opendingux-test/blob/master/mipsel-gcw0-linux-uclibc.json for some interesting notes
            wimw("mipsel-gcw0-linux-uclibc.json", |o| write!(o, "{}", include_str!("mipsel-gcw0-linux-uclibc.json"))).or_die();

            wimw(out_dir.join("main.rs"), |o|{
                writeln!(o, "// AUTOGENERATED BY {}", env!("CARGO_PKG_NAME"))?;
                writeln!(o)?;
                writeln!(o, "fn main() {{ app::init(app_common::ConsoleDialogProvider) }}")?;
                Ok(())
            }).or_die();

            wimw(out_dir.join("Cargo.toml"), |o|{
                writeln!(o, "# AUTOGENERATED BY {}", env!("CARGO_PKG_NAME"))?;
                writeln!(o)?;
                writeln!(o, "[package]")?;
                writeln!(o, "name            = {:?}", package.generated_name())?;
                writeln!(o, "version         = {:?}", package.version())?;
                writeln!(o, "description     = {:?}", package.description())?;
                writeln!(o, "publish         = false")?;
                writeln!(o, "edition         = {:?}", "2018")?;
                writeln!(o)?;
                writeln!(o, "[dependencies]")?;
                writeln!(o, "app-common      = {{ path = {:?}, features = [{:?}] }}", "../../../../app-common", "platform-console")?;
                writeln!(o, "app             = {{ path = {:?}, package = {:?} }}",    package.original_path(), package.original_name())?;
                writeln!(o)?;
                writeln!(o, "[[bin]]")?;
                writeln!(o, "name            = {:?}", package.original_name())?;
                writeln!(o, "path            = {:?}", "main.rs")?;
                Ok(())
            }).or_die();
        }
    }

    fn build(&self, state: &State) {
        if !supported(true) { return }

        for config in state.configs.iter() {
            let mut cmd = Command::new("xargo");
            cmd.args(&["build", "--target=mipsel-gcw0-linux-uclibc"]);
            match config.name() {
                "debug"     => {},
                "release"   => { cmd.arg("--release"); },
                other       => fatal!("unexpected config: {:?}", other),
            }
            for package in state.packages.iter() { cmd.arg("-p"); cmd.arg(&package.generated_name()); }
            cmd.status0().or_die();
        }
    }

    fn package(&self, state: &State) {
        if !supported(true) { return }

        for package in state.packages.iter() {
            let pkg_dir = PathBuf::from(format!("target/opendingux/packages/{}", package.original_name()));
            let pkg_opk = PathBuf::from(format!("target/opendingux/packages/{}.opk", package.original_name()));
            let _ = std::fs::remove_dir_all(&pkg_dir);
            let _ = std::fs::remove_file(&pkg_opk);
            std::fs::create_dir_all(&pkg_dir).unwrap_or_else(|err| fatal!("unable to create {}: {}", pkg_dir.display(), err));
            for config in state.configs.iter() {
                let src_bin = PathBuf::from(format!("target/mipsel-gcw0-linux-uclibc/{}/{}", config.name(), package.original_name()));
                let dst_bin = PathBuf::from(format!("target/opendingux/packages/{}/app.{}", package.original_name(), config.name()));
                std::fs::copy(&src_bin, &dst_bin).unwrap_or_else(|err| fatal!("unable to copy {} to {}: {}", src_bin.display(), dst_bin.display(), err));
                wimw(format!("target/opendingux/packages/{}/{}.all.desktop", package.original_name(), config.name()), |o|{
                    let name = package.original_name();
                    // cargo-container doesn't currently pass this information to package commands
                    //let desc = package.description();
                    //let desc = if desc.is_empty() { name } else { desc.as_str() };
                    let desc = name;
                    writeln!(o, "[Desktop Entry]")?;
                    writeln!(o, "Type=Application")?;
                    writeln!(o, "Version=1.0")?;
                    if state.configs.len() <= 1 {
                        writeln!(o, "Name={name}", name=name)?;
                    } else {
                        writeln!(o, "Name={name} ({config})", name=name, config=config.name())?;
                    }
                    writeln!(o, "Comment={desc}", desc=desc)?;
                    writeln!(o, "Categories=rust;")?;
                    writeln!(o, "Icon=icon")?;
                    writeln!(o, "Terminal=true")?; // XXX
                    writeln!(o, "Exec=app.{config}", config=config.name())?;
                    Ok(())
                }).or_die();
            }
            wimw(format!("target/opendingux/packages/{}/icon.png", package.original_name()), |o| o.write_all(include_bytes!("placeholder-icon.png"))).or_die();
            // TODO: filter to heck and back?
            Command::new("wsl")
                .arg("-u").arg("opendingux")
                .arg("-d").arg("cargo-container-platforms-opendingux-1")
                .arg("--").arg("mksquashfs")
                .arg(&pkg_dir)
                .arg(&pkg_opk)
                .arg("-comp").arg("gzip")
                .arg("-noappend")
                .status0().or_die();
        }
    }

    fn deploy(&self, state: &State) {
        if !supported(true) { return }

        let dst_ip  = "10.1.1.2";
        let dst_user= "root";
        for package in state.packages.iter() {
            let src_opk = format!("target/opendingux/packages/{}.opk", package.original_name());
            let dst_opk = format!("/media/sdcard/apps/{}.opk", package.original_name());
            Command::new("scp")
                .arg("-o").arg("StrictHostKeyChecking=accept-new") // XXX
                // Normally this is a bit of a bad idea.  However:
                //  1.  We provide no secrets such as a password, and *only* write .opk s to this target.
                //  2.  Despite the IP addresses, this is supposed to be a USB-local "Remote NDIS" device.
                //      https://docs.microsoft.com/en-us/windows-hardware/drivers/network/overview-of-remote-ndis--rndis-
                //  3.  You're not actually checking the new fingerprint anyways.
                .arg(&src_opk)
                .arg(format!("{user}@{ip}:{path}", user=dst_user, ip=dst_ip, path=dst_opk))
                .status0().or_die();
        }
    }
}

fn install_xargo() {
    cargo_local_install::run_from_strs(vec![
        "--no-path-warning",
        "xargo",
        "--locked",
        "--version",
        &format!("^{}", XARGO_VERSION),
    ].into_iter()).or_die();
}
