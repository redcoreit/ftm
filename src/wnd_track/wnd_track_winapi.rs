// https://stackoverflow.com/questions/76339770/setwineventhook-not-capturing-any-events-in-rust-application
// https://docs.rs/winvd/0.0.48/winvd/#functions

use std::collections::HashMap;
use std::ffi::OsString;
use std::os::windows::ffi::OsStringExt;
use std::ptr;
use winapi::shared::minwindef::DWORD;
use winapi::shared::ntdef::LONG;
use winapi::shared::windef::{HWINEVENTHOOK, HWINEVENTHOOK__, HWND, HWND__};
use winapi::um::winuser::{
	GetWindowTextW, SetWinEventHook, UnhookWinEvent, EVENT_SYSTEM_FOREGROUND,
	WINEVENT_OUTOFCONTEXT,
};

use std::sync::{Mutex, OnceLock};
use crate::wnd_track::wnd_act_winapi;

pub fn wnd_tracker() -> &'static WndTracker {
	static INSTANCE: OnceLock<WndTracker> = OnceLock::new();
	INSTANCE
		.get_or_init(|| WndTracker::new().expect("Failed to create WndTracker instance."))
}

pub struct WndTracker {
	hook: Mutex<*mut HWINEVENTHOOK__>,
	active_wnds: Mutex<HashMap<u32, DedupRingStack<HWND>>>,
	desktop: Mutex<Option<u32>>,
}

impl WndTracker {
	fn new() -> Result<WndTracker, Box<dyn std::error::Error>> {
		let hook = unsafe {
			SetWinEventHook(
				EVENT_SYSTEM_FOREGROUND,
				EVENT_SYSTEM_FOREGROUND,
				ptr::null_mut(),
				Some(callback),
				0,
				0,
				WINEVENT_OUTOFCONTEXT,
			)
		};

		if hook.is_null() {
			return Err("Failed to set event hook.".into());
		}

		let instance = WndTracker {
			hook: Mutex::new(hook),
			active_wnds: Mutex::new(HashMap::new()),
			desktop: Mutex::new(Some(0)),
		};

		Ok(instance)
	}

	fn set_active_wnd(&self, hwnd: HWND) {
		let desktop = self.desktop.lock().expect("Unable to take 'desktop' ref.");

		println!("event desktop: {:?} {}", desktop, format_hwnd(&hwnd));

		if let Some(desktop) = desktop.as_ref() {
			let mut active_wnds = self
				.active_wnds
				.lock()
				.expect("Unable to take 'active_wnds' ref.");

			if !active_wnds.contains_key(desktop) {
				active_wnds.insert(*desktop, DedupRingStack::new(4));
			}

			active_wnds
				.get_mut(desktop)
				.expect("Stack not found for desktop")
				.push(hwnd);

			println!("set desktop: {:?} {}", desktop, format_hwnd(&hwnd));
		}
	}

	pub fn activate(&self, desktop: u32) {
		if let Some(ring) = self
			.active_wnds
			.lock()
			.expect("Lock cannot be taken.")
			.get(&desktop)
		{
			println!("Jump to {}", desktop);

            if let Some (hwnd) = ring.peek() {
                wnd_act_winapi::activate_window(*hwnd);
                println!("Activate {}", format_hwnd(hwnd));
            }
		} 

		let mut prev_desktop =
			self.desktop.lock().expect("Unable to take 'desktop' ref.");
		*prev_desktop = Some(desktop);
	}
}

impl Drop for WndTracker {
	fn drop(&mut self) {
		let hook = self.hook.lock().expect("Lock cannot be taken");
		if hook.is_null() {
			return;
		}

		unsafe {
			UnhookWinEvent(*hook);
		}
	}
}

unsafe extern "system" fn callback(
	_hwin_event_hook: HWINEVENTHOOK,
	event: DWORD,
	hwnd: HWND,
	_id_object: LONG,
	_id_child: LONG,
	_dw_event_thread: DWORD,
	_dwms_event_time: DWORD,
) {
	if event == EVENT_SYSTEM_FOREGROUND && !hwnd.is_null() {
		let tracker = wnd_tracker();
		tracker.set_active_wnd(hwnd);
	}
}

fn format_hwnd(hwnd: &HWND) -> String {
	let window_title = unsafe { get_wnd_title(*hwnd) };

	format!("hanle: {:p} title: {}", *hwnd, window_title)
}

unsafe fn get_wnd_title(hwnd: HWND) -> String {
	let mut buffer: [u16; 256] = [0; 256];
	let len = GetWindowTextW(hwnd, buffer.as_mut_ptr(), buffer.len() as i32);
	if len > 0 {
		let window_title = OsString::from_wide(&buffer[..len as usize])
			.to_string_lossy()
			.to_string();

		return window_title;
	}

	return "Unknown".to_string();
}

unsafe impl Send for WndTracker {}
unsafe impl Sync for WndTracker {}

struct DedupRingStack<T: PartialEq> {
	data: Vec<T>,
	head: usize,
	tail: usize,
	size: usize,
}

impl<T: PartialEq> DedupRingStack<T> {
	fn new(size: usize) -> DedupRingStack<T> {
		DedupRingStack {
			data: Vec::with_capacity(size),
			head: 0,
			tail: 0,
			size: size,
		}
	}

	fn push(&mut self, item: T) {
		if let Some(idx) = self.data.iter().position(|m| m == &item) {
			self.data.remove(idx);
		}

		self.data.push(item);
	}

	fn peek(&self) -> Option<&T> {
		self.data.first()
	}
}
