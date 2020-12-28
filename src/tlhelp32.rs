use std::ffi::OsString;
use std::os::windows::ffi::OsStringExt;
use winapi::shared::minwindef::DWORD;
use winapi::shared::minwindef::TRUE;
use winapi::um::handleapi::CloseHandle;
use winapi::um::handleapi::INVALID_HANDLE_VALUE;
use winapi::um::tlhelp32::CreateToolhelp32Snapshot;
use winapi::um::tlhelp32::Process32FirstW;
use winapi::um::tlhelp32::Process32NextW;
use winapi::um::tlhelp32::PROCESSENTRY32W;
use winapi::um::tlhelp32::TH32CS_SNAPALL;
use winapi::um::winnt::HANDLE;

// TODO: Finish Mask
bitflags::bitflags! {
    /// The flags to pass when creating a new Snapshot
    pub struct SnapshotFlags: DWORD {
        const SNAP_ALL = TH32CS_SNAPALL;
    }
}

/// A Snapshot of process and heap info
#[derive(Debug)]
pub struct Snapshot(HANDLE);

impl Snapshot {
    /// Get a new snapshot
    pub fn new(flags: SnapshotFlags) -> Result<Self, std::io::Error> {
        let handle = unsafe { CreateToolhelp32Snapshot(flags.bits(), 0) };
        if handle == INVALID_HANDLE_VALUE {
            return Err(std::io::Error::last_os_error());
        }
        Ok(Self(handle))
    }

    /// Iter processes
    pub fn iter_processes(&mut self) -> ProcessIter {
        ProcessIter::from_snapshot(self)
    }
}

impl Drop for Snapshot {
    fn drop(&mut self) {
        unsafe {
            CloseHandle(self.0);
        }
    }
}

/// Process Iterator
pub struct ProcessIter<'a> {
    current: PROCESSENTRY32W,
    has_more: bool,
    snapshot: &'a mut Snapshot,
}

impl<'a> ProcessIter<'a> {
    // TODO: Should this take `&'a Snapshot`?
    /// Make a process iter from a snapshot
    pub fn from_snapshot(snapshot: &'a mut Snapshot) -> Self {
        let mut current: PROCESSENTRY32W = unsafe { std::mem::zeroed() };
        current.dwSize = std::mem::size_of::<PROCESSENTRY32W>() as DWORD;

        let has_more = unsafe { Process32FirstW(snapshot.0, &mut current) == TRUE };

        ProcessIter {
            current,
            has_more,
            snapshot,
        }
    }
}

impl Iterator for ProcessIter<'_> {
    type Item = ProcessEntry;

    fn next(&mut self) -> Option<Self::Item> {
        if self.has_more {
            let ret = ProcessEntry::from(self.current);
            self.has_more = unsafe { Process32NextW(self.snapshot.0, &mut self.current) == TRUE };
            Some(ret)
        } else {
            None
        }
    }
}

/// A Process Entry
pub struct ProcessEntry(PROCESSENTRY32W);

impl ProcessEntry {
    /// Get the PID
    pub fn pid(&self) -> u32 {
        self.0.th32ProcessID
    }

    /// Get the number of threads
    pub fn num_threads(&self) -> u32 {
        self.0.cntThreads
    }

    /// Get the thread base priority
    pub fn thread_base_priority(&self) -> i32 {
        self.0.pcPriClassBase
    }

    /// Get the exe name as a u16 slice
    pub fn exe_name_slice(&self) -> &[u16] {
        let len = self
            .0
            .szExeFile
            .iter()
            .position(|el| *el == 0)
            .unwrap_or(self.0.szExeFile.len());

        &self.0.szExeFile[..len]
    }

    /// Get the exe name as an OsString. This allocates per call, so cache the result.
    /// If you want possibly-malformed utf16, use `exe_name_slice` instead.
    pub fn exe_name(&self) -> OsString {
        OsString::from_wide(self.exe_name_slice())
    }
}

impl std::fmt::Debug for ProcessEntry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ProcessEntry")
            .field("pid", &self.pid())
            .field("num_threads", &self.num_threads())
            .field("thread_base_priority", &self.thread_base_priority())
            .field("exe_name", &self.exe_name())
            .finish()
    }
}

impl From<PROCESSENTRY32W> for ProcessEntry {
    fn from(entry: PROCESSENTRY32W) -> Self {
        Self(entry)
    }
}
