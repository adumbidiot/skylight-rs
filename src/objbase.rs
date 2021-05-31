use winapi::shared::guiddef::CLSID;
use winapi::shared::winerror::FAILED;
use winapi::shared::winerror::HRESULT;
use winapi::shared::wtypesbase::CLSCTX_INPROC_SERVER;
use winapi::um::combaseapi::CoCreateInstance;
use winapi::um::combaseapi::CoIncrementMTAUsage;
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
pub unsafe fn create_instance<T: Interface>(class_id: &CLSID) -> Result<*mut T, HRESULT> {
    let mut instance = std::ptr::null_mut();
    let hr = CoCreateInstance(
        class_id,
        std::ptr::null_mut(),
        CLSCTX_INPROC_SERVER,
        &T::uuidof(),
        &mut instance,
    );

    if FAILED(hr) {
        return Err(hr);
    }

    Ok(instance.cast())
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn init_mta_com() {
        init_mta_com_runtime().expect("COM runtime");
    }
}
