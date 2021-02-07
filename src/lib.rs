/// handleapi.h Utilities
#[cfg(feature = "handleapi")]
pub mod handleapi;
#[cfg(feature = "handleapi")]
pub use self::handleapi::*;

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

/// tlhelp32.h Utilities
#[cfg(feature = "tlhelp32")]
pub mod tlhelp32;
#[cfg(feature = "tlhelp32")]
pub use self::tlhelp32::*;
