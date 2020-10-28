# cargo-container

Wrap vanilla cargo rlibs/packages in generated "containers" for various ends.

[![GitHub](https://img.shields.io/github/stars/MaulingMonkey/cargo-container.svg?label=GitHub&style=social)](https://github.com/MaulingMonkey/cargo-container)
[![crates.io](https://img.shields.io/crates/v/cargo-container.svg)](https://crates.io/crates/cargo-container)
[![License](https://img.shields.io/crates/l/cargo_container.svg)](https://github.com/MaulingMonkey/cargo-container)
[![Build Status](https://travis-ci.com/MaulingMonkey/cargo-container.svg?branch=master)](https://travis-ci.com/MaulingMonkey/cargo-container)
<!-- [![dependency status](https://deps.rs/repo/github/MaulingMonkey/cargo-container/status.svg)](https://deps.rs/repo/github/MaulingMonkey/cargo-container) -->

<h2 name="quickstart">Quickstart</h2>

* Clone this repository
* Open in VS Code
* Install extensions recommended by workspace
* Hit F5

This will build and run one of the example projects of [example/multiplatform] in Chrome

<h2 name="basic-guide">Basic Guide</h2>

* `cargo install cargo-container`
* Author a `Container.toml` workspace instead of a regular `Cargo.toml` workspace
    * Write a `[workspace]` like you would in `Cargo.toml`, with `members` (and optionally `exclude`)
    * Define one or more `[[build]]` sections defining what `crates` to wrap with what `tools`
    * <span style="opacity: 50%">Optional: specify more crates to auto-install via `[local-install]`</span>
* Author the crates to wrap in said boilerplate
* Run `cargo container build`.  This will:
    * Install any bin dependencies specified by `[local-install]`
    * Run `tools` to generate Cargo.toml projects
    * Generate a `Cargo.toml` alongside `Container.toml` that references the generated dependencies
    * Runs `tools` to build generated Cargo.toml projects
* Profit!



<h2 name="license">License</h2>

Licensed under either of

* Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
* MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.



<h2 name="contribution">Contribution</h2>

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.



[example/multiplatform]:        https://github.com/MaulingMonkey/cargo-container/tree/master/example/multiplatform
