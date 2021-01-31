use std::mem::ManuallyDrop;
use std::os::windows::raw::HANDLE;
use winapi::um::handleapi::CloseHandle;

// TODO: Consider allowing invalid handles.
/// A wrapper around a winapi `HANDLE`.
///
#[repr(transparent)]
#[derive(Debug)]
pub struct Handle(HANDLE);

impl Handle {
    /// Make a new [`Handle`] from a `HANDLE`.
    ///
    /// # Safety
    /// `handle` must be a valid `HANDLE` that is non_null.    
    ///
    pub unsafe fn from_raw(handle: HANDLE) -> Self {
        debug_assert!(!handle.is_null());
        Self(handle)
    }

    /// Get the inner `HANDLE`.
    ///
    pub fn as_raw(&self) -> HANDLE {
        self.0
    }

    /// Get the inner `HANDLE`, consuming this object and NOT running `Drop`.
    ///
    pub fn into_raw(self) -> HANDLE {
        ManuallyDrop::new(self).0
    }

    /// Try to close this [`Handle`].
    ///
    /// # Errors
    /// Returns an error which contains this [`Handle`] if this could not be destroyed.
    ///
    pub fn close(self) -> Result<(), (Self, std::io::Error)> {
        let handle = ManuallyDrop::new(self);
        let ret = unsafe { CloseHandle(handle.0.cast()) };

        if ret != 0 {
            Ok(())
        } else {
            Err((
                ManuallyDrop::into_inner(handle),
                std::io::Error::last_os_error(),
            ))
        }
    }
}

impl Drop for Handle {
    fn drop(&mut self) {
        std::mem::forget(Self(self.0).close());
    }
}
