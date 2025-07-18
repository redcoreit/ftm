/* SPDX-FileCopyrightText: © 2025 Roland Halbaksz
 * SPDX-License-Identifier: MIT */

use std::{ffi::OsString, io::BufRead, os::windows::process::CommandExt, path::PathBuf};

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

fn is_shell_executed(filename: &str) -> bool {
	// TODO: make this list configurable
	let filter = [
		"cmd.exe",
		"pwsh.exe",
		"cargo.exe",
		"conhost.exe",
		"powershell.exe",
	];

	for item in filter.iter() {
		if filename == *item {
			return true;
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

	Some(path.file_name()?.to_owned())
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
