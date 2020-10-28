# Example: Multiplatform

This demonstrates using `cargo container` to wrap `apps` in multiple `platforms`,
displaying a "dialog".



<h2 name="quickstart">Quickstart</h2>

```cmd
cargo install --path .

cd example\multiplatform
cargo container build
cargo container test

:: Run the debug platforms\console generated executable
target\debug\alpha.exe

:: Run the debug platforms\windows generated executable
target\x86_64-pc-windows-msvc\debug\alpha.exe
```



<h2 name="overview">Overview</h2>

| Source File       | Description                               |
| ----------------- | ----------------------------------------- |
| `app-common/`     | Multiplatform library consumed by apps
| `apps/*/`         | Example crates to wrap
| `platforms/*/`    | Example generators of platform specific boilerplate
| `Container.toml`  | "Workspace" defining what to build



<h2 name="apps">Apps</h2>

In this example, each "app" (alpha, beta, delta) exposes:
```rust
pub fn init(ctx: impl app_common::DialogProvider) { ... }
```

There is nothing special about this function signature - it only needs to match
whatever the accompanying generated packages expect.  This might be pub fns,
this might be [inventory](https://docs.rs/inventory/)-registered flags... anything!



<h2 name="platforms">Platforms</h2>

Each of these generates Cargo.toml `[package]`s wrapping the aforementioned apps.

### console

Displays the 'dialog' via stdout, waits for response via stdin

```rust
fn main() { app::init(app_common::ConsoleDialogProvider) }
```

### stdweb

Displays the dialog as a javascript alert in the browser, via [stdweb](https://docs.rs/stdweb/).

```rust
fn main() { app::init(app_common::StdWebDialogProvider) }
```

### web-sys

Displays the dialog as a javascript alert in the browser, via [web-sys](https://docs.rs/web-sys/).

```rust
#[wasm_bindgen(start)] pub fn start() { app::init(app_common::WebSysDialogProvider) }
```

### windows

Displays the dialog as a message box.  Will cross-compile to windows on non-windows platforms.

```rust
#![windows_subsystem="windows"] fn main() { app::init(app_common::WindowsDialogProvider) }
```



<h2 name="generated-files">Generated Files</h2>

| Generated File                                                    | Config | Platform     | Host      | Target |
| ----------------------------------------------------------------- | ------ | ------------ | --------- | ------ |
| `Cargo.lock`                                                      | \*    | \*            | \*        | \*
| `Cargo.toml`                                                      | \*    | \*            | \*        | \*
| `target/debug/alpha.exe`                                          | debug | console       | windows   | windows
| `target/debug/alpha`                                              | debug | console       | linux     | linux
| `target/x86_64-pc-windows-msvc/debug/alpha.exe`                   | debug | windows       | windows   | windows
| `target/x86_64-pc-windows-gnu/debug/alpha.exe`                    | debug | windows       | linux     | **windows**
| `target/wasm32-unknown-unknown/debug/alpha-stdweb.html`           | debug | stdweb        | \*        | browser
| `target/wasm32-unknown-unknown/debug/alpha-web-sys/index.html`    | debug | web-sys       | \*        | browser
| `target/release/...`                                              | release
| `target/*/release/...`                                            | release



<h2 name="advanced-ideas">advanced-ideas</h2>

In this demo, I abuse traits for monomorphization.
Perhaps instead you'd prefer global `Mutex<Box<dyn Whatever>>`s?
This *can* lead to issues if `app` uses a different version of `app_common`.
(e.g. `reqwest` might use `tokio 0.2` and explode at runtime because you've only initialized a `tokio 0.3` runtime)

```rust
fn main() {
    let dp : Box<dyn DialogProvider> = Box::new(ConsoleDialogProvider);
    app_common::set_dialog_provider(dp);
    app::init();
}
```

Alternatively, maybe you can make `app_common` entirely self configuring:

```rust
fn main() {
    app::init();
}
```

Multiple entry points for different frontends is also possible:

```rust
// Different platform feature-sets?
pub fn init_console(ctx: impl GamepadInput) { ... }
pub fn init_desktop(ctx: impl GamepadInput + KeyboardInput + MouseInput) { ... }

// Different contexts
pub fn render_2d(ctx: impl RenderContext2D) { ... }
pub fn render_3d(ctx: impl RenderContext3D) { ... }
```
