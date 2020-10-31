#![allow(dead_code)]

// https://en.wikipedia.org/wiki/Variant_type

use super::BString;
use crate::mmrbi::fatal;

use winapi::shared::winerror::SUCCEEDED;
use winapi::shared::wtypes::{self, VARENUM, VARTYPE};
use winapi::um::oaidl::VARIANT;
use winapi::um::oleauto::VariantClear;

use std::convert::{TryFrom, TryInto};
use std::ffi::OsString;



pub struct Variant(VARIANT);

impl Variant {
    pub fn new() -> Self { Self(VARIANT::default()) }
    pub fn vt(&self) -> VARTYPE { unsafe { self.0.n1.n2().vt } }
    fn ve(&self) -> VARENUM { self.vt() as _ }

    pub fn as_ptr_mut(&mut self) -> *mut VARIANT { &mut self.0 }

    pub fn to_os_string(&self) -> Option<OsString> {
        match self.ve() {
            wtypes::VT_BSTR         => Some(unsafe { BString::from_bstr(*self.0.n1.n2().n3.bstrVal()) }.into()),
            wtypes::VT_LPSTR        => None, // XXX: only used by propvariant and not by VARIANT?
            wtypes::VT_LPWSTR       => None, // XXX: only used by propvariant and not by VARIANT?
            wtypes::VT_BSTR_BLOB    => None, // "Reserved"
            _other                  => None,
        }
    }

    //pub fn as_i8 (&self) -> Option<i8 > { if self.ve() == wtypes::VT_I1 { unsafe { Some(*self.0.n1.n2().n3.cVal()) } } else { None } }
    //pub fn as_i16(&self) -> Option<i16> { if self.ve() == wtypes::VT_I2 { unsafe { Some(*self.0.n1.n2().n3.iVal()) } } else { None } }
    //pub fn as_i32(&self) -> Option<i32> { if self.ve() == wtypes::VT_I4 { unsafe { Some(*self.0.n1.n2().n3.lVal()) } } else { None } }
    //pub fn as_i64(&self) -> Option<i64> { if self.ve() == wtypes::VT_I8 { unsafe { Some(*self.0.n1.n2().n3.llVal()) } } else { None } }
    //pub fn as_u8 (&self) -> Option<u8 > { if self.ve() == wtypes::VT_UI1 { unsafe { Some(*self.0.n1.n2().n3.bVal()) } } else { None } }
    //pub fn as_u16(&self) -> Option<u16> { if self.ve() == wtypes::VT_UI2 { unsafe { Some(*self.0.n1.n2().n3.uiVal()) } } else { None } }
    //pub fn as_u32(&self) -> Option<u32> { if self.ve() == wtypes::VT_UI4 { unsafe { Some(*self.0.n1.n2().n3.ulVal()) } } else { None } }
    //pub fn as_u64(&self) -> Option<u64> { if self.ve() == wtypes::VT_UI8 { unsafe { Some(*self.0.n1.n2().n3.ullVal()) } } else { None } }

    fn try_into_int<T: TryFrom<u64> + TryFrom<i64>>(&self) -> Option<T> {
        match self.ve() {
            wtypes::VT_I1   => i64::from(unsafe { *self.0.n1.n2().n3.cVal() }).try_into().ok(),
            wtypes::VT_I2   => i64::from(unsafe { *self.0.n1.n2().n3.iVal() }).try_into().ok(),
            wtypes::VT_I4   => i64::from(unsafe { *self.0.n1.n2().n3.lVal() }).try_into().ok(),
            wtypes::VT_I8   => i64::from(unsafe { *self.0.n1.n2().n3.llVal() }).try_into().ok(),
            wtypes::VT_INT  => i64::from(unsafe { *self.0.n1.n2().n3.intVal() }).try_into().ok(),

            wtypes::VT_UI1  => u64::from(unsafe { *self.0.n1.n2().n3.bVal() }).try_into().ok(),
            wtypes::VT_UI2  => u64::from(unsafe { *self.0.n1.n2().n3.uiVal() }).try_into().ok(),
            wtypes::VT_UI4  => u64::from(unsafe { *self.0.n1.n2().n3.ulVal() }).try_into().ok(),
            wtypes::VT_UI8  => u64::from(unsafe { *self.0.n1.n2().n3.ullVal() }).try_into().ok(),
            wtypes::VT_UINT => i64::from(unsafe { *self.0.n1.n2().n3.intVal() }).try_into().ok(),

            _other => None,
        }
    }
}

impl std::ops::Drop for Variant {
    fn drop(&mut self) {
        let hr = unsafe { VariantClear(&mut self.0) };
        if !SUCCEEDED(hr) { fatal!("VariantClear(&...) failed with HRESULT 0x{:08x}", hr) }
    }
}

impl std::ops::Deref for Variant {
    type Target = VARIANT;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl TryFrom<&Variant> for i8  { type Error = (); fn try_from(v: &Variant) -> Result<Self, ()> { v.try_into_int::<Self>().ok_or(()) } }
impl TryFrom<&Variant> for i16 { type Error = (); fn try_from(v: &Variant) -> Result<Self, ()> { v.try_into_int::<Self>().ok_or(()) } }
impl TryFrom<&Variant> for i32 { type Error = (); fn try_from(v: &Variant) -> Result<Self, ()> { v.try_into_int::<Self>().ok_or(()) } }
impl TryFrom<&Variant> for i64 { type Error = (); fn try_from(v: &Variant) -> Result<Self, ()> { v.try_into_int::<Self>().ok_or(()) } }

impl TryFrom<&Variant> for u8  { type Error = (); fn try_from(v: &Variant) -> Result<Self, ()> { v.try_into_int::<Self>().ok_or(()) } }
impl TryFrom<&Variant> for u16 { type Error = (); fn try_from(v: &Variant) -> Result<Self, ()> { v.try_into_int::<Self>().ok_or(()) } }
impl TryFrom<&Variant> for u32 { type Error = (); fn try_from(v: &Variant) -> Result<Self, ()> { v.try_into_int::<Self>().ok_or(()) } }
impl TryFrom<&Variant> for u64 { type Error = (); fn try_from(v: &Variant) -> Result<Self, ()> { v.try_into_int::<Self>().ok_or(()) } }

impl TryFrom< Variant> for i8  { type Error = (); fn try_from(v:  Variant) -> Result<Self, ()> { v.try_into_int::<Self>().ok_or(()) } }
impl TryFrom< Variant> for i16 { type Error = (); fn try_from(v:  Variant) -> Result<Self, ()> { v.try_into_int::<Self>().ok_or(()) } }
impl TryFrom< Variant> for i32 { type Error = (); fn try_from(v:  Variant) -> Result<Self, ()> { v.try_into_int::<Self>().ok_or(()) } }
impl TryFrom< Variant> for i64 { type Error = (); fn try_from(v:  Variant) -> Result<Self, ()> { v.try_into_int::<Self>().ok_or(()) } }

impl TryFrom< Variant> for u8  { type Error = (); fn try_from(v:  Variant) -> Result<Self, ()> { v.try_into_int::<Self>().ok_or(()) } }
impl TryFrom< Variant> for u16 { type Error = (); fn try_from(v:  Variant) -> Result<Self, ()> { v.try_into_int::<Self>().ok_or(()) } }
impl TryFrom< Variant> for u32 { type Error = (); fn try_from(v:  Variant) -> Result<Self, ()> { v.try_into_int::<Self>().ok_or(()) } }
impl TryFrom< Variant> for u64 { type Error = (); fn try_from(v:  Variant) -> Result<Self, ()> { v.try_into_int::<Self>().ok_or(()) } }
