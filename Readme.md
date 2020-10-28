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



<h2 name="alternatives">Alternatives</h2>

## Why not vanilla cargo / [build.rs] / [metabuild] scripts ?

* Cargo is intentionally *not* trying to support everything to keep things simple/sane.  Understandable, but crippling.
* Dependency builds are isolated even when you have use cases for them modifying the final output.
* No support for additional build rules after invoking rustc.

There's already a slew of nonstandard build tools for various specialized needs as a result:
* [cargo dinghy]
* [cargo apk]
* [cargo ndk]
* [cargo web]
* [cargo make]
* [rust-android-gradle]
* [wasm-pack]

These are generally non-composable, incoherent, require extra setup steps to install, etc.

## Why not `cargo run tools -- [...]` / `.cargo/config` aliases?

This is pretty neato.
* Intro:    https://matklad.github.io/2018/01/03/make-your-own-make.html
* Example:  https://github.com/rust-analyzer/rust-analyzer/blob/master/.cargo/config

I want something more zero-config/automatic/declarative for early projects though.  Kind of a [metabuild] equivalent.
I will steal as much inspiration from this as I can.

## Why not [cargo make] ?

It seems pretty great, but has a few drawbacks:
* It's unopinionated and lacks standardization - projects will be inconsistent
* Bring your own build rules
* No sane defaults for way too many project types
* Little-to-no support for creating reusable standard rules, unless you count hardcoding wget s into your own makefiles - which I don't.

## Why not \[non rust toolchain\] ?

* Packaging build rules in crates sounds neato.
* Auto-adding build rules based on dependencies sounds neato.



<!-- internal -->
[example/multiplatform]:        https://github.com/MaulingMonkey/cargo-container/tree/master/example/multiplatform

<!-- external -->
[build.rs]:                     https://doc.rust-lang.org/cargo/reference/build-scripts.html
[cargo dinghy]:                 https://crates.io/crates/cargo-dinghy
[cargo apk]:                    https://crates.io/crates/cargo-apk
[cargo ndk]:                    https://crates.io/crates/cargo-ndk
[cargo web]:                    https://crates.io/crates/cargo-web
[cargo make]:                   https://crates.io/crates/cargo-make
[metabuild]:                    https://doc.rust-lang.org/cargo/reference/unstable.html#metabuild
[rust-android-gradle]:          https://github.com/mozilla/rust-android-gradle
[wasm-pack]:                    https://crates.io/crates/wasm-pack
