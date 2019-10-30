/**
 * Interaction with the console the app is running in.
 */

use crate::format;

#[cfg(windows)]
#[allow(non_camel_case_types)]
mod win32 {
    pub type DWORD = u64;
    pub type BOOL = i32;

    pub type HANDLE = *mut ();
    pub type LPDWORD = *mut DWORD;

    // STD Handles
    pub const STD_OUTPUT_HANDLE : DWORD = 4294967285;
    pub const STD_ERROR_HANDLE : DWORD = 4294967284;

    // Console mode flags
    pub const ENABLE_VIRTUAL_TERMINAL_PROCESSING: DWORD = 0x0004;


    #[link(name = "kernel32")]
    extern "stdcall" {
        pub fn GetStdHandle(nStdHandle: DWORD) -> HANDLE;
        pub fn GetConsoleMode(hConsoleHandle: HANDLE, lpMode: LPDWORD) -> BOOL;
        pub fn SetConsoleMode(hConsoleHandle: HANDLE, dwMode: DWORD) -> BOOL;
    }

    pub static supports_color: bool = false;

    pub fn enable_virtual_terminal_processing(nStdHandle: DWORD) -> bool {
        unsafe {
            let h = GetStdHandle(nStdHandle);
            if h == std::ptr::null() {
                return false;
            }
            let mut mode: DWORD = 0;
            if GetConsoleMode(h, &mut mode) == 0 {
                return false;
            }
            mode |= ENABLE_VIRTUAL_TERMINAL_PROCESSING;
            if SetConsoleMode(h, mode) == 0 {
                return false;
            }
            return true;
        }
    }
}

#[cfg(windows)]
pub fn init_console() {
    unsafe {
        win32::supports_color = true;
        win32::supports_color &= win32::enable_virtual_terminal_processing(win32::STD_OUTPUT_HANDLE);
        win32::supports_color &= win32::enable_virtual_terminal_processing(win32::STD_ERROR_HANDLE);
    }
}

#[cfg(not(windows))]
pub fn init_console() { }

pub fn set_console_format(fmt: &format::Fmt) {

}
