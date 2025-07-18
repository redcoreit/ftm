/* SPDX-FileCopyrightText: © 2023 Nadim Kobeissi <nadim@symbolic.software>
 * SPDX-FileCopyrightText: © 2025 Roland Halbaksz
 * SPDX-License-Identifier: MIT */

use std::{fs::OpenOptions, os::windows::io::AsRawHandle};

use windows::Win32::{
	Foundation::HANDLE,
	System::Console::{
		AllocConsole, FreeConsole, SetStdHandle, STD_ERROR_HANDLE, STD_OUTPUT_HANDLE,
	},
};

pub fn init() {
	unsafe {
		let pid = std::process::id();
		let exec_info = super::proc::get_exec_info(pid);
		let is_shell_executed = exec_info
			.as_ref()
			.map_or(false, |info| info.is_shell_executed);

		if !is_shell_executed {
			return;
		}

		// No console attached, allocate one

		// Establish this app as foreground capable application so it can use SetForegroundWindow
		// Create gui console and immediately close it
		let _ = AllocConsole();
		let _ = FreeConsole();

		// we don't have console hence redirect stdio to a file
		redirect_stdio();

		// TODO: log in debug builds the parent process name
	}
}

fn redirect_stdio() {
	let file = OpenOptions::new()
		.create(true)
		.append(true)
		.open("stdio.log")
		.expect("Failed to open stderr.log");

	let handle = HANDLE(file.as_raw_handle());
	std::mem::forget(file);

	unsafe {
		if SetStdHandle(STD_OUTPUT_HANDLE, handle).is_err() {
			panic!("Failed to set stdout handle");
		}

		if SetStdHandle(STD_ERROR_HANDLE, handle).is_err() {
			panic!("Failed to set stderr handle");
		}
	}
}
