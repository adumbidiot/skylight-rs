use crate::HModule;
use crate::LocalWideString;
use std::ptr::NonNull;
use winapi::shared::ntdef::LANG_SYSTEM_DEFAULT;
use winapi::shared::ntdef::MAKELANGID;
use winapi::shared::ntdef::SUBLANG_SYS_DEFAULT;
use winapi::um::winbase::FormatMessageW;
use winapi::um::winbase::FORMAT_MESSAGE_ALLOCATE_BUFFER;
use winapi::um::winbase::FORMAT_MESSAGE_FROM_HMODULE;
use winapi::um::winbase::FORMAT_MESSAGE_FROM_SYSTEM;
use winapi::um::winbase::FORMAT_MESSAGE_IGNORE_INSERTS;

/// A wrapper for a windows HRESULT.
pub struct HResult(pub u32);

impl HResult {
    /// Get the message for this error using default settings.
    pub fn message(&self) -> std::io::Result<LocalWideString> {
        self.message_with_hmodule(None)
    }

    /// Get the message for this error loading definitions from a given dll.
    ///
    /// The dll must be loaded in this process when this function is called.
    pub fn message_with_hmodule(
        &self,
        module: Option<&HModule>,
    ) -> std::io::Result<LocalWideString> {
        let mut flags = 0;

        if module.is_some() {
            flags |= FORMAT_MESSAGE_FROM_HMODULE;
        }

        let mut ptr: *mut u16 = std::ptr::null_mut();
        let size = unsafe {
            FormatMessageW(
                flags
                    | FORMAT_MESSAGE_ALLOCATE_BUFFER
                    | FORMAT_MESSAGE_FROM_SYSTEM
                    | FORMAT_MESSAGE_IGNORE_INSERTS,
                module
                    .map(|hmodule| hmodule.as_raw())
                    .unwrap_or(std::ptr::null_mut())
                    .cast(),
                self.0,
                MAKELANGID(LANG_SYSTEM_DEFAULT, SUBLANG_SYS_DEFAULT).into(),
                std::mem::transmute(&mut ptr), // This param is a *mut u16, but needs to accept a *mut *mut u16 since we sepcify the FORMAT_MESSAGE_ALLOCATE_BUFFER flag.
                0,
                std::ptr::null_mut(),
            )
        };

        if size == 0 || ptr.is_null() {
            return Err(std::io::Error::last_os_error());
        }

        let ptr = NonNull::new(ptr).expect("ptr is null");
        let ret = unsafe { LocalWideString::from_raw(ptr) };

        Ok(ret)
    }
}

impl From<u32> for HResult {
    fn from(data: u32) -> Self {
        Self(data)
    }
}

impl From<i32> for HResult {
    fn from(data: i32) -> Self {
        // `as` is basically a safe transmute here
        Self(data as u32)
    }
}

impl From<HResult> for std::io::Error {
    fn from(result: HResult) -> Self {
        // `as` is basically a safe transmute here
        std::io::Error::from_raw_os_error(result.0 as i32)
    }
}

impl std::fmt::Display for HResult {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.message() {
            Ok(msg) => msg.display().fmt(f),
            Err(e) => e.fmt(f),
        }
    }
}

impl std::fmt::Debug for HResult {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.message() {
            Ok(msg) => msg.fmt(f),
            Err(e) => e.fmt(f),
        }
    }
}

impl std::error::Error for HResult {}

#[cfg(test)]
mod test {
    use super::*;
    use winapi::shared::winerror::CLASS_E_NOAGGREGATION;
    use winapi::shared::winerror::CO_E_NOTINITIALIZED;
    use winapi::shared::winerror::REGDB_E_CLASSNOTREG;
    use winapi::shared::winerror::RPC_E_CHANGED_MODE;
    use winapi::shared::winerror::S_FALSE;
    use winapi::shared::winerror::S_OK;

    #[test]
    fn display_s_ok() {
        assert!(HResult::from(S_OK).message().is_ok());
    }

    #[test]
    fn display_s_false() {
        assert!(HResult::from(S_FALSE).message().is_ok());
    }

    #[test]
    fn display_rpc_e_changed_mode() {
        assert!(HResult::from(RPC_E_CHANGED_MODE).message().is_ok());
    }

    #[test]
    fn display_co_e_not_initialized() {
        assert!(HResult::from(CO_E_NOTINITIALIZED).message().is_ok());
    }

    #[test]
    fn display_regdb_e_class_not_reg() {
        assert!(HResult::from(REGDB_E_CLASSNOTREG).message().is_ok());
    }

    #[test]
    fn display_class_e_no_aggregation() {
        assert!(HResult::from(CLASS_E_NOAGGREGATION).message().is_ok());
    }
}
