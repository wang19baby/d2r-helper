// Windows: hide console window in release builds
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    d2r_marketplace_lib::run();
}