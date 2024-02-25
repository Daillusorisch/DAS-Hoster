// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    match das_lib::run() {
        Ok(_) => println!("ok"),
        Err(e) => eprintln!("{:#?}", e),
    }
}
