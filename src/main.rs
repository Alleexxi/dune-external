#![feature(proc_macro_hygiene)]

use libc::exit;
#[allow(non_camel_case_types)]
#[allow(nonstandard_style)]
use std::fs::File;

#[flamer::flame]
mod app;
#[flamer::flame]
mod ui;
#[flamer::flame]
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
            if line.contains(" /proc ") {
                if line.contains("hidepid=invisible") {
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
