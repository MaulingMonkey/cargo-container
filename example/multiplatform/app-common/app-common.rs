pub trait DialogProvider {
    fn show_dialog(&self, text: &str);
}



#[cfg(feature = "platform-console")]
pub struct ConsoleDialogProvider;

#[cfg(feature = "platform-console")]
impl DialogProvider for ConsoleDialogProvider {
    fn show_dialog(&self, text: &str) {
        println!("{}", text);
        std::io::stdin().read_line(&mut String::new()).unwrap();
    }
}



#[cfg(feature = "platform-windows")]
pub struct WindowsDialogProvider;

#[cfg(feature = "platform-windows")]
impl DialogProvider for WindowsDialogProvider {
    fn show_dialog(&self, text: &str) {
        use winapi::um::winuser::*;
        use std::ptr::*;
        use std::io;

        let caption = "cargo-container example";

        let text    = text      .encode_utf16().chain(Some(0)).collect::<Vec<_>>();
        let caption = caption   .encode_utf16().chain(Some(0)).collect::<Vec<_>>();
        let r = unsafe { MessageBoxW(null_mut(), text.as_ptr(), caption.as_ptr(), MB_OK) };
        assert!(r != 0, "MessageBoxW failed: {}", io::Error::last_os_error());
    }
}



#[cfg(feature = "platform-stdweb")]
pub struct StdwebDialogProvider;

#[cfg(feature = "platform-stdweb")]
impl DialogProvider for StdwebDialogProvider {
    fn show_dialog(&self, text: &str) {
        stdweb::web::alert(text);
    }
}



#[cfg(feature = "platform-web-sys")]
pub use wasm_bindgen;

#[cfg(feature = "platform-web-sys")]
pub struct WebSysDialogProvider;

#[cfg(feature = "platform-web-sys")]
impl DialogProvider for WebSysDialogProvider {
    fn show_dialog(&self, text: &str) {
        let window = web_sys::window().unwrap();
        window.alert_with_message(text).unwrap();
    }
}
