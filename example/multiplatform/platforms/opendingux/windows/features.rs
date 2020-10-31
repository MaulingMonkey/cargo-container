use crate::fatal;
use super::*;

use std::collections::BTreeMap;
use std::ffi::{OsStr, OsString};



pub type Map = BTreeMap<OsString, InstallState>;

#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum InstallState {
    Enabled     = 1,
    Disabled    = 2,
    Absent      = 3,
    Unknown     = 4,
}

#[cfg(not(windows))]
pub fn get() -> Map {
    Default::default()
}

#[cfg(windows)]
pub fn get() -> Map {
    // Need MTA COM, current thread could be STA so spin up a new one
    std::thread::spawn(||{
        use winapi::shared::rpcdce::{RPC_C_AUTHN_LEVEL_CALL, RPC_C_AUTHN_LEVEL_DEFAULT, RPC_C_AUTHN_WINNT, RPC_C_AUTHZ_NONE, RPC_C_IMP_LEVEL_IMPERSONATE};
        use winapi::shared::winerror::SUCCEEDED;
        use winapi::shared::wtypesbase::CLSCTX_INPROC_SERVER;
        use winapi::um::combaseapi::{CoCreateInstance, CoInitializeEx, CoInitializeSecurity, CoSetProxyBlanket};
        use winapi::um::objbase::COINIT_MULTITHREADED;
        use winapi::um::objidlbase::EOAC_NONE;
        use winapi::um::wbemcli::*;

        use wio::com::ComPtr;

        use std::convert::TryFrom;
        use std::ptr::*;

        // https://docs.microsoft.com/en-us/windows/win32/wmisdk/initializing-com-for-a-wmi-application

        let hr = unsafe { CoInitializeEx(null_mut(), COINIT_MULTITHREADED) };
        if !SUCCEEDED(hr) { fatal!("CoInitializeEx(0, COINIT_MULTITHREADED) failed with HRESULT 0x{:08x}", hr); }

        let hr = unsafe { CoInitializeSecurity(null_mut(), -1, null_mut(), null_mut(), RPC_C_AUTHN_LEVEL_DEFAULT, RPC_C_IMP_LEVEL_IMPERSONATE, null_mut(), EOAC_NONE, null_mut()) };
        if !SUCCEEDED(hr) { fatal!("CoInitializeSecurity(...) failed with HRESULT 0x{:08x}", hr); }

        // https://docs.microsoft.com/en-us/windows/win32/wmisdk/creating-a-connection-to-a-wmi-namespace

        let mut locator = null_mut();
        let hr = unsafe { CoCreateInstance(&CLSID_WbemLocator, null_mut(), CLSCTX_INPROC_SERVER, &IID_IWbemLocator, &mut locator) };
        if !SUCCEEDED(hr) { fatal!("CoCreateInstance(CLSID_WbemLocator, ...) failed with HRESULT 0x{:08x}", hr); }
        let locator = unsafe { ComPtr::<IWbemLocator>::from_raw(locator.cast()) };

        let namespace = BString::new("ROOT\\CIMV2");

        let mut services = null_mut();
        let hr = unsafe { locator.ConnectServer(namespace.as_ptr(), null_mut(), null_mut(), null_mut(), 0, null_mut(), null_mut(), &mut services) };
        if !SUCCEEDED(hr) { fatal!("IWbemLocator::ConnectServer(...) failed with HRESULT 0x{:08x}", hr) }
        let services = unsafe { ComPtr::<IWbemServices>::from_raw(services.cast()) };

        // https://docs.microsoft.com/en-us/windows/win32/wmisdk/setting-the-security-levels-on-a-wmi-connection

        let hr = unsafe { CoSetProxyBlanket(services.as_raw().cast(), RPC_C_AUTHN_WINNT, RPC_C_AUTHZ_NONE, null_mut(), RPC_C_AUTHN_LEVEL_CALL, RPC_C_IMP_LEVEL_IMPERSONATE, null_mut(), EOAC_NONE) };
        if !SUCCEEDED(hr) { fatal!("CoSetProxyBlanket(...) failed with HRESULT 0x{:08x}", hr) }

        // https://docs.microsoft.com/en-us/windows/win32/wmisdk/invoking-a-synchronous-query

        let language = BString::new("WQL");
        let query = BString::new("SELECT * From Win32_OptionalFeature");

        let mut qenum = null_mut();
        let hr = unsafe { services.ExecQuery(language.as_ptr(), query.as_ptr(), (WBEM_FLAG_FORWARD_ONLY | WBEM_FLAG_DIRECT_READ) as _, null_mut(), &mut qenum) };
        if !SUCCEEDED(hr) { fatal!("IWbemServices::ExecQuery(...) failed with HRESULT 0x{:08x}", hr) }
        let qenum = unsafe { ComPtr::<IEnumWbemClassObject>::from_raw(qenum) };

        let key_name            = BString::new("Name");
        let key_install_state   = BString::new("InstallState");

        let mut o = BTreeMap::new();

        loop {
            let mut obj = null_mut();
            let mut ret = 0;
            let hr = unsafe { qenum.Next(0, 1, &mut obj, &mut ret) };
            if !SUCCEEDED(hr) { fatal!("IEnumWbemClassObject::Next(...) failed with HRESULT 0x{:08x}", hr) }
            if ret == 0 { break }
            let obj = unsafe { ComPtr::<IWbemClassObject>::from_raw(obj) };

            let mut name            = Variant::new();
            let mut install_state   = Variant::new();

            let hr = unsafe { obj.Get(key_name.as_ptr(), 0, name.as_ptr_mut(), null_mut(), null_mut()) };
            if !SUCCEEDED(hr) { fatal!("IWbemClassObject::Get(BSTR(\"Name\", ...) failed with HRESULT 0x{:08x}", hr) }
            let hr = unsafe { obj.Get(key_install_state.as_ptr(), 0, install_state.as_ptr_mut(), null_mut(), null_mut()) };
            if !SUCCEEDED(hr) { fatal!("IWbemClassObject::Get(BSTR(\"InstallState\", ...) failed with HRESULT 0x{:08x}", hr) }

            let name = name.to_os_string().unwrap();
            let install_state = match u8::try_from(&install_state) {
                Ok(1)   => InstallState::Enabled,
                Ok(2)   => InstallState::Disabled,
                Ok(3)   => InstallState::Absent,
                Ok(4)|_ => InstallState::Unknown,
            };
            o.insert(name, install_state);
        }

        o
    }).join().unwrap()
}

pub fn require(feature: &str) {
    if cfg!(windows) {
        let state = get().get(OsStr::new(feature)).copied().unwrap_or(InstallState::Unknown);
        if state != InstallState::Enabled {
            println!("cargo-container:sudo=dism /online /enable-feature /featureName:{} /all /norestart", feature);
        }
    }
}
