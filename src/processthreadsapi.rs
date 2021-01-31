use crate::handleapi::Handle;
use winapi::shared::minwindef::DWORD;
use winapi::shared::minwindef::FALSE;
use winapi::um::processthreadsapi::OpenProcess;
use winapi::um::processthreadsapi::TerminateProcess;
use winapi::um::synchapi::WaitForSingleObject;
use winapi::um::winbase::WAIT_FAILED;
use winapi::um::winnt::PROCESS_TERMINATE;
use winapi::um::winnt::SYNCHRONIZE;

// TODO: Finish Flags
bitflags::bitflags! {
    /// Process access rights for opening access to a process.
    ///
    pub struct ProcessAccessRights: DWORD {

        /// Terminate right
        ///
        const TERMINATE = PROCESS_TERMINATE;

        /// Synchronize right
        ///
        const SYNCHRONIZE = SYNCHRONIZE;
    }
}

/// A Process
#[derive(Debug)]
pub struct Process(Handle);

impl Process {
    /// Open an existing process
    pub fn open(access_rights: ProcessAccessRights, pid: DWORD) -> std::io::Result<Self> {
        let handle = unsafe { OpenProcess(access_rights.bits(), FALSE, pid) };

        if handle.is_null() {
            Err(std::io::Error::last_os_error())
        } else {
            unsafe { Ok(Self(Handle::from_raw(handle.cast()))) }
        }
    }

    /// Signal this process to terminate.
    /// This requires the `TERMINATE` permission.
    pub fn terminate(&self, exit_code: u32) -> std::io::Result<()> {
        if unsafe { TerminateProcess(self.0.as_raw().cast(), exit_code) == FALSE } {
            return Err(std::io::Error::last_os_error());
        }

        Ok(())
    }

    /// Wait for this process to terminate until the given interval elapses, immediately if it is 0, and indefinitely if it is `u32::MAX`.
    /// This requires the `SYNCHRONIZE` permission.
    pub fn wait(&self, millis: u32) -> std::io::Result<()> {
        let ret = unsafe { WaitForSingleObject(self.0.as_raw().cast(), millis) };

        if ret == WAIT_FAILED {
            return Err(std::io::Error::last_os_error());
        }

        Ok(())
    }

    /// Try to close this [`Process`] handle.
    ///
    /// # Errors
    /// Returns an error which contains this object if this object could not be destroyed.
    ///
    pub fn close(self) -> Result<(), (Self, std::io::Error)> {
        self.0.close().map_err(|(handle, err)| (Self(handle), err))
    }
}
