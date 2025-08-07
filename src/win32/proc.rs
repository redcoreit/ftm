/* SPDX-FileCopyrightText: © 2025 Roland Halbaksz
 * SPDX-License-Identifier: MIT */

use std::{ffi::OsString, io::BufRead, os::windows::{ffi::OsStringExt, process::CommandExt}, path::PathBuf};

use windows::{core::PWSTR, Win32::{Foundation::{CloseHandle, HWND}, System::Threading::{OpenProcess, QueryFullProcessImageNameW, PROCESS_NAME_FORMAT, PROCESS_QUERY_LIMITED_INFORMATION}, UI::WindowsAndMessaging::GetWindowThreadProcessId}};

const CREATE_NO_WINDOW: u32 = 0x08000000;

#[allow(dead_code)]
pub struct ProcExecInfo {
	pub pid: u32,
	pub parent_pid: u32,
	pub parent_pname: String,
	pub is_shell_executed: bool,
}

pub fn get_exec_info(pid: u32) -> Option<ProcExecInfo> {
	let parent_pid = get_parent_pid()?;
	let parent_pname = get_parent_pname()?;
	let parent_pname = match parent_pname.into_string() {
		Ok(name) => name.to_lowercase(),
		Err(_) => return None,
	};

	let is_shell_executed = is_shell_executed(&parent_pname);

	Some(ProcExecInfo {
		pid,
		parent_pid,
		parent_pname,
		is_shell_executed,
	})
}

pub fn get_proc_hwnd_pname(hwnd: HWND) -> Option<String> {
    let exe_path = get_proc_hwnd_ppath(hwnd);

    if exe_path.is_none() {
        return None
    }

    let file = PathBuf::from(exe_path.unwrap());

    if !file.is_file()
    {
        return None
    }

    file.file_stem().map(|s| s.to_string_lossy().into_owned())
}

pub fn get_proc_hwnd_ppath(hwnd: HWND) -> Option<String> {
    let mut pid = 0;
    unsafe {
        GetWindowThreadProcessId(hwnd, Some(&mut pid));
        if pid == 0 {
            return None;
        }

        let h_process = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, false, pid);
        if h_process.is_err() {
            return None;
        }

        let h_process = h_process.unwrap();

        let mut buf = vec![0u16; 260];
        let mut size = buf.len() as u32;

        let success = QueryFullProcessImageNameW(
            h_process,
            PROCESS_NAME_FORMAT(0),
            PWSTR::from_raw(buf.as_mut_ptr()),
            &mut size,
        )
        .is_ok();

        _ = CloseHandle(h_process);

        if success {
            buf.truncate(size as usize);
            let path = OsString::from_wide(&buf);
            return path.to_str().map(|s| s.to_string());
        }
    }

    None
}

fn is_shell_executed(filename: &str) -> bool {
	// TODO: make this list configurable
	let filter = [
		"cmd",
		"pwsh",
		"cargo",
		"conhost",
		"powershell",
		"wezterm-gui",
	];

	for item in filter.iter() {
		if filename.eq_ignore_ascii_case(*item) {
			return false;
		}
	}

	return true;
}

fn get_parent_pname() -> Option<OsString> {
	let parent_pid = get_parent_pid()?;

	let cmd = format!(
		"wmic process where ProcessId={} get ExecutablePath",
		parent_pid
	);

	let output = std::process::Command::new("cmd")
		.args(["/C", &cmd])
		.creation_flags(CREATE_NO_WINDOW)
		.output()
		.ok()?;

	let line = output.stdout.lines().nth(1)?.ok()?;

	let line = line.trim().to_lowercase();
	let path: PathBuf = line.into();

	if !path.is_file() {
		return None;
	}

	Some(path.file_stem()?.to_owned())
}

fn get_parent_pid() -> Option<u32> {
	let pid = std::process::id();
	let cmd = format!("wmic process where ProcessId={} get ParentProcessId", pid);

	let output = std::process::Command::new("cmd")
		.args(["/C", &cmd])
		.creation_flags(CREATE_NO_WINDOW)
		.output()
		.ok()?;

	let line = output.stdout.lines().nth(1)?.ok()?;

	let line = line.trim();
	let parent_pid = line.parse::<u32>();

	if parent_pid.is_ok() {
		Some(parent_pid.unwrap())
	} else {
		None
	}
}
