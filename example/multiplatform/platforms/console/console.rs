use platform_common::*;

use mmrbi::{fatal, CommandExt, ResultExt};
use mmrbi::env::*;
use mmrbi::fs::write_if_modified_with as wimw;

use std::io::Write;
use std::process::{Command, exit};



trait PackageExt { fn generated_name(&self) -> String; }
impl PackageExt for Package { fn generated_name(&self) -> String { format!("{}-console", self.original_name()) } }

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
            let mut cmd = Command::new("cargo");
            cmd.args(&["build"]);
            match config.name() {
                "debug"     => {},
                "release"   => { cmd.arg("--release"); },
                other       => fatal!("unexpected config: {:?}", other),
            }
            for package in self.packages.iter() { cmd.arg("-p"); cmd.arg(&package.generated_name()); }
            cmd.status0().or_die();
        }
    }

    fn run(&self) {
        // TODO: config selection?
        // TODO: package(s) selection?
        fatal!("run not yet implemented");
    }

    fn test(&self) {
        for config in self.configs.iter() {
            let mut cmd = Command::new("cargo");
            cmd.args(&["test"]);
            match config.name() {
                "debug"     => {},
                "release"   => { cmd.arg("--release"); },
                other       => fatal!("unexpected config: {:?}", other),
            }
            for package in self.packages.iter() { cmd.arg("-p"); cmd.arg(&package.generated_name()); }
            cmd.status0().or_die();
        }
    }

    fn generate(&self) {
        for package in self.packages.iter() {
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
}
