use sha2::Digest;

use mmrbi::*;

use std::fmt::Write as _ ;
use std::io::{Read, Write};
use std::path::{Path};



pub struct Download {
    pub name:   &'static str,
    pub url:    &'static str,
    pub sha256: &'static str,
}

impl Download {
    pub fn download(&self) -> impl AsRef<[u8]> {
        status!("Downloading", "{} ({})", self.name, self.url);
        let download = reqwest::blocking::Client::builder()
            .user_agent("github.com/MaulingMonkey/cargo-container/example/multiplatform/platforms/common")
            .build().or_die()
            .get(self.url)
            .send().or_die()
            .bytes().or_die();

        let mut hasher = sha2::Sha256::new();
        hasher.update(download.as_ref());
        let mut hash = String::new();
        for b in hasher.finalize().into_iter() {
            let _ = write!(&mut hash, "{:02X}", b);
        }

        if self.sha256 != hash {
            fatal!("expected hash {}\r\nbut got hash {}", self.sha256, hash);
        }

        download
    }

    pub fn download_gunzip(&self) -> impl AsRef<[u8]> {
        let mut o = Vec::new();
        libflate::gzip::Decoder::new(self.download().as_ref())
            .unwrap_or_else(|err| fatal!("failed to gunzip: {}", err))
            .read_to_end(&mut o)
            .unwrap_or_else(|err| fatal!("failed to gunzip: {}", err));
        o
    }

    pub fn download_gunzip_to(&self, to: impl AsRef<Path>, _unix_mode: u32) {
        let mut o = std::fs::File::create(to.as_ref()).unwrap_or_else(|err| fatal!("failed to create {}: {}", to.as_ref().display(), err));
        o.write_all(self.download_gunzip().as_ref()).unwrap_or_else(|err| fatal!("failed to write to {}: {}", to.as_ref().display(), err));
        #[cfg(unix)] {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = o.metadata().unwrap_or_else(|err| fatal!("failed to get permissions for {}: {}", to.as_ref().display(), err)).permissions();
            perms.set_mode(_unix_mode);
            o.set_permissions(perms).unwrap_or_else(|err| fatal!("failed to set permissions for {}: {}", to.as_ref().display(), err));
        }
    }

    pub fn download_gunzip_untar_entry_to(&self, entry: impl AsRef<Path>, to: impl AsRef<Path>, _unix_mode: u32) {
        let mut tar = tar::Archive::new(std::io::Cursor::new(self.download_gunzip()));
        for e in tar.entries().unwrap_or_else(|err| fatal!("failed to read tar entries: {}", err)) {
            let mut e = e.unwrap_or_else(|err| fatal!("failed to read tar entry: {}", err));
            let path = e.path().unwrap_or_else(|err| fatal!("failed to read tar entry path: {}", err));
            if path == entry.as_ref() {
                e.unpack(to.as_ref()).unwrap_or_else(|err| fatal!("failed to unpack {} to {}: {}", entry.as_ref().display(), to.as_ref().display(), err));
                #[cfg(unix)] {
                    use std::os::unix::fs::PermissionsExt;
                    let mut perms = to.as_ref().metadata().unwrap_or_else(|err| fatal!("failed to get permissions for {}: {}", to.as_ref().display(), err)).permissions();
                    perms.set_mode(_unix_mode);
                    std::fs::set_permissions(to.as_ref(), perms).unwrap_or_else(|err| fatal!("failed to set permissions for {}: {}", to.as_ref().display(), err));
                }
                return;
            }
        }
        fatal!("unable to find {} in archive", entry.as_ref().display());
    }
}
