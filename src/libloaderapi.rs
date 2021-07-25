use std::ffi::OsStr;
use std::mem::ManuallyDrop;
use std::os::windows::ffi::OsStrExt;
use winapi::shared::minwindef::HMODULE;
use winapi::um::libloaderapi::FreeLibrary;
use winapi::um::libloaderapi::LoadLibraryW;

/// A dynamically loaded library
pub struct HModule(HMODULE);

impl HModule {
    /// Load a library from a string/path.
    ///
    /// # Safety
    /// The startup code for this dll must not cause UB.
    pub unsafe fn load(lib: &OsStr) -> std::io::Result<Self> {
        let lib = lib.encode_wide().chain(Some(0)).collect::<Vec<_>>();
        let hmodule = LoadLibraryW(lib.as_ptr());
        if hmodule.is_null() {
            return Err(std::io::Error::last_os_error());
        }
        Ok(Self(hmodule))
    }

    /// Get the raw HMODULE
    pub fn as_raw(&self) -> HMODULE {
        self.0
    }

    /// Destroy this object.
    pub fn destroy(self) -> Result<(), (Self, std::io::Error)> {
        let lib = ManuallyDrop::new(self);
        let ret = unsafe { FreeLibrary(lib.0) };
        if ret == 0 {
            return Err((
                ManuallyDrop::into_inner(lib),
                std::io::Error::last_os_error(),
            ));
        }

        Ok(())
    }
}

// All APIs seem threadsafe?
unsafe impl Send for HModule {}

impl Drop for HModule {
    fn drop(&mut self) {
        // Prevent recursive drop
        std::mem::forget(Self(self.0).destroy());
    }
}
