// main.rs — Tauri application entrypoint.
//
// This file is the binary entrypoint only; all application logic lives in
// lib.rs so it is accessible to tests and to the Tauri plugin runner.
//
// Tauri 2 requires the entrypoint to call `hub_tauri_lib::run()`.

// Prevent a console window from appearing on Windows in release builds.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    hub_tauri_lib::run();
}
