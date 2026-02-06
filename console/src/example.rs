use std::io::{self, Read};
use std::mem::MaybeUninit;
use std::ptr::null_mut;
use std::os::windows::io::AsRawHandle;

type HANDLE = *mut std::ffi::c_void;
type DWORD = u32;
type BOOL = i32;

const STD_INPUT_HANDLE: DWORD = -10i32 as DWORD;
const ENABLE_ECHO_INPUT: DWORD = 0x0004;
const ENABLE_LINE_INPUT: DWORD = 0x0002;
const ENABLE_PROCESSED_INPUT: DWORD = 0x0001;

extern "system" {
    fn GetStdHandle(nStdHandle: DWORD) -> HANDLE;
    fn GetConsoleMode(hConsoleHandle: HANDLE, lpMode: *mut DWORD) -> BOOL;
    fn SetConsoleMode(hConsoleHandle: HANDLE, dwMode: DWORD) -> BOOL;
}

fn example() {
    unsafe {
        let handle = GetStdHandle(STD_INPUT_HANDLE);
        if handle.is_null() {
            panic!("Failed to get input handle");
        }

        let mut original_mode: DWORD = 0;
        if GetConsoleMode(handle, &mut original_mode) == 0 {
            panic!("Failed to get console mode");
        }

        // Disable echo and line input → raw mode
        let raw_mode = original_mode & !(ENABLE_ECHO_INPUT | ENABLE_LINE_INPUT);
        if SetConsoleMode(handle, raw_mode) == 0 {
            panic!("Failed to set raw mode");
        }

        println!("Raw mode enabled. Press keys (q to quit):");

        let mut buffer = [0u8; 1];
        loop {
            io::stdin().read_exact(&mut buffer).unwrap();
            let c = buffer[0] as char;
            println!("You pressed: {:?}", c);
            if c == 'q' {
                break;
            }
        }

        // Restore original mode
        SetConsoleMode(handle, original_mode);
        println!("Restored cooked mode. Goodbye!");
    }
}
