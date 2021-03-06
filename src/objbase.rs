use winapi::shared::winerror::CLASS_E_NOAGGREGATION;
use winapi::shared::winerror::CO_E_NOTINITIALIZED;
use winapi::shared::winerror::REGDB_E_CLASSNOTREG;
use winapi::shared::winerror::S_OK;
use winapi::um::combaseapi::CoIncrementMTAUsage;
use winapi::um::winnt::HRESULT;

// TODO: Consider making a more generic error design
/// An error that may occur while creating or using a COM object.
///
#[derive(Debug, PartialEq)]
pub enum ComError {
    /// Class ID was not registered.
    ///
    ClassNotRegistered,

    /// This class cannot be made as part of an aggregate.
    ///
    NoAggregation,

    /// COM is not initalized.
    ///
    ComNotInit,

    /// Unknown COM Error.
    ///
    Unknown(HRESULT),
}

impl From<HRESULT> for ComError {
    fn from(code: HRESULT) -> Self {
        match code {
            REGDB_E_CLASSNOTREG => Self::ClassNotRegistered,
            CLASS_E_NOAGGREGATION => Self::NoAggregation,
            CO_E_NOTINITIALIZED => Self::ComNotInit,
            code => Self::Unknown(code),
        }
    }
}

impl std::fmt::Display for ComError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match &self {
            Self::ClassNotRegistered => write!(
                f,
                "failed to create COM interface, class was not registered"
            ),
            Self::NoAggregation => write!(
                f,
                "failed to create COM interface, class cannot be made as part of an aggregation"
            ),
            Self::ComNotInit => write!(f, "COM is not initalized"),
            Self::Unknown(code) => write!(f, "unknown error (code 0x{:X})", code),
        }
    }
}

impl std::error::Error for ComError {}

// TODO: Consider returning cookie
/// Init a MTA COM runtime. Only needs to be called once per process.
///
/// # Errors
/// Returns an error if an MTA COM Runtime could not be created.
///
pub fn init_mta_com_runtime() -> Result<(), ComError> {
    let mut cookie = std::ptr::null_mut();
    let ret = unsafe { CoIncrementMTAUsage(&mut cookie) };

    match ret {
        S_OK => Ok(()),
        code => Err(ComError::from(code)),
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
