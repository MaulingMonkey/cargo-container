use mmrbi::env;

use std::ffi::OsString;
use std::path::Path;



pub fn prepend_paths<I: IntoIterator<Item = P>, P: AsRef<Path>>(paths: I) -> OsString {
    let mut o = OsString::new();
    for path in paths {
        o.push(path.as_ref());
        o.push(if cfg!(windows) { ";" } else { ":" });
    }
    o.push(env::req_var_os("PATH"));
    o
}
