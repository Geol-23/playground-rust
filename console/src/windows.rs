use std::mem;

pub type SHORT = i16;
pub type WORD = u16;
pub type DWORD = u32;
pub type BOOL = i32;
pub type HANDLE = *mut std::ffi::c_void;

pub const STD_INPUT_HANDLE: DWORD = -10i32 as DWORD;
pub const STD_OUTPUT_HANDLE: DWORD = -11i32 as DWORD;
pub const STD_ERROR_HANDLE: DWORD = -12i32 as DWORD;
pub const INVALID_HANDLE_VALUE: HANDLE = (-1 as i64) as HANDLE;

pub const ENABLE_ECHO_INPUT: DWORD = 0x0004;
pub const ENABLE_LINE_INPUT: DWORD = 0x0002;
pub const ENABLE_PROCESSED_INPUT: DWORD = 0x0001;

#[repr(C)]
#[allow(non_snake_case)]
pub struct COORD {
    X: SHORT,
    Y: SHORT,
}

#[repr(C)]
#[allow(non_snake_case)]
pub struct SMALL_RECT {
    Left: SHORT,
    Top: SHORT,
    Right: SHORT,
    Bottom: SHORT,
}

#[repr(C)]
#[allow(non_snake_case)]
pub struct CONSOLE_SCREEN_BUFFER_INFO {
    dwSize: COORD,
    dwCursorPosition: COORD,
    wAttributes: WORD,
    srWindow: SMALL_RECT,
    dwMaximumWindowSize: COORD,
}

#[link(name = "kernel32")]
unsafe extern "system" {
    fn GetStdHandle(nStdHandle: DWORD) -> HANDLE;
    fn GetConsoleScreenBufferInfo(
        hConsoleOutput: HANDLE,
        lpConsoleScreenBufferInfo: *mut CONSOLE_SCREEN_BUFFER_INFO
    ) -> BOOL;
    // fn GetConsoleMode(hConsoleHandle: HANDLE, lpMode: *mut DWORD) -> BOOL;
    // fn SetConsoleMode(hConsoleHandle: HANDLE, dwMode: DWORD) -> BOOL;
}

#[derive(Debug)]
pub struct TerminalSize(pub usize, pub usize);
pub fn get_terminal_size() -> Result<TerminalSize, &'static str> {
    unsafe {
        let handle = GetStdHandle(STD_OUTPUT_HANDLE);
        if handle == INVALID_HANDLE_VALUE { return Err("Failed to get stdout handle") }
        if handle.is_null() { return Err("No associated standard handles for this application") }

        let mut buffinfo: CONSOLE_SCREEN_BUFFER_INFO = mem::zeroed();
        if GetConsoleScreenBufferInfo(handle, &mut buffinfo) == 0 {
            return Err("Failed to get console screen buffer info")
        }
        let window = buffinfo.srWindow;
        let x: usize = (window.Right - window.Left).try_into().unwrap();
        let y: usize = (window.Bottom - window.Top).try_into().unwrap();
        return Ok(TerminalSize(x, y))
    }
}
