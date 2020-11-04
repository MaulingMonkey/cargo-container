# `cargo container setup`

This command is designed to allow tools to download, extract, install, and
configure any missing dependencies the tool might have.  Examples include:

* `sudo apt-get update && apt-get -y install ...` to install missing linux packages
* `dism /online /enable-feature /featureName:VirtualMachinePlatform /all /norestart` to enable [WSL] 2
* `powershell Add-AppxPackage -Path ...` to install Ubuntu on WSL
* `wsl --import ... && wsl --set-version ...` to create & configure new WSL 2 VMs
* `wsl curl ... | tar jxv -C /opt` to download & extract GCW0 tarballs
* `rustup target add ...` for cross compilation
* `cargo local-install wasm-pack` for build tools

This is intended for use both by developers using said tools, and by CI servers
which might need to install said tools from scratch.  To customize behavior,
consider running with some of the following arguments:

* `cargo container setup --tool [tool1] --tool [tool2]` to setup specified tools instead of all of them
* `cargo container setup --arch aarch64 --arch x86_64` to be explicit about what architectures to cross compile for, instead of guessing
* `cargo container setup --allow-sudo` to accept the "Run these commands?" prompt for CI builds

To implement this, `cargo container` will:

*   Install any `[local-install]` packages from `Container.toml` to `bin\*`
*   Invoke each `tool` (by searching `%PATH%`) found in `Container.toml`, without arguments, with the following env vars:
    | Environment Variable      | Value         |
    | ------------------------- | ------------- |
    | `PATH`                    | `bin;%PATH%`  |
    | `CARGO_CONTAINER_COMMAND` | `setup`       |
    | `CARGO_CONTAINER_ARCHES`  | (blank by default)
*   Parsing `stdout` for the following directives, on top of letting the tool do whatever else it might want to do:
    | Stdout Directive                              | Description   |
    | --------------------------------------------- | ------------- |
    | `cargo-container:sudo=[command]`              | Request `[command]` be run as an administrator (Windows) or root (Linux, OS X, ...)
    | `cargo-container:apt-get-install=[package]`   | Request `apt-get update && apt-get install -y [package]` be run as root on linux.  Combined and deduplicated with other install requests.
    | `cargo-container:error=[message]`             | Display an `error:` message (+ increment any error counts)
    | `cargo-container:warning=[message]`           | Display a `warning:` message (+ increment any warning counts)
    | `cargo-container:info=[message]`              | Display an `info:` message
    Admin/root commands will be aggregated and run with `cmd.exe` (windows) or `sh` (\*nix), to require only a single elevation or password prompt.
    For bonus points, `cargo container` will by default display the commands and let you choose to run them or not.

Well behaved tools will detect what's already installed to allow spamming the
setup command, and filter outputs to show progress indicators / reduce spam.
Additionally, for target-specific tools, they should install cross compilers if
at all possible by default if necessary.  `cargo-container` should handle
filtering tools to those appropriate for the host if possible.



<!-- # References -->

[WSL]:  https://en.wikipedia.org/wiki/Windows_Subsystem_for_Linux
