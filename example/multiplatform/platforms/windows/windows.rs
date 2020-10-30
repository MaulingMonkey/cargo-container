use platform_common::*;

use mmrbi::*;
use mmrbi::fs::write_if_modified_with as wimw;

use std::collections::BTreeSet;
use std::io::Write;



// For linux 64-bit:
//      sudo apt-get install gcc-mingw-w64-x86-64
//      sudo apt-get install mingw-w64
//      rustup target add x86_64-pc-windows-gnu
// Resolves error:
//      error: linker `x86_64-w64-mingw32-gcc` not found

// For linux 32-bit:
//      rustup target add x86_64-pc-windows-gnu
//      sudo apt-get install gcc-mingw-w64-i686
//      sudo apt-get install mingw-w64
// Resolves error:
//      error: linker `i686-w64-mingw32-gcc` not found
// binutils-mingw-w64-i686  <-- windres only perhaps?

// http://michellcomputing.co.uk/blog/2016/05/windres-installation-on-linux/
// sudo apt-get install mingw-w64
// sudo ln -s /usr/bin/x86_64-w64-mingw32-windres /usr/bin/windres ?

fn main() {
    platform_common::exec(Tool, "windows")
}

struct Tool;

impl Tool {
    fn targets(&self, state: &State) -> BTreeSet<Option<&'static str>> {
        let rustup = mmrbi::Rustup::default().unwrap_or_else(|err| fatal!("unable to find rustup: {}", err));
        let toolchain = rustup.toolchains().active().unwrap_or_else(|| fatal!("no active rustup toolchain"));

        let aarch64 = toolchain.as_str().contains("-aarch64-");
        let x86_64  = toolchain.as_str().contains("-x86_64-");
        let i686    = toolchain.as_str().contains("-i686-");
        let i586    = toolchain.as_str().contains("-i586-");

        let mut targets = BTreeSet::new();
        if toolchain.as_str().ends_with("-msvc") {
            if state.arches.contains("aarch64"  ).unwrap_or(aarch64) { targets.insert(Some("aarch64-pc-windows-msvc"  )); }
            if state.arches.contains("x86_64"   ).unwrap_or(x86_64 ) { targets.insert(Some("x86_64-pc-windows-msvc"   )); }
            if state.arches.contains("x86"      ).unwrap_or(false  ) { targets.insert(Some("i686-pc-windows-msvc"     )); }
            if state.arches.contains("i686"     ).unwrap_or(i686   ) { targets.insert(Some("i686-pc-windows-msvc"     )); }
            if state.arches.contains("i586"     ).unwrap_or(i586   ) { targets.insert(Some("i586-pc-windows-msvc"     )); }
        } else if toolchain.as_str().ends_with("-gnu") {
            if state.arches.contains("aarch64"  ).unwrap_or(aarch64) { targets.insert(Some("aarch64-pc-windows-gnu"   )); }
            if state.arches.contains("x86_64"   ).unwrap_or(x86_64 ) { targets.insert(Some("x86_64-pc-windows-gnu"    )); }
            if state.arches.contains("x86"      ).unwrap_or(false  ) { targets.insert(Some("i686-pc-windows-gnu"      )); }
            if state.arches.contains("i686"     ).unwrap_or(i686   ) { targets.insert(Some("i686-pc-windows-gnu"      )); }
            if state.arches.contains("i586"     ).unwrap_or(i586   ) { targets.insert(Some("i586-pc-windows-gnu"      )); }
        } else {
            warning!("expected '*-msvc' or '*-gnu' toolchain but got {} - will try the default target", toolchain);
        }
        if targets.is_empty() {
            targets.insert(None);
        }
        targets
    }
}

impl platform_common::Tool for Tool {
    fn setup(&self, state: &State) {
        let rustup = mmrbi::Rustup::default().unwrap_or_else(|err| fatal!("unable to find rustup: {}", err));
        let toolchain = rustup.toolchains().active().unwrap_or_else(|| fatal!("no active rustup toolchain"));

        let aarch64 = toolchain.as_str().contains("-aarch64-");
        let x86_64  = toolchain.as_str().contains("-x86_64-");
        let i686    = toolchain.as_str().contains("-i686-");
        let i586    = toolchain.as_str().contains("-i586-");

        if toolchain.as_str().ends_with("-msvc") {
            if state.arches.contains("aarch64"  ).unwrap_or(aarch64       ) { toolchain.targets().add("aarch64-pc-windows-msvc"  ).or_die() }
            if state.arches.contains("x86_64"   ).unwrap_or(x86_64        ) { toolchain.targets().add("x86_64-pc-windows-msvc"   ).or_die() }
            if state.arches.contains("x86"      ).unwrap_or(false         ) { toolchain.targets().add("i686-pc-windows-msvc"     ).or_die() }
            if state.arches.contains("i686"     ).unwrap_or(i686 || x86_64) { toolchain.targets().add("i686-pc-windows-msvc"     ).or_die() }
            if state.arches.contains("i586"     ).unwrap_or(i586          ) { toolchain.targets().add("i586-pc-windows-msvc"     ).or_die() }
        } else if toolchain.as_str().ends_with("-gnu") {
            if state.arches.contains("aarch64"  ).unwrap_or(aarch64       ) { toolchain.targets().add("aarch64-pc-windows-gnu"  ).or_die() }
            if state.arches.contains("x86_64"   ).unwrap_or(x86_64        ) { toolchain.targets().add("x86_64-pc-windows-gnu"   ).or_die() }
            if state.arches.contains("x86"      ).unwrap_or(false         ) { toolchain.targets().add("i686-pc-windows-gnu"     ).or_die() }
            if state.arches.contains("i686"     ).unwrap_or(i686 || x86_64) { toolchain.targets().add("i686-pc-windows-gnu"     ).or_die() }
            if state.arches.contains("i586"     ).unwrap_or(i586          ) { toolchain.targets().add("i586-pc-windows-gnu"     ).or_die() }
        } else {
            warning!("unable to setup platform-windows for toolchain {}: expected '*-msvc' or '*-gnu'", toolchain);
        }
    }

    fn generate(&self, state: &State) {
        append_setup_script("linux.sh", |o|{
            writeln!(o)?;
            writeln!(o, "# AUTOGENERATED BY {}", env!("CARGO_PKG_NAME"))?;
            writeln!(o, "if ! x86_64-w64-mingw32-gcc --version >/dev/null 2>/dev/null; then")?;
            writeln!(o, "    for_apt_get -y install gcc-mingw-w64-x86-64")?;
            writeln!(o, "fi")?;
            writeln!(o, "if ! i686-w64-mingw32-gcc --version >/dev/null 2>/dev/null; then")?;
            writeln!(o, "    for_apt_get -y install gcc-mingw-w64-i686")?;
            writeln!(o, "fi")?;
            Ok(())
        });

        for package in state.packages.iter() {
            let out_dir = package.generated_path();
            std::fs::create_dir_all(&out_dir).unwrap_or_else(|err| fatal!("unable to create `{}`: {}", out_dir.display(), err));

            wimw(out_dir.join("main.rs"), |o|{
                writeln!(o, "// AUTOGENERATED BY {}", env!("CARGO_PKG_NAME"))?;
                writeln!(o, "#![windows_subsystem=\"windows\"]")?;
                writeln!(o)?;
                writeln!(o, "fn main() {{ app::init(app_common::WindowsDialogProvider) }}")?;
                Ok(())
            }).or_die();

            wimw(out_dir.join("build.rs"), |o|{
                writeln!(o, "// AUTOGENERATED BY {}", env!("CARGO_PKG_NAME"))?;
                writeln!(o, "fn main() {{")?;
                writeln!(o, "    natvis_pdbs::metabuild();")?;
                writeln!(o, "    winres::WindowsResource::new().compile().unwrap_or_else(|err| println!(\"cargo:warning=winres failed for {{}}: {{}}\", env!(\"CARGO_PKG_NAME\"), err));")?;
                writeln!(o, "}}")?;
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
                writeln!(o, "app-common      = {{ path = {:?}, features = [{:?}] }}", "../../../../app-common", "platform-windows")?;
                writeln!(o, "app             = {{ path = {:?}, package = {:?} }}",    package.original_path(), package.original_name())?;
                writeln!(o)?;
                writeln!(o, "[build-dependencies]")?;
                writeln!(o, "winres         = {:?}", "0.1")?;
                writeln!(o, "natvis-pdbs    = {:?}", "1")?;
                writeln!(o)?;
                writeln!(o, "[[bin]]")?;
                writeln!(o, "name            = {:?}", package.original_name())?;
                writeln!(o, "path            = {:?}", "main.rs")?;
                Ok(())
            }).or_die();
        }
    }

    fn build(&self, state: &State) {
        let targets = self.targets(state);
        for config in state.configs.iter() {
            for target in targets.iter().copied() {
                let mut cmd = Command::new("cargo");
                cmd.args(&["build"]);
                if let Some(target) = target {
                    cmd.arg("--target").arg(target);
                }
                match config.name() {
                    "debug"     => {},
                    "release"   => { cmd.arg("--release"); },
                    other       => fatal!("unexpected config: {:?}", other),
                }
                for package in state.packages.iter() { cmd.arg("-p"); cmd.arg(&package.generated_name()); }
                cmd.status0().or_die()
            }
        }
    }

    fn test(&self, state: &State) {
        if !cfg!(windows) {
            warning!("skipping tests - `cargo test --target *-pc-windows-*` requires windows");
            return;
        }

        let targets = self.targets(state);
        for config in state.configs.iter() {
            for target in targets.iter().copied() {
                let mut cmd = Command::new("cargo");
                cmd.args(&["test"]);
                if let Some(target) = target {
                    cmd.arg("--target").arg(target);
                }
                match config.name() {
                    "debug"     => {},
                    "release"   => { cmd.arg("--release"); },
                    other       => fatal!("unexpected config: {:?}", other),
                }
                for package in state.packages.iter() { cmd.arg("-p"); cmd.arg(&package.generated_name()); }
                cmd.status0().or_die()
            }
        }
    }
}
