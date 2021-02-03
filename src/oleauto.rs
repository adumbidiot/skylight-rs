/// A wrapper around a `BSTR` allocated with `SysAllocStringLen` or similar.
pub mod bstr;

pub use self::bstr::BStr;
pub use self::bstr::BStrRef;
