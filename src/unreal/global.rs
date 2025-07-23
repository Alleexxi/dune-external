use memflex::external::OwnedProcess;
use once_cell::sync::{Lazy, OnceCell};
use std::{cell::RefCell, sync::Mutex};

use crate::unreal::types::structs::FUObjectItem;

// Process Handle
static GLOBAL_PROCESS: OnceCell<OwnedProcess> = OnceCell::new();
pub fn set_process(proc: OwnedProcess) {
    GLOBAL_PROCESS.set(proc).unwrap();
}

pub fn get_process() -> &'static OwnedProcess {
    GLOBAL_PROCESS.get().expect("OwningProcess not set")
}

// GNames
static GLOBAL_GNAMES: OnceCell<usize> = OnceCell::new();
pub fn set_gnames(proc: usize) {
    GLOBAL_GNAMES.set(proc).unwrap();
}

pub fn get_gnames() -> &'static usize {
    GLOBAL_GNAMES.get().expect("GLOBAL_GNAMES not set")
}

// UWorld
static GLOBAL_UWORLD: OnceCell<usize> = OnceCell::new();
pub fn set_uworld(proc: usize) {
    GLOBAL_UWORLD.set(proc).unwrap();
}

pub fn get_uworld() -> &'static usize {
    GLOBAL_UWORLD.get().expect("OwningProcess not set")
}

// GObjects
static GLOBAL_GOBJECTS: OnceCell<Mutex<Vec<FUObjectItem>>> = OnceCell::new();

pub fn set_gobjects(gobjects: Vec<FUObjectItem>) {
    let gobjects_mutex = GLOBAL_GOBJECTS.get_or_init(|| Mutex::new(Vec::new()));
    {
        let mut guard = gobjects_mutex.lock().unwrap();
        *guard = gobjects;
    } // Unlocks here when guard goes out of scope
}

pub fn get_gobjects() -> Vec<FUObjectItem> {
    GLOBAL_GOBJECTS
        .get()
        .expect("GObjects not set")
        .lock()
        .expect("Failed to lock GObjects mutex")
        .clone() // Clone the Vec so we don't hold the lock
}

pub static STRING_CACHE: Lazy<Mutex<Vec<Option<String>>>> = Lazy::new(|| Mutex::new(Vec::new()));
