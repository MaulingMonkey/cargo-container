use mmrbi::*;
use mmrbi::cargo::metadata::Package;
use mmrbi::cargo::toml::package;

use serde::*;

use std::collections::*;
use std::io;
use std::ops::Deref;
use std::path::*;



impl ContainerToml {
    pub fn from_current_dir() -> io::Result<Self> {
        Self::from_dir(std::env::current_dir().map_err(|err| io::Error::new(err.kind(), format!("unable to determine the current directory to find Container.toml: {}", err)))?)
    }

    pub fn from_dir(path: impl AsRef<Path> + Into<PathBuf>) -> io::Result<Self> {
        let original_dir = path.as_ref();
        let mut file = original_dir.join("Container.toml");
        while !file.exists() {
            if !file.pop() || !file.pop() { return Err(io::Error::new(
                io::ErrorKind::NotFound,
                format!("unable to find Container.toml in `{}` or it's parent directories", original_dir.display())
            ))}
            file.push("Container.toml");
        }
        Self::from_container_toml(file)
    }

    pub fn from_container_toml(path: impl AsRef<Path> + Into<PathBuf>) -> io::Result<Self> {
        let path_ref = path.as_ref();
        let bytes = std::fs::read(path_ref).map_err(|err| io::Error::new(err.kind(), format!("unable to read `{}`: {}", path_ref.display(), err)))?;
        Ok(Self {
            root: toml::from_slice(&bytes[..]).map_err(|err| io::Error::new(io::ErrorKind::InvalidData, err))?,
            path: path.into(),
        })
    }

    pub fn manifest_path(&self) -> &Path {
        self.path.as_path()
    }

    pub fn root_directory(&self) -> &Path {
        self.path.parent().unwrap()
    }

    pub fn create_dir_all(&self, path: impl AsRef<Path>) -> PathBuf {
        let path = self.root_directory().join(path);
        std::fs::create_dir_all(&path).unwrap_or_else(|err| fatal!("unable to create directory `{}`: {}", path.display(), err));
        path
    }

    pub fn resolve_packages(&self) -> io::Result<BTreeMap<package::Name, Package>> {
        Ok(cargo::Metadata::from_file_workspace(&self.path, self.root.workspace.clone()).packages.iter().map(|p| (p.package.name.clone(), p.clone())).collect())
    }
}

impl Deref for ContainerToml {
    type Target = Root;
    fn deref(&self) -> &Self::Target { &self.root }
}



pub struct ContainerToml {
    path:   PathBuf,
    root:   Root,
}

/// # Example
///
/// ```toml
/// [local-install]
/// platform-console = { path = "example/multiplatform/platforms/console" }
/// 
/// [workspace]
/// members = [
///     "cargo-container",
///     "example/multiplatform/app-common",
///     "example/multiplatform/apps/*",
///     "example/multiplatform/tools/*",
/// ]
/// 
/// [[build]]
/// crates  = ["alpha", "beta", "delta"]
/// tools   = ["platform-console"]
/// ```
#[derive(Deserialize)]
#[serde(rename_all = "kebab-case")]
#[non_exhaustive]
pub struct Root {
    #[serde(default)]                   pub local_install:  toml::value::Table,
    #[serde(default)]                   pub workspace:      cargo::toml::Workspace,
    #[serde(default, rename = "build")] pub builds:         Vec<Build>,
}

/// # Example
///
/// ```toml
/// # [[build]]
/// crates  = ["alpha", "beta", "delta"]
/// tools   = ["platform-console"]
/// ```
#[derive(Deserialize)]
#[non_exhaustive]
pub struct Build {
    pub crates: Vec<package::Name>,
    pub tools:  Vec<package::Name>,
}
