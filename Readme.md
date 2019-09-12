**Work in progress, only barely kinda partially usable, subject to breaking changes!**

# cargo-container

[![GitHub](https://img.shields.io/github/stars/MaulingMonkey/cargo-container.svg?label=GitHub&style=social)](https://github.com/MaulingMonkey/cargo-container)
[![Build Status](https://travis-ci.org/MaulingMonkey/cargo-container.svg)](https://travis-ci.org/MaulingMonkey/cargo-container)
![unsafe: forbid](https://img.shields.io/badge/unsafe-forbid-green.svg)
![rust: 1.36.0+](https://img.shields.io/badge/rust-1.36.0%2B-green.svg)
[![Open issues](https://img.shields.io/github/issues-raw/MaulingMonkey/cargo-container.svg)](https://github.com/MaulingMonkey/cargo-container/issues)
[![License](https://img.shields.io/crates/l/cargo-container.svg)](https://github.com/MaulingMonkey/cargo-container)
[![dependency status](https://deps.rs/repo/github/MaulingMonkey/cargo-container/status.svg)](https://deps.rs/repo/github/MaulingMonkey/cargo-container)

[Cargo containers](https://en.wikipedia.org/wiki/Intermodal_container) revolutionized the shipping industry through standardization.

`cargo-container` seeks to standardize build rules by wrapping `cargo` (and other) commands in an opinionated, zero/low-config, consistent manner.

# Quick Start

```cmd
cargo install cargo-container
:: And then something like...
cargo container build config=debug   platform=windows-desktop-x64
cargo container build config=release platform=browser-wasm32
cargo container build config=*       platform=*
cargo container build
cargo container test
:: ?
```

# License

Licensed under either of

* Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
* MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

# Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.

# Alternatives

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

# Project Scope

## v0.0.0

* Hardcode everything
* Windows desktop, browser, and maybe android support.
* [Always Be Shipping](https://blog.codinghorror.com/yes-but-what-have-you-done/)

## v0.5.0

* Extract configuration options
* Extract build logic into seperate crates that can be imported, perhaps by default
* Core `cargo-container` executable should be relatively minimal

## v1.0.0

By this point we should hopefully be supporting:
* `cargo web` for `stdweb` projects
* `wasm-pack` for `web_sys` projects
* Embedding [manifests](https://docs.microsoft.com/en-us/windows/win32/sysinfo/targeting-your-application-at-windows-8-1) in windows executables
* MSI installers?
* Zip packages?
* Dependencies to contribute assets
    * Amethyst?
    * Quicksilver?
    * Zip packages
* Code signing?
* Fuzz testing



[build.rs]:             https://doc.rust-lang.org/cargo/reference/build-scripts.html
[cargo dinghy]:         https://crates.io/crates/cargo-dinghy
[cargo apk]:            https://crates.io/crates/cargo-apk
[cargo ndk]:            https://crates.io/crates/cargo-ndk
[cargo web]:            https://crates.io/crates/cargo-web
[cargo make]:           https://crates.io/crates/cargo-make
[metabuild]:            https://doc.rust-lang.org/cargo/reference/unstable.html#metabuild
[rust-android-gradle]:  https://github.com/mozilla/rust-android-gradle
[wasm-pack]:            https://crates.io/crates/wasm-pack
