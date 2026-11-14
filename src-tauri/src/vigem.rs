//! Minimal ViGEmBus client binding via dynamic loading of ViGEmClient.dll.
//!
//! We deliberately avoid a wrapper crate so the ABI surface stays explicit and
//! small. Only the Xbox 360 target is wired up.

use std::ffi::c_void;
use std::path::PathBuf;

use libloading::{Library, Symbol};

pub const VIGEM_ERROR_NONE: u32 = 0x2000_0000;

#[repr(C)]
#[derive(Clone, Copy, Default)]
pub struct XusbReport {
    pub w_buttons: u16,
    pub b_left_trigger: u8,
    pub b_right_trigger: u8,
    pub s_thumb_lx: i16,
    pub s_thumb_ly: i16,
    pub s_thumb_rx: i16,
    pub s_thumb_ry: i16,
}

pub mod xusb {
    pub const DPAD_UP: u16 = 0x0001;
    pub const DPAD_DOWN: u16 = 0x0002;
    pub const DPAD_LEFT: u16 = 0x0004;
    pub const DPAD_RIGHT: u16 = 0x0008;
    pub const START: u16 = 0x0010;
    pub const BACK: u16 = 0x0020;
    pub const LEFT_THUMB: u16 = 0x0040;
    pub const RIGHT_THUMB: u16 = 0x0080;
    pub const LEFT_SHOULDER: u16 = 0x0100;
    pub const RIGHT_SHOULDER: u16 = 0x0200;
    pub const GUIDE: u16 = 0x0400;
    pub const A: u16 = 0x1000;
    pub const B: u16 = 0x2000;
    pub const X: u16 = 0x4000;
    pub const Y: u16 = 0x8000;
}

/// Map an internal canonical button code to its XUSB bit.
pub fn xusb_flag(code: &str) -> u16 {
    match code {
        "BTN_DPAD_UP" => xusb::DPAD_UP,
        "BTN_DPAD_DOWN" => xusb::DPAD_DOWN,
        "BTN_DPAD_LEFT" => xusb::DPAD_LEFT,
        "BTN_DPAD_RIGHT" => xusb::DPAD_RIGHT,
        "BTN_START" => xusb::START,
        "BTN_BACK" => xusb::BACK,
        "BTN_LTHUMB" => xusb::LEFT_THUMB,
        "BTN_RTHUMB" => xusb::RIGHT_THUMB,
        "BTN_LB" => xusb::LEFT_SHOULDER,
        "BTN_RB" => xusb::RIGHT_SHOULDER,
        "BTN_GUIDE" => xusb::GUIDE,
        "BTN_A" => xusb::A,
        "BTN_B" => xusb::B,
        "BTN_X" => xusb::X,
        "BTN_Y" => xusb::Y,
        _ => 0,
    }
}

type FnAlloc = unsafe extern "C" fn() -> *mut c_void;
type FnConnect = unsafe extern "C" fn(*mut c_void) -> u32;
type FnDisconnect = unsafe extern "C" fn(*mut c_void);
type FnFree = unsafe extern "C" fn(*mut c_void);
type FnTargetAlloc = unsafe extern "C" fn() -> *mut c_void;
type FnTargetFree = unsafe extern "C" fn(*mut c_void);
type FnTargetAdd = unsafe extern "C" fn(*mut c_void, *mut c_void) -> u32;
type FnTargetRemove = unsafe extern "C" fn(*mut c_void, *mut c_void) -> u32;
type FnX360Update = unsafe extern "C" fn(*mut c_void, *mut c_void, XusbReport) -> u32;

pub struct VigemApi {
    _lib: Library,
    vigem_alloc: FnAlloc,
    vigem_free: FnFree,
    vigem_connect: FnConnect,
    vigem_disconnect: FnDisconnect,
    target_alloc: FnTargetAlloc,
    target_free: FnTargetFree,
    target_add: FnTargetAdd,
    target_remove: FnTargetRemove,
    x360_update: FnX360Update,
}

fn candidate_paths() -> Vec<PathBuf> {
    let mut out = Vec::new();
    if let Ok(exe) = std::env::current_exe() {
        if let Some(dir) = exe.parent() {
            out.push(dir.join("libs").join("ViGEmClient.dll"));
            out.push(dir.join("ViGEmClient.dll"));
            if let Some(res) = dir.parent() {
                out.push(res.join("libs").join("ViGEmClient.dll"));
            }
        }
    }
    if let Ok(system_root) = std::env::var("SystemRoot") {
        out.push(PathBuf::from(system_root).join("System32").join("ViGEmClient.dll"));
    }
    out.push(PathBuf::from("C:\\Windows\\System32\\ViGEmClient.dll"));
    out
}

impl VigemApi {
    /// Load the client library and bind symbols.
    pub fn load() -> Result<Self, String> {
        // Prefer the current working directory / exe dir, then System32.
        let mut last_err = String::from("ViGEmClient.dll not found");

        for path in candidate_paths() {
            if !path.exists() {
                continue;
            }
            match unsafe { Library::new(&path) } {
                Ok(lib) => match Self::bind(lib, &path) {
                    Ok(api) => return Ok(api),
                    Err(e) => last_err = e,
                },
                Err(e) => last_err = format!("failed to load {}: {}", path.display(), e),
            }
        }

        // Last resort: let the OS resolve it through PATH.
        match unsafe { Library::new("ViGEmClient.dll") } {
            Ok(lib) => Self::bind(lib, PathBuf::from("ViGEmClient.dll").as_path()),
            Err(_) => Err(last_err),
        }
    }

    fn bind(lib: Library, path: &std::path::Path) -> Result<Self, String> {
        unsafe {
            macro_rules! sym {
                ($name:literal, $ty:ty) => {{
                    let s: Symbol<$ty> = lib.get($name)
                        .map_err(|e| {
                            format!("{} missing symbol: {}", path.display(), e)
                        })?;
                    *s
                }};
            }

            Ok(VigemApi {
                vigem_alloc: sym!(b"vigem_alloc\0", FnAlloc),
                vigem_free: sym!(b"vigem_free\0", FnFree),
                vigem_connect: sym!(b"vigem_connect\0", FnConnect),
                vigem_disconnect: sym!(b"vigem_disconnect\0", FnDisconnect),
                target_alloc: sym!(b"vigem_target_x360_alloc\0", FnTargetAlloc),
                target_free: sym!(b"vigem_target_free\0", FnTargetFree),
                target_add: sym!(b"vigem_target_add\0", FnTargetAdd),
                target_remove: sym!(b"vigem_target_remove\0", FnTargetRemove),
                x360_update: sym!(b"vigem_target_x360_update\0", FnX360Update),
                _lib: lib,
            })
        }
    }
}

/// One plugged virtual Xbox 360 pad.
pub struct VirtualPad {
    api: std::sync::Arc<VigemApi>,
    client: *mut c_void,
    target: *mut c_void,
}

unsafe impl Send for VirtualPad {}

impl VirtualPad {
    pub fn connect(api: std::sync::Arc<VigemApi>) -> Result<Self, String> {
        unsafe {
            let client = (api.vigem_alloc)();
            if client.is_null() {
                return Err("vigem_alloc returned null".into());
            }

            let err = (api.vigem_connect)(client);
            if err != VIGEM_ERROR_NONE {
                (api.vigem_free)(client);
                return Err(format!(
                    "ViGEmBus connect failed (0x{:X}). Is the ViGEmBus driver installed and running?",
                    err
                ));
            }

            let target = (api.target_alloc)();
            if target.is_null() {
                (api.vigem_disconnect)(client);
                (api.vigem_free)(client);
                return Err("vigem_target_x360_alloc returned null".into());
            }

            let err = (api.target_add)(client, target);
            if err != VIGEM_ERROR_NONE {
                (api.target_free)(target);
                (api.vigem_disconnect)(client);
                (api.vigem_free)(client);
                return Err(format!("ViGEmBus target_add failed (0x{:X})", err));
            }

            Ok(VirtualPad { api, client, target })
        }
    }

    pub fn update(&self, report: &XusbReport) -> Result<(), String> {
        unsafe {
            let err = (self.api.x360_update)(self.client, self.target, *report);
            if err != VIGEM_ERROR_NONE {
                return Err(format!("x360_update failed (0x{:X})", err));
            }
            Ok(())
        }
    }

    pub fn reset(&self) {
        let _ = self.update(&XusbReport::default());
    }
}

impl Drop for VirtualPad {
    fn drop(&mut self) {
        unsafe {
            if !self.target.is_null() {
                (self.api.target_remove)(self.client, self.target);
                (self.api.target_free)(self.target);
                self.target = std::ptr::null_mut();
            }
            if !self.client.is_null() {
                (self.api.vigem_disconnect)(self.client);
                (self.api.vigem_free)(self.client);
                self.client = std::ptr::null_mut();
            }
        }
    }
}

// The bound function pointers are plain C ABI symbols; the underlying library
// handle is reference counted by the OS loader.
unsafe impl Send for VigemApi {}
unsafe impl Sync for VigemApi {}
