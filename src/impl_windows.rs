use std::ffi::OsStr;
use std::os::raw::c_ushort;
use std::os::windows::ffi::OsStrExt;

use windows_sys::Win32::Foundation::FALSE;
use windows_sys::Win32::System::Com::{COINIT_APARTMENTTHREADED, COINIT_DISABLE_OLE1DDE, CoInitializeEx};
use windows_sys::Win32::System::Threading::GetExitCodeProcess;
use windows_sys::Win32::System::Threading::INFINITE;
use windows_sys::Win32::System::Threading::WaitForSingleObject;
use windows_sys::Win32::UI::Shell::SEE_MASK_NOASYNC;
use windows_sys::Win32::UI::Shell::SEE_MASK_NOCLOSEPROCESS;
use windows_sys::Win32::UI::Shell::{SHELLEXECUTEINFOW, ShellExecuteExW};
use windows_sys::Win32::UI::WindowsAndMessaging::{SW_HIDE, SW_NORMAL};

use crate::Command;

unsafe fn win_runas(cmd: *const c_ushort, args: *const c_ushort, show: bool, wait: bool) -> std::io::Result<u32> {
    let mut code = 0;
    let mut sei: SHELLEXECUTEINFOW = unsafe { std::mem::zeroed() };
    let verb = "runas\0".encode_utf16().collect::<Vec<u16>>();
    unsafe { CoInitializeEx(std::ptr::null(), (COINIT_APARTMENTTHREADED | COINIT_DISABLE_OLE1DDE) as u32) };

    sei.fMask = SEE_MASK_NOASYNC | SEE_MASK_NOCLOSEPROCESS;
    sei.cbSize = std::mem::size_of::<SHELLEXECUTEINFOW>() as _;
    sei.lpVerb = verb.as_ptr();
    sei.lpFile = cmd;
    sei.lpParameters = args;
    sei.nShow = if show { SW_NORMAL } else { SW_HIDE } as _;

    if unsafe { ShellExecuteExW(&mut sei) } == FALSE || sei.hProcess.is_null() {
        return Err(std::io::Error::last_os_error());
    }

    if wait {
        unsafe { WaitForSingleObject(sei.hProcess, INFINITE) };

        if unsafe { GetExitCodeProcess(sei.hProcess, &mut code) } == FALSE {
            return Err(std::io::Error::last_os_error());
        }
    }
    Ok(code)
}

pub fn runas_impl(cmd: &Command) -> std::io::Result<std::process::ExitStatus> {
    let mut params = String::new();
    for arg in cmd.args.iter() {
        let arg = arg.to_string_lossy();
        params.push(' ');
        if arg.is_empty() {
            params.push_str("\"\"");
        } else if arg.find(&[' ', '\t', '"'][..]).is_none() {
            params.push_str(&arg);
        } else {
            params.push('"');
            for c in arg.chars() {
                match c {
                    '\\' => params.push_str("\\\\"),
                    '"' => params.push_str("\\\""),
                    c => params.push(c),
                }
            }
            params.push('"');
        }
    }

    let file = OsStr::new(&cmd.command).encode_wide().chain(Some(0)).collect::<Vec<_>>();
    let params = OsStr::new(&params).encode_wide().chain(Some(0)).collect::<Vec<_>>();

    let status = unsafe { win_runas(file.as_ptr(), params.as_ptr(), !cmd.hide, cmd.wait_to_complete)? };
    use std::os::windows::process::ExitStatusExt;
    Ok(std::process::ExitStatus::from_raw(status))
}

/// Check if the current process is running with elevated privileges.
pub fn is_elevated() -> bool {
    use windows_sys::Win32::Foundation::{CloseHandle, FALSE};
    use windows_sys::Win32::Security::{GetTokenInformation, TOKEN_ELEVATION, TOKEN_QUERY, TokenElevation};
    use windows_sys::Win32::System::Threading::{GetCurrentProcess, OpenProcessToken};
    use windows_sys::core::BOOL;

    let mut token = std::ptr::null_mut();
    if unsafe { OpenProcessToken(GetCurrentProcess(), TOKEN_QUERY, &mut token) } == FALSE {
        return false;
    }
    let mut elevation = TOKEN_ELEVATION { TokenIsElevated: 0 };
    let len = std::mem::size_of::<TOKEN_ELEVATION>() as u32;
    let mut ret_len = 0;
    let res: BOOL = unsafe { GetTokenInformation(token, TokenElevation, &mut elevation as *mut _ as *mut _, len, &mut ret_len) };
    unsafe { CloseHandle(token) };
    res != FALSE && elevation.TokenIsElevated != 0
}
