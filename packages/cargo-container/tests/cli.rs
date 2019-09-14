use std::process::{Command, Stdio};

fn cargo_container(args: &str) -> Option<i32> {
    let args = args.split_ascii_whitespace();
    Command::new("cargo")
        .args(&["run", "-p", "cargo-container", "--", "container"])
        .args(args)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .unwrap()
        .code()
}

#[test]
fn exit_codes() {
    assert_eq!(Some(0), cargo_container(""));
    assert_eq!(Some(0), cargo_container("help"));
    assert_eq!(Some(1), cargo_container("xxx-non-existant-subcommand"));
}
