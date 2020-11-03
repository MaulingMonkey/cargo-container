#![cfg_attr(unix, allow(dead_code))]
#![cfg_attr(unix, allow(unused_imports))]

#[path="windows/_windows.rs"] mod windows;

use platform_common::*;

use mmrbi::*;
use mmrbi::fs::write_if_modified_with as wimw;

use std::ffi::OsString;
use std::io::Write;
use std::path::{Component, Path, PathBuf, Prefix};



const DISTRO_ID : &'static str = "cargo-container-platforms-opendingux-1";
const PFNS : &'static [&'static str] = &[
    "CanonicalGroupLimited.UbuntuonWindows_79rhkp1fndgsc",      // Ubuntu (16.04)
    "CanonicalGroupLimited.Ubuntu16.04onWindows_79rhkp1fndgsc", // Ubuntu 16.04 via https://aka.ms/wsl-ubuntu-1604
    "CanonicalGroupLimited.Ubuntu18.04onWindows_79rhkp1fndgsc", // Ubuntu 18.04 via https://aka.ms/wsl-ubuntu-1804
    "CanonicalGroupLimited.Ubuntu20.04onWindows_79rhkp1fndgsc", // Ubuntu 20.04 via ???
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



fn main() { platform_common::exec(Tool, "opendingux") }

struct Tool;
impl platform_common::Tool for Tool {
    fn setup(&self, _state: &State) {
        windows::features::require("Microsoft-Windows-Subsystem-Linux");    // WSL 1 (required)
        //windows::features::require("VirtualMachinePlatform");             // WSL 2 (optional)

        if cfg!(target_os = "windows") {
            #[cfg(windows)] {
                let wsl = wslapi::Library::new().unwrap_or_else(|err| fatal!(
                    "unable to check/install a distro for opendingux builds: WSL not available ({})!  You may need to install WSL, or restart if you recently did.",
                    err
                ));
                let distro = wsl_ensure_distro_installed(&wsl);
                wsl_ensure_root_stuff(&wsl, distro);
                wsl_ensure_gcw0_installed(&wsl, distro);
            }
        } else if cfg!(target_os = "linux") {
            // ...
        } else {
            // ...?
        }
    }

    fn generate(&self, state: &State) {
        for package in state.packages.iter() {
            let out_dir = package.generated_path();
            std::fs::create_dir_all(&out_dir).unwrap_or_else(|err| fatal!("unable to create `{}`: {}", out_dir.display(), err));

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

    fn build(&self, _state: &State) {
        return;
    }

    fn test(&self, _state: &State) {
        return;
    }
}

#[cfg(windows)]
fn wsl_ensure_distro_installed(wsl: &wslapi::Library) -> &'static str {
    if let Some(distro) = distros().find(|d| wsl.is_distribution_registered(d)) { return distro; }

    // Unnecessary - or perhaps auto-invoked by winrt?
    //use winapi::shared::winerror::SUCCEEDED;
    //use winapi::winrt::roapi::*;
    //let hr = unsafe { RoInitialize(RO_INIT_MULTITHREADED) };
    //if !SUCCEEDED(hr) { fatal!("RoInitialize(RO_INIT_MULTITHREADED) failed with HRESULT 0x{:08x}", hr); }

    let pm = windows::management::deployment::PackageManager::new().unwrap_or_else(|err| fatal!("unable to create PackageManager to locate WSL images: {:?}", err));
    for pfn in PFNS.iter().copied() {
        if wsl_install_distro_from_pfn(wsl, &pm, pfn) { return DISTRO_ID; }
    }

    let tmp_appx_path = std::env::temp_dir().join(format!("ccod-ubuntu.appx"));
    std::fs::write(&tmp_appx_path, UBUNTU_APPX.download()).unwrap_or_else(|err| fatal!("unable to write to {}: {}", tmp_appx_path.display(), err));

    //#[cfg(nope)] { // XXX: figure out how the heck to create an IIterable for add_package_async
    //    let tmp_appx_uri = urlify_path(&tmp_appx_path);
    //    let tmp_appx_uri = Uri::create_uri(tmp_appx_uri.to_string_lossy().into()).unwrap_or_else(|err| fatal!("unable to create URI from {}: {:?}", tmp_appx_uri.to_string_lossy(), err));
    //    let op = pm.add_package_async(tmp_appx_uri, Vec::new(), DeploymentOptions::None).unwrap_or_else(|err| fatal!("unable to packages->AddPackageAsync(...): {:?}", err));
    //    op.set_completed(AsyncOperationWithProgressCompletedHandler::new(|_aowp, _status| {
    //        panic!(); // XXX
    //    })).unwrap_or_else(|err| fatal!("unable to add_appx_op->SetCompleted(...): {:?}", err));
    //}

    status!("Installing", "{} ({})", UBUNTU_APPX.name, UBUNTU_APPX.url);
    Command::new("powershell").args(&["Add-AppxPackage", "-Path"]).arg(&tmp_appx_path).status0().or_die();
    if wsl_install_distro_from_pfn(wsl, &pm, UBUNTU_PFN) { return DISTRO_ID; }

    error!("unable to install {} from {}", DISTRO_ID, UBUNTU_PFN);
    eprintln!("Consider installing from one of the following sources:");
    for pfn in PFNS.iter().copied() {
        eprintln!("    ms-windows-store://pdp/?PFN={}", pfn);
    }
    eprintln!("And then re-run `cargo container setup`");
    open_url(&format!("ms-windows-store://pdp/?PFN={}", UBUNTU_PFN));
    std::process::exit(1);
}

#[cfg(windows)]
#[allow(deprecated)] // std::env::home_dir
fn wsl_install_distro_from_pfn(wsl: &wslapi::Library, pm: &windows::management::deployment::PackageManager, pfn: &str) -> bool {
    use std::os::windows::prelude::*;

    let user_sid = ""; // empty string = current user
    let packages = pm.find_packages_by_user_security_id_package_family_name(user_sid, pfn).unwrap_or_else(|err| fatal!("unable to find packages to locate WSL images: {:?}", err));
    for package in packages {
        let loc = package.installed_location().unwrap_or_else(|err| fatal!("unable to get Package({:?})->InstalledLocation: {:?}", pfn, err));
        let path = loc.path().unwrap_or_else(|err| fatal!("unable to get Package({:?})->InstalledLocation->Path: {:?}", pfn, err));
        let path = PathBuf::from(OsString::from_wide(path.as_wide()));
        let install_tar_gz = path.join("install.tar.gz");
        if install_tar_gz.exists() {
            status!("Installing", "{} from {} (this may take a minute)", DISTRO_ID, install_tar_gz.display());
            let home = std::env::home_dir().unwrap_or_else(|| fatal!("unable to determine home directory"));
            let distro_dir = home.join(format!(r".cargo\container\{}", DISTRO_ID));
            // --import requires build 18305 https://docs.microsoft.com/en-us/windows/wsl/release-notes#build-18305
            Command::new("wsl").arg("--import").arg(DISTRO_ID).arg(&distro_dir).arg(&install_tar_gz).status0().unwrap_or_else(|err| fatal!(
                "unable to import distro {} from {} into {}: {}", DISTRO_ID, install_tar_gz.display(), distro_dir.display(), err
            ));
            let _ = wsl.register_distribution(DISTRO_ID, ""); // wslapi caches distro information which wsl --import bypassed, try and refresh
            if !wsl.is_distribution_registered(DISTRO_ID) {
                warning!("wslapi doesn't recognize {} as registered despite supposedly importing successfully", DISTRO_ID);
            }
            return true;
        } else {
            warning!("missing {} (expected to exist thanks to {} but it doesn't)", install_tar_gz.display(), pfn);
        }
    }
    false
}

#[cfg(windows)]
fn open_url(url: &str) {
    use winapi::um::errhandlingapi::GetLastError;
    use winapi::um::shellapi::ShellExecuteW;
    use winapi::um::winuser::SW_SHOWDEFAULT;

    use std::os::windows::ffi::OsStrExt;
    use std::ptr::null_mut;

    // https://docs.microsoft.com/en-us/windows/win32/api/shellapi/nf-shellapi-shellexecutew
    let success = unsafe { ShellExecuteW(
        null_mut(),                     // hwnd
        wchar::wch_c!("open").as_ptr(), // operation
        OsString::from(url).encode_wide().chain(Some(0)).collect::<Vec<_>>().as_ptr(), // "file" (url)
        null_mut(),                     // parameters
        null_mut(),                     // directory
        SW_SHOWDEFAULT,                 // show
    )} as usize > 32; // "If the function succeeds, it returns a value greater than 32."
    if !success {
        let gle = unsafe { GetLastError() };
        fatal!("ShellExecuteW(0, \"open\", store_url, ...) failed with GetLastError() == 0x{:08x}", gle);
    }
}

#[cfg(windows)] fn wsl_root(distro: &str) -> Command {
    let mut c = Command::new("wsl");
    c.arg("--distribution").arg(distro).arg("--user").arg("root").arg("--");
    c
}

#[cfg(windows)] fn wsl_od(distro: &str) -> Command {
    let mut c = Command::new("wsl");
    c.arg("--distribution").arg(distro).arg("--user").arg("opendingux").arg("--");
    c
}

#[cfg(windows)]
fn wsl_ensure_root_stuff(wsl: &wslapi::Library, distro: &str) {
    // Create "opendingux" user
    wsl_root(distro).args(&["adduser", "--disabled-password", "--gecos", "User for building opendingux packages", "--system", "--shell", "/bin/bash", "opendingux"]).io0(|line|{
        if line.trim_end().ends_with(" already exists. Exiting.") { return } // ignore spam if user already exists
        println!("{}", line);
    }, |line| {
        if line.trim_end().ends_with(" already exists. Exiting.") { return } // ignore spam if user already exists
        eprintln!("{}", line);
    }).or_die();

    // Use "opendingux" user by default
    use std::sync::{Arc, atomic::{AtomicU32, Ordering::SeqCst}};
    let uid = Arc::new(AtomicU32::new(0));
    let uid2 = uid.clone();
    wsl_od(distro).args(&["id", "-u"]).io0(move |id| uid2.store(id.parse().unwrap_or(0), SeqCst), |l| error!("failed to get user id: {}", l)).or_die();
    if let Ok(cfg) = wsl.get_distribution_configuration(distro) {
        wsl.configure_distribution(distro, uid.load(SeqCst), cfg.flags).or_die();
    } else {
        warning!("unable to set default user to \"opendingux\"");
    }

    // Update/install apt packages
    if cfg!(nope) {
        status!("Updating", "apt sources and packages");
        wsl_root(distro).args(&["apt-get", "-y", "update"]).status0().or_die();
        //wsl_root(distro).args(&["apt-get", "-y", "install", "..."]).status0().or_die();
    }
}

#[cfg(windows)]
fn wsl_ensure_gcw0_installed(_wsl: &wslapi::Library, distro: &str) {
    use std::sync::{Arc, atomic::{AtomicBool, AtomicUsize, Ordering::{SeqCst, Relaxed}}};

    let hash_ok = Arc::new(AtomicBool::new(false));
    let hash_ok_io = hash_ok.clone();
    wsl_root(distro).args(&["cat", "/opt/gcw0-toolchain/sha256"]).io(move |sha256| {
        let sha256 = sha256.trim();
        hash_ok_io.store(sha256 == GCW0_TOOLCHAIN.sha256, SeqCst);
    }, |_| {
        // ignore errors
    }).or_die();
    if hash_ok.load(SeqCst) { return } // already installed

    wsl_root(distro).args(&["rm", "/opt/gcw0-toolchain/sha256"]).io(|_|{}, |_|{}).or_die();
    let tmp_path = std::env::temp_dir().join(GCW0_TOOLCHAIN.sha256);
    std::fs::write(&tmp_path, GCW0_TOOLCHAIN.download()).unwrap_or_else(|err| fatal!("failed to write to {}: {}", tmp_path.display(), err));
    let wsl_path = wslify_path(&tmp_path);
    let ci = std::env::var_os("CI").is_some();
    let start = std::time::Instant::now();
    let files  = AtomicUsize::new(0);
    let quanta = AtomicUsize::new(0);
    let extract = wsl_root(distro).args(&["tar", "jxvf"]).arg(&wsl_path).arg("-C").arg("/opt").io0(
        move |path| {
            let files = files.fetch_add(1, Relaxed);
            let elapsed = std::time::Instant::now() - start;
            let ms = elapsed.as_millis();
            let new_quanta = if ci { ms / 5000 } else { ms / 33 } as usize; // wrapping is OK here
            let old_quanta = quanta.swap(new_quanta, Relaxed);
            if new_quanta != old_quanta {
                let n = GCW0_TOOLCHAIN_ENTRIES;
                let pct = files * 100 / n;
                eprint!("\u{001B}[s  \u{001B}[36;1mExtracting\u{001B}[0m {: >5}/{}  {: >2}%  /opt/{}\u{001B}[0J\u{001B}[u", files, n, pct, path.trim());
            }
        },
        |e| fatal!("{}", e),
    );
    eprint!("\u{001B}[0J");
    status!("Extracted", "/opt/gcw0-toolchain/");
    extract.or_die();
    wsl_root(distro).args(&["echo", GCW0_TOOLCHAIN.sha256, ">", "/opt/gcw0-toolchain/sha256"]).status0().or_die();
}

fn wslify_path(path: impl AsRef<Path>) -> OsString {
    let path = path.as_ref();
    let path = path.canonicalize().unwrap_or_else(|err| fatal!("unable to WSLify path {}: canonicalization failed: {}", path.display(), err));
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

    o
}

#[test] fn test_wslify_path() {
    assert_eq!(wslify_path(r"C:\Windows\System32"), "/mnt/c/Windows/System32");
}
