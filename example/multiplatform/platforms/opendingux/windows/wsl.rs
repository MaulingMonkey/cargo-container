use crate::*;

use platform_common::*;

use mmrbi::*;

use winapi::um::errhandlingapi::GetLastError;
use winapi::um::shellapi::ShellExecuteW;
use winapi::um::winuser::SW_SHOWDEFAULT;

use std::ffi::OsString;
use std::os::windows::prelude::*;
use std::path::{Component, Path, PathBuf, Prefix};
use std::ptr::null_mut;
use std::sync::{Arc, atomic::{AtomicBool, AtomicU32, AtomicUsize, Ordering::{SeqCst, Relaxed}}};



pub fn ensure_distro_installed(wsl: &wslapi::Library) -> &'static str {
    if let Some(distro) = distros().find(|d| wsl.is_distribution_registered(d)) { return distro; }

    // Unnecessary - or perhaps auto-invoked by winrt?
    //use winapi::shared::winerror::SUCCEEDED;
    //use winapi::winrt::roapi::*;
    //let hr = unsafe { RoInitialize(RO_INIT_MULTITHREADED) };
    //if !SUCCEEDED(hr) { fatal!("RoInitialize(RO_INIT_MULTITHREADED) failed with HRESULT 0x{:08x}", hr); }

    for pfn in PFNS.iter().copied() {
        if install_distro_from_pfn(wsl, pfn) { return DISTRO_ID; }
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
    appx::repository::add_appx_package(&tmp_appx_path).or_die();
    if install_distro_from_pfn(wsl, UBUNTU_PFN) { return DISTRO_ID; }

    error!("unable to install {} from {}", DISTRO_ID, UBUNTU_PFN);
    eprintln!("Consider installing from one of the following sources:");
    for pfn in PFNS.iter().copied() {
        eprintln!("    ms-windows-store://pdp/?PFN={}", pfn);
    }
    eprintln!("And then re-run `cargo container setup`");
    open_url(&format!("ms-windows-store://pdp/?PFN={}", UBUNTU_PFN));
    std::process::exit(1);
}

#[allow(deprecated)] // std::env::home_dir
pub fn install_distro_from_pfn(_wsl: &wslapi::Library, pfn: &str) -> bool {
    let pfn = appx::PackageFamilyName::new(pfn);
    for package in appx::repository::packages_for_family(&pfn).unwrap() {
        let path = package.install_location().unwrap_or_else(|err| fatal!("unable to get Package({:?})->InstalledLocation: {:?}", pfn, err));
        let install_tar_gz = path.join("install.tar.gz");
        if install_tar_gz.exists() {
            status!("Installing", "{} from {} (this may take a minute)", DISTRO_ID, install_tar_gz.display());
            let home = std::env::home_dir().unwrap_or_else(|| fatal!("unable to determine home directory"));
            let distro_dir = home.join(format!(r".cargo\container\{}", DISTRO_ID));

            if true {
                // --import requires build 18305 https://docs.microsoft.com/en-us/windows/wsl/release-notes#build-18305
                // travis and appveyor are only on build 17763
                Command::new("wsl").arg("--import").arg(DISTRO_ID).arg(&distro_dir).arg(&install_tar_gz).status0().unwrap_or_else(|err| fatal!(
                    "unable to import distro {} from {} into {}: {}", DISTRO_ID, install_tar_gz.display(), distro_dir.display(), err
                ));
                status!("Converting", "{} to WSL 2", DISTRO_ID);
                Command::new("wsl").arg("--set-version").arg(DISTRO_ID).arg("2").status0().unwrap_or_else(|err| fatal!(
                    "unable to convert distro {} to WSL 2: {}", DISTRO_ID, err
                ));
            } else {
                // WslLaunch is a troll API.  Specifically, it uses the directory of the executable to install the distribution into.
                // Not the current directory, not the directory of argv[0] if you symlinked that, the directory of the *executable*.
                // I'm pretty sure that was frowned upon as far back as Windows XP - generally being the program files directory
                // instead of a per-user directory - but I digress.  Long story short, we copy ourselves into `distro_dir` and invoke
                // ourselves as a hackaround.
                let exe = PathBuf::from(std::env::args_os().next().unwrap_or_else(|| fatal!("can't get exe name for self reinvoke")));
                let distro_exe = distro_dir.join("odx.exe");
                std::fs::remove_file(&distro_exe).unwrap_or_else(|err| if err.kind() != std::io::ErrorKind::NotFound { fatal!("unable to remove distro exe `{}`: {}", distro_exe.display(), err) });
                // symlinks won't work here, but hardlinks might if on the same drive?
                std::fs::copy(&exe, &distro_exe).unwrap_or_else(|err| fatal!("unable to copy distro exe `{}` to `{}`: {}", distro_exe.display(), exe.display(), err));
                Command::new(&distro_exe).arg("wsl-register-distro-hack")
                    .arg(DISTRO_ID).arg(&install_tar_gz)
                    .current_dir(&distro_dir) // Does nothing in my current wslapi.dll version... but I could see other, saner versions of wslapi.dll reading it for installation locations?
                    .status0().or_die();
                std::fs::remove_file(&distro_exe).unwrap_or_else(|err| fatal!("unable to remove distro exe `{}`: {}", distro_exe.display(), err));
            }
            // wslapi seems to cache information poorly?  This check seems 100% unreliable if installing through `wsl` command line *or* through `wslapi.dll`
            // if !_wsl.is_distribution_registered(DISTRO_ID) { ... }
            return true;
        } else {
            warning!("missing {} (expected to exist thanks to {} but it doesn't)", install_tar_gz.display(), pfn);
        }
    }
    false
}

pub fn root(distro: &str) -> Command {
    let mut c = Command::new("wsl");
    c.arg("--distribution").arg(distro).arg("--user").arg("root").arg("--");
    c
}

pub fn od(distro: &str) -> Command {
    let mut c = Command::new("wsl");
    c.arg("--distribution").arg(distro).arg("--user").arg("opendingux").arg("--");
    c
}

pub fn open_url(url: &str) {
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

pub fn ensure_root_stuff(wsl: &wslapi::Library, distro: &str) {
    // Create "opendingux" user
    root(distro).args(&["adduser", "--disabled-password", "--gecos", "User for building opendingux packages", "--system", "--shell", "/bin/bash", "opendingux"]).io0(|line|{
        if line.trim_end().ends_with(" already exists. Exiting.") { return } // ignore spam if user already exists
        println!("{}", line);
    }, |line| {
        if line.trim_end().ends_with(" already exists. Exiting.") { return } // ignore spam if user already exists
        eprintln!("{}", line);
    }).or_die();

    // Use "opendingux" user by default
    let uid = Arc::new(AtomicU32::new(0));
    let uid2 = uid.clone();
    od(distro).args(&["id", "-u"]).io0(move |id| uid2.store(id.parse().unwrap_or(0), SeqCst), |l| error!("failed to get user id: {}", l)).or_die();
    if let Ok(cfg) = wsl.get_distribution_configuration(distro) {
        wsl.configure_distribution(distro, uid.load(SeqCst), cfg.flags).or_die();
    } else {
        warning!("unable to set default user to \"opendingux\"");
    }

    // Update/install apt packages
    status!("Updating", "apt sources and packages");
    root(distro).args(&["dpkg", "--add-architecture", "i386"]).status0().or_die(); // gcw0-toolchain is all 32-bit executables so I need this
    root(distro).args(&["apt-get", "-y", "update"]).status0().or_die();
    root(distro).args(&["apt-get", "-y", "install", "libc6:i386", "libstdc++6:i386"]).status0().or_die(); // necessary for mipsel-gcw0-linux-gcc
}

pub fn ensure_gcw0_installed(_wsl: &wslapi::Library, distro: &str) {
    let hash_ok = Arc::new(AtomicBool::new(false));
    let hash_ok_io = hash_ok.clone();
    root(distro).args(&["cat", "/opt/gcw0-toolchain/sha256"]).io(move |sha256| {
        let sha256 = sha256.trim();
        hash_ok_io.store(sha256 == GCW0_TOOLCHAIN.sha256, SeqCst);
    }, |_| {
        // ignore errors
    }).or_die();
    if hash_ok.load(SeqCst) { return } // already installed

    root(distro).args(&["rm", "/opt/gcw0-toolchain/sha256"]).io(|_|{}, |_|{}).or_die();
    let tmp_path = std::env::temp_dir().join(GCW0_TOOLCHAIN.sha256);
    std::fs::write(&tmp_path, GCW0_TOOLCHAIN.download()).unwrap_or_else(|err| fatal!("failed to write to {}: {}", tmp_path.display(), err));
    let wsl_path = wslify_path(&tmp_path);
    let ci = std::env::var_os("CI").is_some();
    let start = std::time::Instant::now();
    let files  = AtomicUsize::new(0);
    let quanta = AtomicUsize::new(0);
    let extract = root(distro).args(&["tar", "jxvf"]).arg(&wsl_path).arg("-C").arg("/opt").io0(
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
    root(distro).args(&["echo", GCW0_TOOLCHAIN.sha256, ">", "/opt/gcw0-toolchain/sha256"]).status0().or_die();
}

pub fn wslify_path(path: impl AsRef<Path>) -> OsString {
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
