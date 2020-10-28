use platform_common::*;

use mmrbi::*;
use mmrbi::fs::write_if_modified_with as wimw;

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

const PREFERRED_TARGET : &'static [&'static str] = &[
    "x86_64-pc-windows-msvc",
    "i686-pc-windows-msvc",
    "i586-pc-windows-msvc",

    "x86_64-pc-windows-gnu",
    "i686-pc-windows-gnu",
    "i586-pc-windows-gnu",
];

fn main() {
    let target = {
        let installed = mmrbi::Rustup::default().or_die().toolchains().active().expect("no toolchain active").targets().installed();
        PREFERRED_TARGET.iter().copied().filter(|t| installed.contains(*t)).next().unwrap_or_else(||{
            fatal!("Unable to find {{x86_64,i686,i586}}-pc-windows-{{msvc,gnu}}");
        })
    };
    platform_common::exec(Tool { target }, "windows")
}

struct Tool {
    target: &'static str,
}

impl platform_common::Tool for Tool {
    fn generate(&self, state: &State) {
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
        for config in state.configs.iter() {
            let mut cmd = Command::new("cargo");
            cmd.args(&["build"]);
            cmd.arg("--target").arg(self.target);
            match config.name() {
                "debug"     => {},
                "release"   => { cmd.arg("--release"); },
                other       => fatal!("unexpected config: {:?}", other),
            }
            for package in state.packages.iter() { cmd.arg("-p"); cmd.arg(&package.generated_name()); }
            cmd.status0().or_die()
        }
    }

    fn test(&self, state: &State) {
        if !cfg!(windows) {
            warning!("skipping tests - `cargo test --target *-pc-windows-*` requires windows");
            return;
        }

        for config in state.configs.iter() {
            let mut cmd = Command::new("cargo");
            cmd.args(&["test"]);
            cmd.arg("--target").arg(self.target);
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
