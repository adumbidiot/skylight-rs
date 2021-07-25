use crate::winbase::LocalWideString;
use std::convert::TryInto;
use std::mem::ManuallyDrop;
use std::mem::MaybeUninit;
use std::ptr::NonNull;
use winapi::shared::minwindef::FALSE;
use winapi::um::dpapi::CryptUnprotectData;
use winapi::um::dpapi::CRYPTPROTECT_UI_FORBIDDEN;
use winapi::um::{
    winbase::{LocalAlloc, LocalFree},
    wincrypt::DATA_BLOB,
};

/// A wincrypt DataBlob.
#[repr(transparent)]
pub struct DataBlob(DATA_BLOB);

impl DataBlob {
    /// Make a [`DATA_BLOB`] from a byte slice.
    ///
    /// # Panics
    /// Panics if `data.len() > u32::MAX`.
    pub fn from_slice(data: &[u8]) -> Self {
        let len = data.len();
        let len_u32: u32 = len.try_into().expect("data.len() > u32::MAX");

        let buffer_ptr = unsafe {
            let ptr: *mut u8 = LocalAlloc(0, len).cast();
            assert!(!ptr.is_null(), "failed to allocate memory");

            std::ptr::copy(data.as_ptr(), ptr, len);
            ptr
        };

        let mut blob: MaybeUninit<DATA_BLOB> = MaybeUninit::uninit();
        Self(unsafe {
            (*blob.as_mut_ptr()).cbData = len_u32;
            (*blob.as_mut_ptr()).pbData = buffer_ptr;
            blob.assume_init()
        })
    }

    /// Get a mut ptr to the inner value
    pub fn as_mut_ptr(&mut self) -> *mut DATA_BLOB {
        &mut self.0
    }

    /// Get the length of this blob
    ///
    /// # Panics
    /// Panics if the length cannot fit in a `usize`.
    pub fn len(&self) -> usize {
        self.0
            .cbData
            .try_into()
            .expect("cannot fit length in a `usize`")
    }

    /// Check if this blob is empty.
    ///
    /// # Panics
    /// Panics if the length cannot fit in a `usize`.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Get this blob as a byte slice.
    pub fn as_slice(&self) -> &[u8] {
        unsafe { std::slice::from_raw_parts(self.0.pbData, self.len()) }
    }

    /// Try to destroy this object.
    ///
    /// # Errors
    /// Returns a tuple of this object and an error if this object could not be destroyed.
    pub fn destroy(self) -> Result<(), (Self, std::io::Error)> {
        let mut blob = ManuallyDrop::new(self);
        let ret = unsafe { LocalFree(blob.as_mut_ptr().cast()) };

        if ret.is_null() {
            Ok(())
        } else {
            Err((
                ManuallyDrop::into_inner(blob),
                std::io::Error::last_os_error(),
            ))
        }
    }
}

impl AsRef<[u8]> for DataBlob {
    fn as_ref(&self) -> &[u8] {
        self.as_slice()
    }
}

impl std::fmt::Debug for DataBlob {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DataBlob")
            .field("data", &self.as_slice())
            .finish()
    }
}

impl From<&[u8]> for DataBlob {
    fn from(data: &[u8]) -> Self {
        Self::from_slice(data)
    }
}

impl From<&Vec<u8>> for DataBlob {
    fn from(data: &Vec<u8>) -> Self {
        Self::from_slice(data)
    }
}

impl Drop for DataBlob {
    fn drop(&mut self) {
        std::mem::forget(Self(self.0).destroy());
    }
}

/// Data decrypted with [`crypt_unprotect_data`].
#[derive(Debug)]
pub struct DecryptedData {
    /// The decrypted data
    pub decrypted: DataBlob,

    /// The description of the decrypted data
    pub description: Option<LocalWideString>,
}

/// Decrypt data encrypted with `CryptProtectData`.
///
/// # Errors
/// Returns an error if the data could not be decrypted.
pub fn crypt_unprotect_data<E>(encrypted: E) -> std::io::Result<DecryptedData>
where
    E: Into<DataBlob>,
{
    let mut encrypted = encrypted.into();
    let mut decrypted: MaybeUninit<DataBlob> = MaybeUninit::zeroed();

    let mut description_ptr = std::ptr::null_mut();

    let ret = unsafe {
        CryptUnprotectData(
            encrypted.as_mut_ptr(),
            &mut description_ptr,
            std::ptr::null_mut(),
            std::ptr::null_mut(),
            std::ptr::null_mut(),
            CRYPTPROTECT_UI_FORBIDDEN,
            decrypted.as_mut_ptr().cast(),
        )
    };

    let description = NonNull::new(description_ptr)
        .map(|description_ptr| unsafe { LocalWideString::from_raw(description_ptr) });

    if ret == FALSE {
        return Err(std::io::Error::last_os_error());
    }

    Ok(DecryptedData {
        decrypted: unsafe { decrypted.assume_init() },
        description,
    })
}
