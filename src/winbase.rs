use std::fmt::Write;
use std::mem::ManuallyDrop;
use std::mem::MaybeUninit;
use std::ptr::NonNull;
use std::{convert::TryInto, ffi::OsString, os::windows::ffi::OsStringExt};
use winapi::shared::lmcons::UNLEN;
use winapi::um::winbase::lstrlenW;
use winapi::um::winbase::GetUserNameW;
use winapi::um::winbase::LocalFree;

/// Get the user name of the current user.
///
/// # Errors
/// * Returns an error if the username could not be retrieved.
pub fn get_user_name() -> std::io::Result<OsString> {
    const BUFFER_LEN: u32 = UNLEN + 1;

    let mut buffer_len = BUFFER_LEN;
    let mut buffer = MaybeUninit::<[u16; BUFFER_LEN as usize]>::uninit();

    // # Safety
    // This is safe as the buffer exists and the correct buffer length is passed to this function for initialization.
    let ret = unsafe { GetUserNameW(buffer.as_mut_ptr().cast(), &mut buffer_len) };

    if ret == 0 {
        return Err(std::io::Error::last_os_error());
    }

    // # Safety
    // The data must be valid at this point.
    // The length of data (minus the nul terminator) has been updated and is passed in.
    // There are only immutable references left to `buffer`, so making another immutable one is safe.
    let buffer = unsafe {
        // -1 for the NUL terminator.
        let len = (buffer_len - 1) as usize;
        std::slice::from_raw_parts(buffer.as_ptr().cast(), len)
    };

    Ok(OsString::from_wide(buffer))
}

/// A Wide String that has been allocated with `LocalAlloc`.
#[repr(transparent)]
pub struct LocalWideString(NonNull<u16>);

impl LocalWideString {
    /// Make a [`LocalWideString`] from a ptr.
    ///
    /// # Safety
    /// ptr must be a valid LPWSTR allocated with `LocalAlloc`.
    pub unsafe fn from_raw(ptr: NonNull<u16>) -> Self {
        Self(ptr)
    }

    /// Get a mut ptr to the string
    pub fn as_mut_ptr(&mut self) -> *mut u16 {
        self.0.as_ptr()
    }

    /// Get the length of the string in characters.
    ///
    /// This is an O(n) operation.
    ///
    /// # Panics
    /// Panics if the length cannot fit in a `usize`.
    pub fn len(&self) -> usize {
        unsafe {
            lstrlenW(self.0.as_ptr())
                .try_into()
                .expect("len cannot fit in a `usize`")
        }
    }

    //// Check if this string is empty.
    ///
    /// This is an O(n) operation.
    ///
    /// # Panics
    /// Panics if the length cannot fit in a `usize`.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Get this string as a slice of u16s.
    ///
    /// This is an O(n) operation.
    ///
    /// # Panics
    /// Panics if the length cannot fit in a `usize`.
    pub fn as_slice(&self) -> &[u16] {
        unsafe { std::slice::from_raw_parts(self.0.as_ptr(), self.len()) }
    }

    /// Get this as an [`OsString`].
    ///
    /// # Panics
    /// Panics if the length cannot fit in a `usize`.
    pub fn as_os_string(&self) -> OsString {
        OsString::from_wide(self.as_slice())
    }

    /// Convert this to a [`String`].
    ///
    /// # Errors
    /// Returns an error if this contains invalid utf16
    pub fn to_str(&self) -> Result<String, std::string::FromUtf16Error> {
        String::from_utf16(self.as_slice())
    }

    /// Convert this to a [`String`] lossily.
    pub fn to_str_lossy(&self) -> String {
        String::from_utf16_lossy(self.as_slice())
    }

    /// Try to iterate over the chars in this string
    pub fn chars(&self) -> impl Iterator<Item = Result<char, std::char::DecodeUtf16Error>> + '_ {
        std::char::decode_utf16(self.as_slice().iter().copied())
    }

    /// Get a struct that implements display lossily for this string.
    pub fn display(&self) -> LocalWideStringDisplay {
        LocalWideStringDisplay(self)
    }

    /// Try to destroy this object.
    ///
    /// # Errors
    /// Returns a tuple of this object and an error if this object could not be destroyed.
    pub fn destroy(self) -> Result<(), (Self, std::io::Error)> {
        let mut obj = ManuallyDrop::new(self);
        let ret = unsafe { LocalFree(obj.as_mut_ptr().cast()) };

        if ret.is_null() {
            Ok(())
        } else {
            Err((
                ManuallyDrop::into_inner(obj),
                std::io::Error::last_os_error(),
            ))
        }
    }
}

impl Drop for LocalWideString {
    fn drop(&mut self) {
        std::mem::forget(Self(self.0).destroy());
    }
}

impl std::fmt::Debug for LocalWideString {
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

/// A struct that implements display for [`LocalWideString`]
pub struct LocalWideStringDisplay<'a>(&'a LocalWideString);

impl std::fmt::Display for LocalWideStringDisplay<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
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

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn get_user_name_works() {
        let user_name = get_user_name().unwrap();
        dbg!(user_name);
    }
}
