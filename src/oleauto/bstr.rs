use std::borrow::Borrow;
use std::borrow::Cow;
use std::convert::TryFrom;
use std::convert::TryInto;
use std::ffi::OsStr;
use std::ffi::OsString;
use std::fmt::Write;
use std::hash::Hash;
use std::hash::Hasher;
use std::mem::ManuallyDrop;
use std::num::TryFromIntError;
use std::ops::Deref;
use std::ops::DerefMut;
use std::os::windows::ffi::OsStrExt;
use std::os::windows::ffi::OsStringExt;
use std::str::FromStr;
use winapi::shared::wtypes::BSTR;
use winapi::um::oleauto::SysAllocStringLen;
use winapi::um::oleauto::SysFreeString;

/// An Error that may occur while creating a [`BStr`].
#[derive(Debug, PartialEq)]
pub enum BStrCreationError {
    /// Failed to convert the input length into a [`u32`]
    LenTooLarge(TryFromIntError),

    /// Failed to allocate a [`BStr`] using `SysAllocStringLen` or similar.
    AllocFailed,

    /// The iterator was too long. Use `take(len)` before passing the iter to the constructor.
    IterTooLarge,

    /// The iterator was too short. Make sure `len == iter.count()`.
    IterTooShort,
}

impl std::fmt::Display for BStrCreationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self {
            Self::LenTooLarge(e) => write!(f, "the length is too large ({})", e),
            Self::AllocFailed => "failed to allocate a bstr".fmt(f),
            Self::IterTooLarge => "the iterator provided too many elements".fmt(f),
            Self::IterTooShort => "the iterator provided too few elements".fmt(f),
        }
    }
}

impl std::error::Error for BStrCreationError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match &self {
            Self::LenTooLarge(e) => Some(e),
            _ => None,
        }
    }
}

/// A BStr allocated using `SysAllocString` or similar.
/// This type may or may not contain valid UTF16.
///
/// # Construction
/// It is suggested to use the `TryFrom` impls for constructing this struct.
/// To reduce the amount of generic code, some functions may use `BStr` directly in their parameters rather than something akin to `impl TryInto<BStr>`.
/// The user should use `BStr::new("data")` in these cases.
///
/// # Panics
/// The methods on the `Deref` impl [`BStrRef`] will panic if the [`BStr`]'s length cannot fit in a [`usize`].
///
#[repr(transparent)]
pub struct BStr(BSTR);

impl BStr {
    /// Make a new [`BStr`].
    /// This is a convenience function for using the `TryFrom` impls.
    /// Use [`BStr::try_new()`] for a non-panicking variant of this constructor.
    ///
    /// # Panics
    /// Panics if a new BStr could not be allocated, if the length cannot be stored in a [`u32`],
    /// or if the number of items in the iterator does not match the reported length (assuming an iterator was passed here).
    ///
    pub fn new<D>(data: D) -> Self
    where
        D: TryInto<BStr, Error = BStrCreationError>,
    {
        Self::try_new(data).expect("Valid BStr")
    }

    /// Make a new [`BStr`].
    /// This is a convenience function for using the `TryFrom` impls.
    /// Use [`BStr::new()`] for a panicking variant of this constructor.
    ///
    /// # Errors
    /// Returns a [`BStrCreationError`] if a new [`BStr`] could not be allocated, if the length cannot be stored in a [`u32`],
    /// or if the number of items in the iterator does not match the reported length (assuming an iterator was passed here).
    ///
    pub fn try_new<D>(data: D) -> Result<Self, BStrCreationError>
    where
        D: TryInto<BStr, Error = BStrCreationError>,
    {
        data.try_into()
    }

    /// Try to make a new [`BStr`] from a wide char iterator.
    ///
    /// # Errors
    /// Returns a [`BStrCreationError`] if a new [`BStr`] could not be allocated, if the length cannot be stored in a [`u32`],
    /// or if the number of items in the iterator does not match the reported length.
    ///
    pub fn from_wide_iter(
        iter: impl Iterator<Item = u16>,
        len: usize,
    ) -> Result<Self, BStrCreationError> {
        let len_u32 = len.try_into().map_err(BStrCreationError::LenTooLarge)?;
        let ptr = unsafe { SysAllocStringLen(std::ptr::null_mut(), len_u32) };

        if ptr.is_null() {
            return Err(BStrCreationError::AllocFailed);
        }

        unsafe {
            let mut str_len = 0;
            for (i, c) in iter.enumerate() {
                if i >= len {
                    // Dealloc ptr
                    drop(Self(ptr));
                    return Err(BStrCreationError::IterTooLarge);
                }

                ptr.add(i).write(c);

                str_len += 1;
            }

            if str_len != len {
                // Dealloc ptr
                drop(Self(ptr));
                return Err(BStrCreationError::IterTooShort);
            }
        }

        Ok(Self(ptr))
    }

    /// Try to make a new [`BStr`] from a wide char slice.
    ///
    /// # Errors
    /// Returns a `BStrCreationError` if a new [`BStr`] could not be allocated or if the length cannot be stored in a [`u32`].
    ///
    pub fn from_wide_slice(slice: &[u16]) -> Result<Self, BStrCreationError> {
        let len = slice
            .len()
            .try_into()
            .map_err(BStrCreationError::LenTooLarge)?;

        let ptr = unsafe { SysAllocStringLen(slice.as_ptr(), len) };

        if ptr.is_null() {
            Err(BStrCreationError::AllocFailed)
        } else {
            Ok(Self(ptr))
        }
    }

    /// Make a new [`BStr`] from a raw BSTR ptr.
    ///
    /// # Safety
    /// `ptr` must be a `BSTR` allocated with `SysAllocStringLen` or similar.
    ///
    /// # Panics
    /// Panics if `ptr` is null.
    ///
    pub unsafe fn from_raw(ptr: *mut u16) -> Self {
        assert!(!ptr.is_null());
        Self(ptr)
    }

    /// Leak this [`BStr`] and return the inner pointer.
    ///
    pub fn into_raw(self) -> *mut u16 {
        ManuallyDrop::new(self).0
    }

    /// Get this [`BStr`] as a `&BStrRef`.
    ///
    /// # Panics
    /// Panics if the size of the string cannot fit in a [`usize`].
    ///
    pub fn as_bstr_ref(&self) -> &BStrRef {
        unsafe { BStrRef::from_ptr(self.0) }
    }

    /// Get this [`BStr`] as a `&mut BStrRef`.
    ///
    /// # Panics
    /// Panics if the size of the string cannot fit in a [`usize`].
    ///
    pub fn as_mut_bstr_ref(&mut self) -> &mut BStrRef {
        unsafe { BStrRef::from_mut_ptr(self.0) }
    }
}

impl Deref for BStr {
    type Target = BStrRef;

    fn deref(&self) -> &Self::Target {
        self.as_bstr_ref()
    }
}

impl DerefMut for BStr {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.as_mut_bstr_ref()
    }
}

impl Drop for BStr {
    fn drop(&mut self) {
        unsafe {
            SysFreeString(self.0);
        }
    }
}

/// A reference to a `BSTR`.
//
#[repr(transparent)]
pub struct BStrRef {
    inner: [u16],
}

impl BStrRef {
    /// Make a &[`BStrRef`] from a `BSTR` ptr.
    ///
    /// # Safety
    /// 1. `ptr` must point to a valid `BSTR`.
    /// 2. `ptr` must not be null.
    /// 3. The reference must not outlive the pointer.
    /// 4. The number of bytes in the `BSTR` must be divisible by 2 (the`BSTR` must be a wide char `BSTR`, not a byte string).
    ///
    /// # Panics
    /// Panics if len cannot fit in a [`usize`]
    ///
    pub unsafe fn from_ptr<'a>(ptr: *const u16) -> &'a Self {
        debug_assert!(!ptr.is_null());

        let len: usize = (*ptr.cast::<u32>().sub(1))
            .try_into()
            .expect("len can fit in a usize");

        debug_assert!(len % 2 == 0);

        let ptr: *const [u16] = std::slice::from_raw_parts(ptr, (len / 2) + 1);

        &*(ptr as *const Self)
    }

    /// Make a &mut [`BStrRef`] from a `BSTR` ptr.
    ///
    /// # Safety
    /// 1. `ptr` must point to a valid `BSTR`.
    /// 2. `ptr` must not be null.
    /// 3. `ptr` must be mutable.
    /// 4. The reference must not outlive the pointer.
    /// 5. The number of bytes in the `BSTR` must be divisible by 2 (the`BSTR` must be a wide char `BSTR`, not a byte string).
    ///
    /// # Panics
    /// Panics if len cannot fit in a [`usize`].
    ///
    pub unsafe fn from_mut_ptr<'a>(ptr: *mut u16) -> &'a mut Self {
        debug_assert!(!ptr.is_null());

        let len: usize = (*ptr.cast::<u32>().sub(1))
            .try_into()
            .expect("len can fit in a usize");

        debug_assert!(len % 2 == 0);

        let ptr: *mut [u16] = std::slice::from_raw_parts_mut(ptr, (len / 2) + 1);

        &mut *(ptr as *mut Self)
    }

    /// Get a `const` ptr to the data.
    /// This is guaranteed to be non-null.
    ///
    pub fn as_ptr(&self) -> *const u16 {
        self.inner.as_ptr()
    }

    /// Get a `mut` ptr to the data.
    /// This is guaranteed to be non-null.
    ///
    pub fn as_mut_ptr(&mut self) -> *mut u16 {
        self.inner.as_mut_ptr()
    }

    /// Get the len of this [`BStrRef`] in bytes.
    /// Note that chars take up twice the room they are stored as UTF16, so `BStr::new("Test").len() != "Test".len()`.
    ///
    pub fn len(&self) -> usize {
        (self.inner.len() * 2) - 2
    }

    /// Checks if this [`BStrRef`] is empty.
    ///
    /// # Panics
    /// Panics if the size of the string cannot fit in a [`usize`].
    ///
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Get this [`BStrRef`] as a wide char slice.
    /// This WILL NOT include the terminating NUL byte.
    /// This MAY or MAY NOT include interior NUL bytes.
    ///
    pub fn as_wide_slice(&self) -> &[u16] {
        &self.inner[..self.inner.len() - 1]
    }

    /// Get this [`BStrRef`] as a wide char slice.
    /// This WILL include terminating NUL byte.
    /// This MAY or MAY NOT include interior NUL bytes.
    /// This is NOT suitable for use as a wide CString unless [`BStrRef::contains_nul`] is false, proving that this [`BStrRef`] does not have any interior NULs.
    ///
    pub fn as_wide_slice_with_nul(&self) -> &[u16] {
        &self.inner
    }

    /// Returns true if this [`BStrRef`] contains any interior NULs.
    ///
    pub fn contains_nul(&self) -> bool {
        self.as_wide_slice().iter().any(|el| *el == 0)
    }

    /// Gets this [`BStrRef`] as an [`OsString`].
    /// This will allocate a new [`OsString`], so the result should be cached.
    ///
    pub fn to_os_string(&self) -> OsString {
        OsString::from_wide(self.as_wide_slice())
    }

    /// Converts this [`BStr`] to a [`String`] lossily. This allocates a new [`String`].
    /// This is just a convenience function for [`String::from_utf16_lossy`]
    ///
    pub fn to_string_lossy(&self) -> String {
        String::from_utf16_lossy(self.as_wide_slice())
    }

    /// Make a new [`BStrDisplay`] from this reference.
    /// [`BStrDisplay`] has a lossy display impl.
    ///
    pub fn display(&self) -> BStrDisplay {
        BStrDisplay(self)
    }

    /// Try to iterate over the chars in this string.
    ///
    pub fn chars(&self) -> impl Iterator<Item = Result<char, std::char::DecodeUtf16Error>> + '_ {
        std::char::decode_utf16(self.as_wide_slice().iter().copied())
    }
}

impl std::fmt::Debug for BStrRef {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_char('"')?;
        for c in self
            .chars()
            .map(|r| r.unwrap_or(std::char::REPLACEMENT_CHARACTER))
        {
            for c in c.escape_debug() {
                f.write_char(c)?
            }
        }

        f.write_char('"')?;

        Ok(())
    }
}

impl TryFrom<&str> for BStr {
    type Error = BStrCreationError;

    /// This delegates to the `TryFrom<&OsStr>` impl.
    fn try_from(data: &str) -> Result<Self, Self::Error> {
        Self::try_from(OsStr::new(data))
    }
}

impl TryFrom<&OsStr> for BStr {
    type Error = BStrCreationError;

    /// An upfront length calc is needed to allocate the correct size of the underlyting buffer.
    /// If this is undesirable, allocate to a [`Vec`] and use the `TryFrom<&[u16]>` impl.
    fn try_from(data: &OsStr) -> Result<Self, Self::Error> {
        let len = data.encode_wide().count();
        Self::from_wide_iter(data.encode_wide(), len)
    }
}

impl TryFrom<&[u16]> for BStr {
    type Error = BStrCreationError;

    fn try_from(data: &[u16]) -> Result<Self, Self::Error> {
        Self::from_wide_slice(data)
    }
}

impl TryFrom<&BStrRef> for BStr {
    type Error = BStrCreationError;

    fn try_from(data: &BStrRef) -> Result<Self, Self::Error> {
        Self::from_wide_slice(data.as_wide_slice())
    }
}

impl FromStr for BStr {
    type Err = BStrCreationError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::try_from(s)
    }
}

impl std::fmt::Debug for BStr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.as_bstr_ref().fmt(f)
    }
}

impl PartialEq<BStr> for BStr {
    fn eq(&self, other: &Self) -> bool {
        self.as_bstr_ref().eq(other.as_bstr_ref())
    }
}

impl PartialEq<BStrRef> for BStr {
    fn eq(&self, other: &BStrRef) -> bool {
        self.as_bstr_ref().eq(other)
    }
}

impl PartialEq<OsStr> for BStr {
    fn eq(&self, other: &OsStr) -> bool {
        self.as_bstr_ref().eq(other)
    }
}

impl PartialEq<&OsStr> for BStr {
    fn eq(&self, other: &&OsStr) -> bool {
        self.as_bstr_ref().eq(other)
    }
}

impl PartialEq<str> for BStr {
    fn eq(&self, other: &str) -> bool {
        self.as_bstr_ref().eq(other)
    }
}

impl PartialEq<&str> for BStr {
    fn eq(&self, other: &&str) -> bool {
        self.as_bstr_ref().eq(other)
    }
}

impl Eq for BStr {}

impl Hash for BStr {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.as_bstr_ref().hash(state)
    }
}

impl Borrow<BStrRef> for BStr {
    fn borrow(&self) -> &BStrRef {
        self
    }
}

/// A [`BStrRef`] wrapper that implments `Display`.
/// This forwars the `Debug` impl to the underlying [`BStrRef`],
/// while lossily displaying the string for `Display`.
///
pub struct BStrDisplay<'a>(&'a BStrRef);

impl<'a> std::fmt::Debug for BStrDisplay<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl<'a> std::fmt::Display for BStrDisplay<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for c in self
            .0
            .chars()
            .map(|r| r.unwrap_or(std::char::REPLACEMENT_CHARACTER))
        {
            f.write_char(c)?
        }

        Ok(())
    }
}

impl Hash for BStrRef {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.as_wide_slice_with_nul().hash(state)
    }
}

impl PartialEq<BStr> for BStrRef {
    fn eq(&self, other: &BStr) -> bool {
        self.eq(other.as_bstr_ref())
    }
}

impl<'a> PartialEq<BStr> for &'a BStrRef {
    fn eq(&self, other: &BStr) -> bool {
        self.eq(other.as_bstr_ref())
    }
}

impl PartialEq<BStrRef> for BStrRef {
    fn eq(&self, other: &Self) -> bool {
        self.as_wide_slice().eq(other.as_wide_slice())
    }
}

impl<'a> PartialEq<BStrRef> for &'a BStrRef {
    fn eq(&self, other: &BStrRef) -> bool {
        self.as_wide_slice().eq(other.as_wide_slice())
    }
}

impl PartialEq<OsStr> for BStrRef {
    fn eq(&self, other: &OsStr) -> bool {
        self.as_wide_slice().iter().copied().eq(other.encode_wide())
    }
}

impl PartialEq<&OsStr> for BStrRef {
    fn eq(&self, other: &&OsStr) -> bool {
        self.eq(*other)
    }
}

impl PartialEq<str> for BStrRef {
    fn eq(&self, other: &str) -> bool {
        self.eq(OsStr::new(other))
    }
}

impl PartialEq<&str> for BStrRef {
    fn eq(&self, other: &&str) -> bool {
        self.eq(OsStr::new(other))
    }
}

impl Eq for BStrRef {}

impl ToOwned for BStrRef {
    type Owned = BStr;

    fn to_owned(&self) -> <Self as ToOwned>::Owned {
        BStr::new(self)
    }
}

impl From<BStr> for Cow<'_, BStrRef> {
    fn from(data: BStr) -> Self {
        Cow::Owned(data)
    }
}

impl<'a> From<&'a BStrRef> for Cow<'a, BStrRef> {
    fn from(data: &'a BStrRef) -> Self {
        Cow::Borrowed(data)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn smoke() {
        let phrase = "The quick brown fox jumps over the lazy dog";
        let bstr_phrase = BStr::new(phrase);
        assert_eq!(phrase.len() * 2, bstr_phrase.len());
        assert!(bstr_phrase
            .as_wide_slice()
            .iter()
            .copied()
            .eq(OsStr::new(phrase).encode_wide()));
        assert!(bstr_phrase
            .as_wide_slice_with_nul()
            .iter()
            .copied()
            .eq(OsStr::new(phrase).encode_wide().chain(std::iter::once(0))));
        assert_eq!(bstr_phrase.to_os_string(), phrase);
        assert!(!bstr_phrase.contains_nul());

        let phrase_debug = format!("{:#?}", phrase);
        let bstr_phrase_debug = format!("{:#?}", bstr_phrase);
        assert_eq!(phrase_debug, bstr_phrase_debug);
    }

    #[test]
    fn interior_nul_smoke() {
        let phrase = "hello\0world!";
        let bstr_phrase = BStr::new(phrase);

        assert_eq!(phrase.len() * 2, bstr_phrase.len());
        assert!(bstr_phrase
            .as_wide_slice()
            .iter()
            .copied()
            .eq(OsStr::new(phrase).encode_wide()));
        assert!(bstr_phrase
            .as_wide_slice_with_nul()
            .iter()
            .copied()
            .eq(OsStr::new(phrase).encode_wide().chain(std::iter::once(0))));
        assert_eq!(bstr_phrase.to_os_string(), phrase);
        assert!(bstr_phrase.contains_nul());

        let phrase_debug = format!("{:#?}", phrase);
        let bstr_phrase_debug = format!("{:#?}", bstr_phrase);
        assert_eq!(phrase_debug, bstr_phrase_debug);
    }

    #[test]
    fn empty_bstr() {
        let s = BStr::new("");
        assert!(s.is_empty());
        assert_eq!(s.len(), 0);
        assert_eq!(s.as_wide_slice(), &[]);
    }

    #[test]
    fn clone_bstr_ref() {
        let s = BStr::new("Hello World!");
        let r = s.as_bstr_ref();
        let r1 = <&BStrRef>::clone(&r);

        assert_eq!(r, r1);
    }

    #[test]
    fn bstr_to_owned() {
        let s = BStr::new("Hello World!");
        let r = s.as_bstr_ref();
        let s1 = r.to_owned();

        assert_eq!(s1, s);
    }

    #[test]
    fn borrow_bstr() {
        let s = BStr::new("Hello World!");
        let b = s.borrow();
        assert_eq!(b, s);
    }

    #[test]
    fn cow_bstr() {
        let owned_cow_bstr: Cow<BStrRef> = BStr::new("data").into();
        let borrowed_cow_bstr: Cow<BStrRef> = owned_cow_bstr.as_ref().into();

        assert_eq!(owned_cow_bstr, borrowed_cow_bstr);
    }
}
