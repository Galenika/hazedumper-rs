extern crate num;
extern crate winapi;

use self::winapi::shared::minwindef::TRUE;
use self::winapi::um::tlhelp32::{MODULEENTRY32W,
                                 Module32FirstW,
                                 Module32NextW,
                                 TH32CS_SNAPMODULE,
                                 TH32CS_SNAPMODULE32};
use memlib::*;
use std::mem;

#[derive(Debug, Clone)]
pub struct Module {
    pub name: String,
    pub base: usize,
    pub size: usize,
    pub data: Vec<u8>,
}

/// Enum for the different signature modes:
///
/// - `Nop`: No operation
/// - `Read`: Read address
/// - `Substract`: Subtract base address
/// - `ReadSubtract`: Read and subtract base address
#[derive(Debug, Clone, PartialEq)]
pub enum Mode {
    Nop,
    Read,
    Subtract,
    ReadSubtract,
}

impl Constructor for MODULEENTRY32W {
    /// Create a new instance of `MODULEENTRY32W`
    fn new() -> Self {
        let mut module: MODULEENTRY32W = unsafe { mem::zeroed() };
        module.dwSize = mem::size_of::<MODULEENTRY32W>() as u32;
        module
    }
}

impl Module {
    fn from_module_entry(me: &MODULEENTRY32W, name: &str, process: &Process) -> Option<Self> {
        let mut i = Module {
            name: name.to_string(),
            base: me.modBaseAddr as usize,
            size: me.modBaseSize as usize,
            data: vec![0u8; me.modBaseSize as usize],
        };

        if process.read_ptr(i.data.as_mut_ptr(), i.base, i.size) {
            return Some(i);
        }

        None
    }

    pub fn find_pattern(&self, pattern: &str) -> Option<usize> {
        findpattern::find_pattern(&self.data, pattern)
    }
}

/// Wrapper around the `Module32FirstW` windows api
fn module32_first(h: &SnapshotHandle, me: &mut MODULEENTRY32W) -> bool {
    unsafe { Module32FirstW(**h, me) == TRUE }
}

/// Wrapper around the `Module32NextW` windows api
fn module32_next(h: &SnapshotHandle, me: &mut MODULEENTRY32W) -> bool {
    unsafe { Module32NextW(**h, me) == TRUE }
}

pub fn get(name: &str, process: &Process) -> Option<Module> {
    let snapshot = SnapshotHandle::new(process.id, TH32CS_SNAPMODULE | TH32CS_SNAPMODULE32)?;
    let mut me = MODULEENTRY32W::new();

    if !module32_first(&snapshot, &mut me) {
        return None;
    }

    loop {
        let s = String::from_utf16_lossy(&me.szModule)
            .trim_matches('\0')
            .to_string();

        if name == s {
            return Module::from_module_entry(&me, &s, process);
        }

        if !module32_next(&snapshot, &mut me) {
            break;
        }
    }

    None
}