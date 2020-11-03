use crate::*;

use mmrbi::*;
use mmrbi::cargo::metadata::Package;
use mmrbi::cargo::toml::package;
use mmrbi::fs::write_if_modified_with as wimw;

use std::collections::*;
use std::ffi::*;
use std::io::{self, BufRead, Write};
use std::path::*;
use std::process::Command;



pub fn dot_container(meta: &ContainerToml) -> PathBuf {
    let path = meta.root_directory().join(".container");
    match std::fs::create_dir(&path) {
        Ok(()) => {
            wimw(path.join(".gitignore"), |o| {
                writeln!(o, "# Generated by cargo-container")?;
                writeln!(o, "*")?;
                Ok(())
            }).or_die();
        },
        Err(err) if err.kind() == io::ErrorKind::AlreadyExists => {},
        Err(err) => fatal!("unable to create `{}`: {}", path.display(), err),
    }
    path
}

pub fn workspace_toml(meta: &ContainerToml) {
    const WARNING_COMMENT : &'static str = "# DO NOT EDIT BY HAND - AUTOGENERATED BY cargo-container FROM Container.toml";

    let path = meta.root_directory().join("Cargo.toml");
    match std::fs::File::open(&path) {
        Ok(file) => {
            let file = io::BufReader::new(file);
            let first_line = file.lines().next().unwrap_or(Ok(String::new())).unwrap_or(String::new());
            if first_line != WARNING_COMMENT { fatal!("unable to overwrite `{}`: missing expected warning comment: `{}`", path.display(), WARNING_COMMENT) }
        },
        Err(err) if err.kind() == io::ErrorKind::NotFound => {},
        Err(err) => fatal!("unable to check `{}`: {}", path.display(), err),
    }

    wimw(path, |o| {
        writeln!(o, "{}", WARNING_COMMENT)?;
        writeln!(o)?;

        if !meta.local_install.is_empty() {
            writeln!(o, "[workspace.metadata.local-install]")?;
            for (k, v) in meta.local_install.iter() {
                writeln!(o, "{} = {}", toml::to_string(k).unwrap(), toml_util::to_string_single_line(v))?;
            }
            writeln!(o)?;
        }

        writeln!(o, "[workspace]")?;
        writeln!(o, "members = [")?;
        for member in meta.workspace.members.iter() {
            writeln!(o, "    {},", toml::to_string(member).unwrap())?;
        }
        writeln!(o, "    {:?}", ".container/crates/*/*")?;
        writeln!(o, "]")?;
        writeln!(o, "exclude = [")?;
        for exclude in meta.workspace.exclude.iter() {
            writeln!(o, "    {},", toml::to_string(exclude).unwrap())?;
        }
        writeln!(o, "]")?;

        if !meta.profile.is_empty() {
            writeln!(o)?;
            writeln!(o, "[profile]")?;
            for (k, v) in meta.profile.iter() {
                writeln!(o, "{} = {}", toml::to_string(k).unwrap(), toml_util::to_string_single_line(v))?;
            }
        }

        Ok(())
    }).or_die();

    let zzz_stub = meta.root_directory().join(".container/crates/zzz/stub");
    std::fs::create_dir_all(&zzz_stub).or_die();
    wimw(zzz_stub.join("Cargo.toml"), |o| {
        writeln!(o, "{}", WARNING_COMMENT)?;
        writeln!(o)?;
        writeln!(o, "[package]")?;
        writeln!(o, "name    = {:?}", "zzz-stub")?;
        writeln!(o, "version = {:?}", "0.0.0")?;
        writeln!(o, "publish = {:?}", false)?;
        writeln!(o)?;
        writeln!(o, "[lib]")?;
        writeln!(o, "path    = {:?}", "zzz-stub.rs")?;
        Ok(())
    }).or_die();

    wimw(zzz_stub.join("zzz-stub.rs"), |o| {
        writeln!(o, "// Auto-Generated by cargo-container")?;
        writeln!(o, "//")?;
        writeln!(o, "// This crate only exists to ensure \".container/crates/*/*\" always")?;
        writeln!(o, "// has something to match, even when no tools generate crates.")?;
        writeln!(o, "#![doc(hidden)]")?;
        Ok(())
    }).or_die();
}

pub fn crates(meta: &ContainerToml) {
    let packages = meta.resolve_packages().unwrap_or_else(|err| fatal!("unable to resolve packages: {}", err));

    let mut gen = BTreeMap::<package::Name, BTreeSet<package::Name>>::new();
    for build in meta.builds.iter() {
        for tool in build.tools.iter() {
            let gen = gen.entry(tool.clone()).or_default();
            for krate in build.crates.iter() {
                gen.insert(krate.clone());
            }
        }
    }

    let path = prepend_paths(Some("bin"));

    for (tool, crates) in gen.iter() {
        let mut cmd = Command::new(tool.as_str());
        cmd.env("PATH", &path);
        cmd.env("CARGO_CONTAINER_COMMAND",      "generate");
        cmd.env("CARGO_CONTAINER_CRATES_DIR",   format!(".container/crates/{}", tool));
        cmd.env("CARGO_CONTAINER_CONFIGS",      "debug,release"); // XXX
        gather_crates(&mut cmd, meta, &packages, crates.iter());
        cmd.status0().unwrap_or_else(|err| fatal!("`{}` generate failed: {}", tool, err));
    }
}

fn gather_crates<'p>(cmd: &mut Command, meta: &ContainerToml, packages: &BTreeMap<package::Name, Package>, names: impl Iterator<Item = &'p package::Name>) {
    let mut o = String::new();
    for name in names {
        let file = &packages[name];
        let mut path = OsString::from("../../../..");
        let pd = file.directory().strip_prefix(meta.root_directory()).unwrap();
        for c in pd.components() {
            path.push("/");
            path.push(c);
        }

        cmd.env(format!("CARGO_CONTAINER_PACKAGE_{}_PATH",          file.package.name), path);
        cmd.env(format!("CARGO_CONTAINER_PACKAGE_{}_VERSION",       file.package.name), file.package.version.as_str());
        cmd.env(format!("CARGO_CONTAINER_PACKAGE_{}_DESCRIPTION",   file.package.name), file.package.description.as_ref().map_or("", |d| d.as_str()));
        if !o.is_empty() { o.push(',') }
        o.push_str(&file.package.name.as_str());
    }
    cmd.env("CARGO_CONTAINER_PACKAGES", o);
}
