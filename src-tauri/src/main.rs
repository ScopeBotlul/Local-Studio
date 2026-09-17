#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    if std::env::args_os().nth(1).as_deref() == Some(std::ffi::OsStr::new("--worker")) {
        std::process::exit(local_studio_lib::run_worker());
    }
    local_studio_lib::run();
}
