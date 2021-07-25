/// handleapi.h Utilities
#[cfg(feature = "handleapi")]
pub mod handleapi;
#[cfg(feature = "handleapi")]
pub use self::handleapi::*;

/// libloaderapi.h Utilities
#[cfg(feature = "libloaderapi")]
pub mod libloaderapi;
#[cfg(feature = "libloaderapi")]
pub use self::libloaderapi::*;

/// objbase.h Utilities
#[cfg(feature = "objbase")]
pub mod objbase;
#[cfg(feature = "objbase")]
pub use self::objbase::*;

/// oleauto.h Utilities
#[cfg(feature = "oleauto")]
pub mod oleauto;
#[cfg(feature = "oleauto")]
pub use self::oleauto::*;

/// processthreadsapi.h Utilities
#[cfg(feature = "processthreadsapi")]
pub mod processthreadsapi;
#[cfg(feature = "processthreadsapi")]
pub use self::processthreadsapi::*;

/// shlobj.h Utilities
#[cfg(feature = "shlobj")]
pub mod shlobj;
#[cfg(feature = "shlobj")]
pub use self::shlobj::*;

/// tlhelp32.h Utilities
#[cfg(feature = "tlhelp32")]
pub mod tlhelp32;
#[cfg(feature = "tlhelp32")]
pub use self::tlhelp32::*;

/// winbase.h Utilities
#[cfg(feature = "winbase")]
pub mod winbase;
#[cfg(feature = "winbase")]
pub use self::winbase::*;

/// wincrypt.h Utilities
#[cfg(feature = "wincrypt")]
pub mod wincrypt;
#[cfg(feature = "wincrypt")]
pub use self::wincrypt::*;

/// winerror.h Utilities
#[cfg(feature = "winerror")]
pub mod winerror;
#[cfg(feature = "winerror")]
pub use self::winerror::*;
