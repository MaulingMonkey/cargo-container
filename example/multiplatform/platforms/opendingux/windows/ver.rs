use platform_common::mmrbi::*;

pub fn version() -> Option<(u64, u64, u64, u64)> {
    if !cfg!(windows) {
        None
    } else {
        let ver = Command::new("cmd").arg("/C").arg("ver").stdout0().ok()?;
        let ver = ver.trim().strip_prefix("Microsoft Windows [Version ")?.strip_suffix("]")?;
        let mut ver = ver.splitn(4, '.');

        let major = ver.next()?.parse().ok()?;
        let minor = ver.next()?.parse().ok()?;
        let patch = ver.next()?.parse().ok()?;
        let build = ver.next()?.parse().ok()?;
        Some((major, minor, patch, build))
    }
}
