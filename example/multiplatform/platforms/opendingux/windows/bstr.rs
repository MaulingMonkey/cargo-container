use winapi::shared::wtypes::BSTR;
use winapi::um::oleauto::{SysAllocStringLen, SysFreeString, SysStringLen};

use std::convert::AsRef;
use std::borrow::Borrow;
use std::convert::TryInto;
use std::ffi::{OsStr, OsString};
use std::ops::Deref;
use std::os::windows::ffi::OsStrExt;



pub struct BString(BSTR);

impl BString {
    pub fn new(value: impl Into<Self>) -> Self { value.into() }

    /// # SAFETY
    ///
    /// * Assumes `bstr` is a valid `BSTR` (valid pointer, length prefixed)
    pub unsafe fn from_bstr(bstr: BSTR) -> Self {
        let len = SysStringLen(bstr);
        Self(SysAllocStringLen(bstr, len))
    }

    pub fn sys_alloc_string_len(olechar: &[u16]) -> Self {
        Self(unsafe { SysAllocStringLen(olechar.as_ptr(), olechar.len().try_into().unwrap()) })
    }

    pub fn as_ptr(&self) -> BSTR { self.0 }
    pub fn len(&self) -> usize { unsafe { SysStringLen(self.0) }.try_into().unwrap() }

    fn as_slice(&self) -> &[u16] { unsafe { std::slice::from_raw_parts(self.0, self.len()) } }

    pub fn to_os_string(&self) -> OsString { std::os::windows::ffi::OsStringExt::from_wide(self.as_slice()) }
}

impl AsRef<[u16]>   for BString { fn as_ref(&self) -> &[u16] { self.as_slice() } }
impl Borrow<[u16]>  for BString { fn borrow(&self) -> &[u16] { self.as_slice() } }
impl Deref          for BString { fn deref (&self) -> &[u16] { self.as_slice() } type Target = [u16]; }

impl std::ops::Drop for BString {
    fn drop(&mut self) {
        unsafe { SysFreeString(self.0) };
    }
}

impl From<&  str   > for BString { fn from(value: &  str   ) -> Self { let utf16 = value.encode_utf16().collect::<Vec<_>>(); Self::sys_alloc_string_len(&utf16) } }
impl From<   String> for BString { fn from(value:    String) -> Self { let utf16 = value.encode_utf16().collect::<Vec<_>>(); Self::sys_alloc_string_len(&utf16) } }
impl From<&  String> for BString { fn from(value: &  String) -> Self { let utf16 = value.encode_utf16().collect::<Vec<_>>(); Self::sys_alloc_string_len(&utf16) } }
impl From<&OsStr   > for BString { fn from(value: &OsStr   ) -> Self { let utf16 = value.encode_wide().collect::<Vec<_>>(); Self::sys_alloc_string_len(&utf16) } }
impl From< OsString> for BString { fn from(value:  OsString) -> Self { let utf16 = value.encode_wide().collect::<Vec<_>>(); Self::sys_alloc_string_len(&utf16) } }
impl From<&OsString> for BString { fn from(value: &OsString) -> Self { let utf16 = value.encode_wide().collect::<Vec<_>>(); Self::sys_alloc_string_len(&utf16) } }

impl From< BString> for OsString { fn from(value:  BString) -> Self { value.to_os_string() } }
impl From<&BString> for OsString { fn from(value: &BString) -> Self { value.to_os_string() } }

impl From<&BString> for BSTR {
    fn from(value: &BString) -> Self { value.0 }
}
