use std::fmt::Write;
use std::mem::ManuallyDrop;
use std::mem::MaybeUninit;
use std::{convert::TryInto, ffi::OsString, os::windows::ffi::OsStringExt};
use winapi::shared::lmcons::UNLEN;
use winapi::um::winbase::GetUserNameW;
use winapi::um::{
    winbase::{lstrlenW, LocalFree},
    winnt::LPWSTR,
};

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
pub struct LocalWideString(LPWSTR);

impl LocalWideString {
    /// Make a [`LocalWideString`] from a ptr.
    ///
    /// # Safety
    /// s must be a valid LPWSTR
    ///
    /// # Panics
    /// Panics if ptr is null.
    pub unsafe fn from_lpwstr(ptr: LPWSTR) -> Self {
        Self::try_from_lpwstr(ptr).expect("ptr is null")
    }

    /// Try to make a [`LocalWideString`] from a ptr.
    ///
    /// # Safety
    /// s must be a valid LPWSTR
    ///
    /// # Errors
    /// Returns `None` if ptr is null.
    pub unsafe fn try_from_lpwstr(ptr: LPWSTR) -> Option<Self> {
        if ptr.is_null() {
            None
        } else {
            Some(Self(ptr))
        }
    }

    /// Get a mut ptr to the widestring
    pub fn as_mut_ptr(&mut self) -> LPWSTR {
        self.0
    }

    /// Get the length of the string in characters.
    ///
    /// # Panics
    /// Panics if the length cannot fit in a `usize`.
    pub fn len(&self) -> usize {
        unsafe {
            lstrlenW(self.0)
                .try_into()
                .expect("length cannot fit in a `usize`")
        }
    }

    //// Check if this string is empty.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Get this string as a slice of u16s.
    ///
    /// # Panics
    /// Panics if the length cannot fit in a `usize`.
    pub fn as_slice(&self) -> &[u16] {
        unsafe { std::slice::from_raw_parts(self.0, self.len()) }
    }

    /// Get this as an [`OsString`].
    ///
    /// # Panics
    /// Panics if the length cannot fit in a `usize`.
    pub fn as_os_string(&self) -> OsString {
        OsString::from_wide(self.as_slice())
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
        for c in std::char::decode_utf16(self.as_slice().iter().copied())
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

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn get_user_name_works() {
        let user_name = get_user_name().unwrap();
        dbg!(user_name);
    }
}
