#![feature(proc_macro_hygiene)]

use fastcontains::fastcontains;
use libc::exit;
#[allow(non_camel_case_types)]
#[allow(nonstandard_style)]
mod app;
mod ui;
mod unreal;

fn main() {
    // Check if running as root (sudo)
    if unsafe { libc::geteuid() } != 0 {
        println!("Not running with sudo.");
        println!("Please run the file as sudo.");
        unsafe { exit(0) };
    }

    if let Ok(mounts) = std::fs::read_to_string("/proc/mounts") {
        let mut found = false;
        for line in mounts.lines() {
            if line.fast_contains(" /proc ") {
                if line.fast_contains("hidepid=invisible") {
                    found = true;
                    break;
                }
            }
        }
        if !found {
            println!("Error: /proc is not mounted with hidepid=invisible.");
            println!("Please remount /proc with hidepid=invisible.");
            unsafe { exit(1) };
        }
    }

    println!("yay");

    let app = app::App::init();

    egui_overlay::start(app);
}
