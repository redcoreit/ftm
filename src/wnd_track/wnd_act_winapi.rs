extern crate winapi;

use winapi::shared::windef::HWND;
use winapi::um::winuser::{SetForegroundWindow, ShowWindow, SW_RESTORE, SetActiveWindow};

pub fn activate_window(hwnd: HWND) {
    unsafe {
        // Restore the window if it is minimized
        ShowWindow(hwnd, SW_RESTORE);

        // Bring the window to the foreground
        if SetForegroundWindow(hwnd) == 0 {
            // If unable to set foreground, explicitly activate it
            SetActiveWindow(hwnd);
        }
    }
}
