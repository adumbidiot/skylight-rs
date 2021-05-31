use winapi::shared::winerror::S_OK;
use winapi::um::combaseapi::CoIncrementMTAUsage;

// TODO: Consider returning cookie
/// Init a MTA COM runtime. Only needs to be called once per process.
///
/// # Errors
/// Returns an error if an MTA COM Runtime could not be created.
pub fn init_mta_com_runtime() -> std::io::Result<()> {
    let mut cookie = std::ptr::null_mut();
    let ret = unsafe { CoIncrementMTAUsage(&mut cookie) };

    match ret {
        S_OK => Ok(()),
        code => Err(std::io::Error::from_raw_os_error(code)),
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn init_mta_com() {
        init_mta_com_runtime().expect("COM runtime");
    }
}
