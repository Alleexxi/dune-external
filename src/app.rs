use std::{default, slice::SliceIndex};

use memflex::external::OwnedProcess;
use sysinfo::System;

use crate::unreal::{
    global::{set_gnames, set_process},
    offsets::{GNAMES, GOBJECTS, UWORLD},
    update_gobjects,
};

fn return_pid() -> u32 {
    let mut target_process_id = u32::MAX;
    // Uhhhh, add windows too. TODO
    let mut sys = System::new_all();
    sys.refresh_all();

    let target = "DuneSandbox-Win64-Shipping.exe";

    use std::ffi::OsStr;
    for (pid, process) in sys.processes() {
        let cmdline = process.cmd().join(OsStr::new(" "));
        if cmdline.to_string_lossy().contains(target)
            && process.name().to_str().unwrap_or_default() == "GameThread"
        {
            if target_process_id > pid.as_u32() {
                target_process_id = pid.as_u32();
            }
        }
    }

    return target_process_id;
}
#[flamer::flame]
#[derive(Debug)]
pub struct App {
    pub init: bool,
    pub pid: u32,

    pub process: OwnedProcess,

    // Addresses maybe?
    pub uworld: usize,
    pub gnames: usize,
    pub playercam: usize,
    pub gamestate: usize,
    pub persistantlevel: usize,
    pub actors: Vec<usize>,

    pub frames: u32,
}

impl App {
    pub fn init() -> Self {
        let process_id = return_pid();
        let base_address = 0x140000000;

        let proc =
            memflex::external::find_process_by_id(process_id).expect("Error opening handle?");
        let gproc =
            memflex::external::find_process_by_id(process_id).expect("Error opening handle?");
        set_process(gproc);

        // Set UWorld Global
        let uworld_address: usize = proc
            .read(base_address + UWORLD)
            .expect("Error on getting the UWorld");

        let gnames_address = base_address + GNAMES;
        set_gnames(gnames_address);

        let _ = update_gobjects();

        Self {
            pid: process_id,
            init: false,
            process: proc,

            uworld: uworld_address,
            gnames: gnames_address,
            playercam: 0x0,
            gamestate: 0x0,
            persistantlevel: 0x0,

            actors: Vec::default(),
            frames: 0,
        }
    }
}
