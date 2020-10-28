use platform_common::*;

use mmrbi::{*, fatal};
use mmrbi::env::*;
use mmrbi::fs::write_if_modified_with as wimw;

use std::io::Write;
use std::process::{Command, exit};



trait PackageExt {
    fn generated_name(&self) -> String;
    fn generated_target(&self) -> String { self.generated_name().replace("-", "_") }
}

impl PackageExt for Package {
    fn generated_name(&self) -> String { format!("{}-web-sys", self.original_name()) }
}

struct State {
    pub command:    String,
    pub packages:   Vec<Package>,
    pub configs:    Vec<Config>,
}

fn main() {
    State::get().exec();
}

impl State {
    pub fn get() -> Self {
        let command     = req_var_str("CARGO_CONTAINER_COMMAND");
        let configs     = Config::list();
        let packages    = Package::list();
        Self { command, packages, configs }
    }

    pub fn exec(&mut self) {
        match self.command.as_str() {
            "build"     => self.build(),
            "run"       => self.run(),
            "test"      => self.test(),
            "generate"  => self.generate(),
            _other      => exit(1),
        }
    }

    fn build(&self) {
        for config in self.configs.iter() {
            for package in self.packages.iter() {
                let mut cmd = Command::new("wasm-pack");
                cmd.current_dir(package.generated_path());
                let pkg_dir = format!("../../../../target/wasm32-unknown-unknown/{config}/{package}", config=config.name(), package=package.generated_name());
                cmd.args(&["build", "--no-typescript", "--target", "no-modules", "--out-dir", &pkg_dir]);
                match config.name() {
                    "debug"     => { cmd.arg("--dev"); },
                    "release"   => { cmd.arg("--profiling"); }, // release + debuginfo
                    other       => fatal!("unexpected config: {:?}", other),
                }
                cmd.status0().or_die();

                wimw(format!("target/wasm32-unknown-unknown/{config}/{package}/index.html", config=config.name(), package=package.generated_name()), |o|{
                    writeln!(o, "<!DOCTYPE html>")?;
                    writeln!(o, "<html lang=\"en\"><head>")?;
                    writeln!(o, "    <meta charset=\"UTF-8\">")?;
                    writeln!(o, "    <meta http-equiv=\"X-UA-Compatible\" content=\"IE=edge\">")?;
                    writeln!(o, "    <title>{}</title>", package.original_name())?;
                    writeln!(o, "</head><body>")?;
                    // what did these do again? IE pollyfills perhaps?
                    //writeln!(o, "    <script src="https://unpkg.com/text-encoding@0.6.4/lib/encoding-indexes.js"></script>")?;
                    //writeln!(o, "    <script src="https://unpkg.com/text-encoding@0.6.4/lib/encoding.js"></script>")?;
                    writeln!(o, "    <script src=\"{}.js\"></script>", package.generated_target())?;
                    writeln!(o, "    <script>")?;
                    writeln!(o, "        // fetch doesn't work over file:// despite using --allow-file-access-from-files, so use XHR instead.")?;
                    writeln!(o, "        var xhr = new XMLHttpRequest();")?;
                    writeln!(o, "        xhr.responseType = \"arraybuffer\";")?;
                    writeln!(o, "        xhr.addEventListener(\"error\", function(err) {{")?;
                    writeln!(o, "            debugger;")?;
                    writeln!(o, "        }});")?;
                    writeln!(o, "        xhr.addEventListener(\"load\", function(load) {{")?;
                    writeln!(o, "            wasm_bindgen(xhr.response); // .then(...)")?;
                    writeln!(o, "        }});")?;
                    writeln!(o, "        xhr.open(\"GET\", \"{}_bg.wasm\");", package.generated_target())?;
                    writeln!(o, "        xhr.send();")?;
                    writeln!(o, "    </script>")?;
                    writeln!(o, "</body></html>")?;
                    Ok(())
                }).or_die();
            }
        }
    }

    fn run(&self) {
        // TODO: config selection?
        // TODO: package(s) selection?
        fatal!("run not yet implemented");
    }

    fn test(&self) {
        for config in self.configs.iter() {
            for package in self.packages.iter() {
                let mut cmd = Command::new("wasm-pack");
                cmd.current_dir(package.generated_path());
                cmd.args(&["test", "--headless"]);

                // XXX: make this configurable somehow?  or auto-infer from installed browsers?
                //cmd.arg("--node");
                cmd.arg("--chrome");
                //cmd.arg("--firefox");
                //cmd.arg("--safari");

                match config.name() {
                    "debug"     => {},
                    "release"   => { cmd.arg("--release"); },
                    other       => fatal!("unexpected config: {:?}", other),
                }
                cmd.status0().or_die();
            }
        }
    }

    fn generate(&self) {
        for package in self.packages.iter() {
            let out_dir = package.generated_path();
            std::fs::create_dir_all(&out_dir).unwrap_or_else(|err| fatal!("unable to create `{}`: {}", out_dir.display(), err));

            wimw(out_dir.join("lib.rs"), |o|{
                writeln!(o, "// AUTOGENERATED BY {}", env!("CARGO_PKG_NAME"))?;
                writeln!(o)?;
                writeln!(o, "use app_common::wasm_bindgen;")?;
                writeln!(o, "use wasm_bindgen::prelude::*;")?;
                writeln!(o)?;
                writeln!(o, "#[wasm_bindgen(start)]")?;
                writeln!(o, "pub fn start() {{")?; // don't use "main" here - conflicts with wasm-pack test
                writeln!(o, "    app::init(app_common::WebSysDialogProvider);")?;
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
                writeln!(o, "app-common      = {{ path = {:?}, features = [{:?}] }}", "../../../../app-common", "platform-web-sys")?;
                writeln!(o, "app             = {{ path = {:?}, package = {:?} }}",    package.original_path(), package.original_name())?;
                writeln!(o)?;
                writeln!(o, "[lib]")?;
                writeln!(o, "crate-type      = [\"cdylib\", \"rlib\"]")?;
                writeln!(o, "name            = {:?}", package.generated_name().replace("-", "_"))?;
                writeln!(o, "path            = {:?}", "lib.rs")?;
                Ok(())
            }).or_die();
        }
    }
}
