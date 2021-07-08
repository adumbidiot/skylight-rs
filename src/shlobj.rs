use crate::objbase::CoTaskMemWideString;
use std::convert::TryInto;
use std::ffi::OsString;
use std::mem::MaybeUninit;
use std::os::windows::ffi::OsStringExt;
use std::path::PathBuf;
use std::ptr::NonNull;
use winapi::ctypes::c_int;
use winapi::shared::guiddef::GUID;
use winapi::shared::minwindef::FALSE;
use winapi::shared::minwindef::MAX_PATH;
use winapi::shared::minwindef::TRUE;
use winapi::shared::winerror::S_OK;
use winapi::um::knownfolders::FOLDERID_Desktop;
use winapi::um::knownfolders::FOLDERID_LocalAppData;
use winapi::um::shlobj::SHGetKnownFolderPath;
use winapi::um::shlobj::SHGetSpecialFolderPathW;
use winapi::um::shlobj::CSIDL_DESKTOP;
use winapi::um::winbase::lstrlenW;

/// A folder type
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
/// Known Folder Ids
pub enum FolderId {
    /// The current user's desktop
    Desktop,

    /// The folder that is a "data repository for local (nonroaming) applications"
    LocalAppData,
}

impl From<FolderId> for GUID {
    fn from(folder_id: FolderId) -> Self {
        match folder_id {
            FolderId::Desktop => FOLDERID_Desktop,
            FolderId::LocalAppData => FOLDERID_LocalAppData,
        }
    }
}

/// Get a known folder path.
///
/// # Errors
/// * Returns an error if the path could not be retrieved.
///
/// # Panics
/// * Panics if the operation was successful, yet the path pointer is still null.
pub fn get_known_folder_path(folder_id: FolderId) -> std::io::Result<CoTaskMemWideString> {
    let folder_id: GUID = folder_id.into();
    let mut path_ptr = std::ptr::null_mut();
    let ret = unsafe { SHGetKnownFolderPath(&folder_id, 0, std::ptr::null_mut(), &mut path_ptr) };
    let path = NonNull::new(path_ptr).map(|ptr| unsafe { CoTaskMemWideString::from_raw(ptr) });

    if ret != S_OK {
        return Err(std::io::Error::from_raw_os_error(ret));
    }

    Ok(path.expect("path ptr was null"))
}

/// The location of a folder
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum ConstantSpecialItemIdList {
    /// The desktop
    Desktop,
}

impl From<ConstantSpecialItemIdList> for c_int {
    fn from(csidl: ConstantSpecialItemIdList) -> c_int {
        match csidl {
            ConstantSpecialItemIdList::Desktop => CSIDL_DESKTOP,
        }
    }
}

/// Get a folder path from a csidl.
///
/// Note: This function is considered legacy.
///
/// # Errors
/// * Returns `None` if the csidl path could not be located.
pub fn get_special_folder_path(
    csidl: ConstantSpecialItemIdList,
    create_folder: bool,
) -> Option<PathBuf> {
    const BUFFER_LEN: usize = MAX_PATH + 1;

    let mut buffer = MaybeUninit::<[u16; BUFFER_LEN]>::uninit();
    let create_folder = if create_folder { TRUE } else { FALSE };

    let csidl: c_int = csidl.into();

    // # Safety
    // The buffer exists and has a minimum length of MAX_PATH.
    let ret = unsafe {
        SHGetSpecialFolderPathW(
            std::ptr::null_mut(),
            buffer.as_mut_ptr().cast(),
            csidl,
            create_folder,
        )
    };

    if ret != TRUE {
        return None;
    }

    // # Safety
    // The data must be valid at this point.
    // The data is NUL terminated.
    // There are only immutable references left to `buffer`, so making another immutable one is safe.
    let buffer = unsafe {
        let len: usize = lstrlenW(buffer.as_ptr().cast())
            .try_into()
            .expect("could not convert string length into a `usize`");
        std::slice::from_raw_parts(buffer.as_ptr().cast(), len)
    };

    Some(OsString::from_wide(buffer).into())
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn get_special_folder_path_smoke() {
        let desktop = get_special_folder_path(ConstantSpecialItemIdList::Desktop, false)
            .expect("failed to get desktop");
        dbg!(desktop);
    }

    #[test]
    fn get_known_folder_path_smoke() {
        let desktop = get_known_folder_path(FolderId::Desktop).expect("failed to get desktop");
        dbg!(desktop);
        let local_app_data =
            get_known_folder_path(FolderId::LocalAppData).expect("failed to get local_app_data");
        dbg!(local_app_data);
    }
}
