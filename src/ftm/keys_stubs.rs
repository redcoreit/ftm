use windows::Win32::Foundation::{HWND};

pub fn is_whitelist_fs_app(hwnd: HWND) -> bool {
	// TODO: make this list configurable
    let whitelist = ["conhost", "wezterm-gui", "WindowsTerminal"];

    let proc_name = crate::win32::get_proc_hwnd_pname(hwnd);
    if proc_name.is_none() {
        println!("Failed to get process name for hwnd: {:?}", hwnd);
        return false;
    }
    let proc_name = proc_name.unwrap();

    for item in whitelist {
        if proc_name.eq_ignore_ascii_case(item) {
            println!("Process '{}' is whitelisted", proc_name);
            return true;
        }
    }

    println!("Process '{}' is not whitelisted", proc_name);
    false
}
