use crate::objbase::CoTaskMemWideString;
use std::ptr::NonNull;
use winapi::shared::guiddef::GUID;
use winapi::shared::winerror::S_OK;
use winapi::um::knownfolders::FOLDERID_Desktop;
use winapi::um::knownfolders::FOLDERID_LocalAppData;
use winapi::um::shlobj::SHGetKnownFolderPath;

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
