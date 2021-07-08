use std::ffi::OsStr;
use std::ffi::OsString;
use std::fmt::Write;
use std::os::windows::ffi::OsStrExt;
use std::os::windows::ffi::OsStringExt;
use std::ptr::NonNull;
use winapi::shared::guiddef::CLSID;
use winapi::shared::minwindef::DWORD;
use winapi::shared::winerror::FAILED;
use winapi::shared::winerror::HRESULT;
use winapi::um::combaseapi::CoCreateInstance;
use winapi::um::combaseapi::CoIncrementMTAUsage;
use winapi::um::combaseapi::CoTaskMemAlloc;
use winapi::um::combaseapi::CoTaskMemFree;
use winapi::Interface;

// TODO: Consider returning cookie
/// Init a MTA COM runtime. Only needs to be called once per process.
///
/// # Errors
/// Returns an error if an MTA COM Runtime could not be created.
pub fn init_mta_com_runtime() -> std::io::Result<()> {
    let mut cookie = std::ptr::null_mut();
    let code = unsafe { CoIncrementMTAUsage(&mut cookie) };

    if FAILED(code) {
        return Err(std::io::Error::from_raw_os_error(code));
    }

    Ok(())
}

// TODO: Try to make a safe but less flexible abstraction for this.
/// Make a new com object from the given class ID.
///
/// # Safety
/// The returned type must match the input class ID.
pub unsafe fn create_instance<T: Interface>(
    class_id: &CLSID,
    flags: DWORD,
) -> Result<*mut T, HRESULT> {
    let mut instance = std::ptr::null_mut();
    let hr = CoCreateInstance(
        class_id,
        std::ptr::null_mut(),
        flags,
        &T::uuidof(),
        &mut instance,
    );

    if FAILED(hr) {
        return Err(hr);
    }

    Ok(instance.cast())
}

/// A Wide String allocated with CoTaskMemAlloc.
pub struct CoTaskMemWideString(NonNull<u16>);

impl CoTaskMemWideString {
    /// Allocate a new string.
    ///
    /// # Errors
    /// * Returns `None` if the memory could not be allocated.
    pub fn new(data: &OsStr) -> Option<Self> {
        // +1 for NUL terminator
        let len = data.encode_wide().count() + 1;

        // x2 since wide chars have twice the bytes
        let ptr = unsafe { CoTaskMemAlloc(len * 2) };

        // Early return on allocation failure
        let ptr: NonNull<u16> = NonNull::new(ptr.cast())?;

        // Copy data + nul terminator
        unsafe {
            for (i, c) in data.encode_wide().chain(std::iter::once(0)).enumerate() {
                std::ptr::write(ptr.as_ptr().add(i), c);
            }
        }

        Some(Self(ptr))
    }

    /// Get the length of the string.
    ///
    /// This does not include the NUL terminator. This is O(n).
    pub fn len(&self) -> usize {
        self.iter().count()
    }

    /// Check if this string is empty.
    ///
    /// This does not include the NUL terminator. This is O(n).
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Iterate over the code points in this wide string.
    ///
    /// This does not include the NUL terminator.
    pub fn iter(&self) -> impl Iterator<Item = u16> + '_ {
        Iter::new(self)
    }

    /// Get this as an [`OsString`].
    ///
    /// This does not include the NUL terminator. This is O(n).
    pub fn as_os_string(&self) -> OsString {
        OsString::from_wide(self.as_slice())
    }

    /// Get a slice from this.
    ///
    /// This does not include the NUL terminator. This is O(n).
    pub fn as_slice(&self) -> &[u16] {
        let ptr = self.0.as_ptr();
        let len = self.len();

        unsafe { std::slice::from_raw_parts(ptr, len) }
    }
}

impl std::fmt::Debug for CoTaskMemWideString {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_char('"')?;
        for c in std::char::decode_utf16(self.iter())
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

impl Drop for CoTaskMemWideString {
    fn drop(&mut self) {
        unsafe {
            CoTaskMemFree(self.0.as_ptr().cast());
        }
    }
}

struct Iter<'a> {
    data: &'a CoTaskMemWideString,
    offset: isize,
}

impl<'a> Iter<'a> {
    /// Make a new [`Iter`] from a [`CoTaskMemWideString`].
    pub fn new(data: &'a CoTaskMemWideString) -> Self {
        Self { data, offset: 0 }
    }

    /// Get the current wide char.
    pub fn current(&self) -> u16 {
        unsafe { std::ptr::read(self.data.0.as_ptr().offset(self.offset)) }
    }

    /// Check if the current wide char is nul.
    pub fn current_is_nul(&self) -> bool {
        self.current() == 0
    }
}

impl Iterator for Iter<'_> {
    type Item = u16;

    fn next(&mut self) -> Option<Self::Item> {
        if self.current_is_nul() {
            return None;
        }

        let ret = self.current();
        self.offset += 1;

        Some(ret)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn init_mta_com() {
        init_mta_com_runtime().expect("failed to init COM runtime");
    }

    #[test]
    fn co_task_mem_wide_string_smoke() {
        {
            let hello_world_str = CoTaskMemWideString::new("hello_world!".as_ref())
                .expect("failed to allocate hello_world_str");
            dbg!(&hello_world_str);
            dbg!(hello_world_str.as_os_string());
            let empty_str =
                CoTaskMemWideString::new("".as_ref()).expect("failed to allocate empty_str");
            dbg!(&empty_str);
            dbg!(empty_str.as_os_string());
            assert!(empty_str.is_empty());
        }
    }
}
