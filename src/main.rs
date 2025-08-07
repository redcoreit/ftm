/* SPDX-FileCopyrightText: © 2023 Nadim Kobeissi <nadim@symbolic.software>
 * SPDX-License-Identifier: MIT */

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod binkybox;
mod win32;
mod ftm;

#[tokio::main]
async fn main() {
    win32::console::init();

    println!("Stdout test");
    eprintln!("Stderr test");
    // panic!("Panic test");

	tokio::spawn(binkybox::init_keys());
	binkybox::init_tray();
}
