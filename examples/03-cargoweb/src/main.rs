fn main() {
    #[cfg(target_arch = "wasm32")] {
        use stdweb::console;
        console!(log, "Hello, world!");
    }
}
