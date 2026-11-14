use std::collections::{HashMap, HashSet};
use std::sync::mpsc::{channel, Receiver, Sender};
use std::sync::Arc;
use std::time::{Duration, Instant};

use gilrs::{Axis, Button, GamepadId, Gilrs};
use parking_lot::Mutex;
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter};

use crate::mapping::{build_report, PadState, TurboState};
use crate::profile::Profile;
use crate::vigem::{VigemApi, VirtualPad};

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PadSnapshot {
    pub id: usize,
    pub axes: HashMap<String, f32>,
    pub buttons: HashMap<String, bool>,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ControllerInfo {
    pub id: usize,
    pub name: String,
    pub uuid: String,
    pub connected: bool,
    pub battery: Option<f32>,
    pub axes: Vec<String>,
    pub buttons: Vec<String>,
    pub is_gamepad: bool,
}

#[derive(Clone, Debug, Serialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct MappingStatus {
    pub emulating: bool,
    pub virtual_connected: bool,
    pub driver_available: bool,
    pub error: Option<String>,
}

pub enum Command {
    Start { id: usize, uuid: String, profile: Profile },
    Stop { id: usize },
    Rescan,
    Shutdown,
}

pub struct Shared {
    pub snapshots: Mutex<Vec<PadSnapshot>>,
    pub controllers: Mutex<Vec<ControllerInfo>>,
    pub profiles: Mutex<HashMap<usize, Profile>>,
    pub uuids: Mutex<HashMap<usize, String>>,
    pub emulating: Mutex<HashSet<usize>>,
    pub status: Mutex<MappingStatus>,
    pub api: OnceApi,
}

pub struct OnceApi(pub Mutex<Option<Arc<VigemApi>>>);

impl OnceApi {
    pub fn new() -> Self {
        OnceApi(Mutex::new(None))
    }
    pub fn get(&self) -> Option<Arc<VigemApi>> {
        self.0.lock().clone()
    }
}

impl Shared {
    pub fn new() -> Self {
        Shared {
            snapshots: Mutex::new(Vec::new()),
            controllers: Mutex::new(Vec::new()),
            profiles: Mutex::new(HashMap::new()),
            uuids: Mutex::new(HashMap::new()),
            emulating: Mutex::new(HashSet::new()),
            status: Mutex::new(MappingStatus::default()),
            api: OnceApi::new(),
        }
    }
}

impl Default for Shared {
    fn default() -> Self {
        Self::new()
    }
}

pub fn channel_pair() -> (Sender<Command>, Receiver<Command>) {
    channel()
}

// ---------------- WinMM Native Windows Joystick API ----------------
#[repr(C)]
struct JoyCapsW {
    w_mid: u16,
    w_pid: u16,
    sz_pname: [u16; 32],
    w_xmin: u32,
    w_xmax: u32,
    w_ymin: u32,
    w_ymax: u32,
    w_zmin: u32,
    w_zmax: u32,
    w_num_buttons: u32,
    w_period_min: u32,
    w_period_max: u32,
    w_rmin: u32,
    w_rmax: u32,
    w_umin: u32,
    w_umax: u32,
    w_vmin: u32,
    w_vmax: u32,
    w_caps: u32,
    w_max_axes: u32,
    w_num_axes: u32,
    w_max_buttons: u32,
    sz_reg_key: [u16; 32],
    sz_oem_vx_d: [u16; 260],
}

#[repr(C)]
struct JoyInfoEx {
    dw_size: u32,
    dw_flags: u32,
    dw_xpos: u32,
    dw_ypos: u32,
    dw_zpos: u32,
    dw_rpos: u32,
    dw_upos: u32,
    dw_vpos: u32,
    dw_buttons: u32,
    dw_button_number: u32,
    dw_pov: u32,
    dw_reserved1: u32,
    dw_reserved2: u32,
}

#[link(name = "winmm")]
extern "system" {
    fn joyGetDevCapsW(u_joy_id: usize, pjc: *mut JoyCapsW, cbjc: u32) -> u32;
    fn joyGetPosEx(u_joy_id: usize, pji: *mut JoyInfoEx) -> u32;
}

const BUTTON_MAP: [(Button, &str); 17] = [
    (Button::South, "BTN_A"),
    (Button::East, "BTN_B"),
    (Button::North, "BTN_Y"),
    (Button::West, "BTN_X"),
    (Button::LeftTrigger, "BTN_LB"),
    (Button::RightTrigger, "BTN_RB"),
    (Button::LeftTrigger2, "BTN_LT"),
    (Button::RightTrigger2, "BTN_RT"),
    (Button::Select, "BTN_BACK"),
    (Button::Start, "BTN_START"),
    (Button::Mode, "BTN_GUIDE"),
    (Button::LeftThumb, "BTN_LTHUMB"),
    (Button::RightThumb, "BTN_RTHUMB"),
    (Button::DPadUp, "BTN_DPAD_UP"),
    (Button::DPadDown, "BTN_DPAD_DOWN"),
    (Button::DPadLeft, "BTN_DPAD_LEFT"),
    (Button::DPadRight, "BTN_DPAD_RIGHT"),
];

const AXIS_MAP: [(Axis, &str); 6] = [
    (Axis::LeftStickX, "AXIS_LX"),
    (Axis::LeftStickY, "AXIS_LY"),
    (Axis::RightStickX, "AXIS_RX"),
    (Axis::RightStickY, "AXIS_RY"),
    (Axis::LeftZ, "AXIS_LT"),
    (Axis::RightZ, "AXIS_RT"),
];

struct PendingDetect {
    before: HashSet<GamepadId>,
    started: Instant,
}

pub fn spawn(app: AppHandle, shared: Arc<Shared>, rx: Receiver<Command>) {
    std::thread::Builder::new()
        .name("ghostpad-input".into())
        .spawn(move || run(app, shared, rx))
        .expect("failed to spawn input thread");
}

fn is_phys_down(map: &HashMap<String, bool>, key: &str) -> bool {
    if *map.get(key).unwrap_or(&false) {
        return true;
    }
    if let (Some(s), Some(e)) = (key.find('('), key.find(')')) {
        if s < e {
            let num = &key[s + 1..e];
            if *map.get(num).unwrap_or(&false) {
                return true;
            }
        }
    }
    if key.chars().all(|c| c.is_ascii_digit()) {
        let bkey = format!("Button({key})");
        if *map.get(&bkey).unwrap_or(&false) {
            return true;
        }
    }
    false
}

fn norm_axis(val: u32) -> f32 {
    ((val as f32 - 32767.5) / 32767.5).clamp(-1.0, 1.0)
}

fn run(app: AppHandle, shared: Arc<Shared>, rx: Receiver<Command>) {
    let mut gilrs = Gilrs::new().ok();

    let mut pads: HashMap<usize, VirtualPad> = HashMap::new();
    let mut turbo: HashMap<usize, TurboState> = HashMap::new();
    let mut virtual_ids: HashSet<GamepadId> = HashSet::new();
    let mut pending: Option<PendingDetect> = None;
    let mut last_ids: Vec<usize> = Vec::new();
    let mut last_emit = Instant::now();

    loop {
        // 1. commands
        while let Ok(cmd) = rx.try_recv() {
            match cmd {
                Command::Start { id, uuid, profile } => {
                    shared.profiles.lock().insert(id, profile);
                    shared.uuids.lock().insert(id, uuid);
                    shared.emulating.lock().insert(id);

                    if let Some(g) = &gilrs {
                        let before: HashSet<GamepadId> = g.gamepads().map(|(gid, _)| gid).collect();
                        pending = Some(PendingDetect { before, started: Instant::now() });
                    }

                    let mut st = shared.status.lock();
                    st.error = None;
                }
                Command::Stop { id } => {
                    shared.emulating.lock().remove(&id);
                    if let Some(pad) = pads.remove(&id) {
                        pad.reset();
                    }
                    turbo.remove(&id);
                }
                Command::Rescan => {
                    last_ids.clear();
                }
                Command::Shutdown => return,
            }
        }

        // 2. build snapshots from WinMM (primary for generic USB sticks) + Gilrs (for Xbox/Bluetooth pads)
        let mut snaps: Vec<PadSnapshot> = Vec::new();
        let mut infos: Vec<ControllerInfo> = Vec::new();

        // --- WinMM scan ---
        for joy_id in 0..8 {
            let mut caps: JoyCapsW = unsafe { std::mem::zeroed() };
            if unsafe { joyGetDevCapsW(joy_id, &mut caps, std::mem::size_of::<JoyCapsW>() as u32) } != 0 {
                continue;
            }

            let mut info: JoyInfoEx = unsafe { std::mem::zeroed() };
            info.dw_size = std::mem::size_of::<JoyInfoEx>() as u32;
            info.dw_flags = 0xFF; // JOY_RETURNALL
            if unsafe { joyGetPosEx(joy_id, &mut info) } != 0 {
                continue;
            }

            // Skip the virtual Xbox 360 pad we create ourselves (and any other
            // Xbox-class device) so emulation doesn't feed back into itself.
            let raw_name_probe = String::from_utf16_lossy(&caps.sz_pname);
            let name_probe = raw_name_probe.to_lowercase();
            let is_xbox360 = caps.w_mid == 0x045e && caps.w_pid == 0x028e;
            if name_probe.contains("xbox")
                || name_probe.contains("360")
                || name_probe.contains("vigem")
                || name_probe.contains("virtual")
                || is_xbox360
            {
                continue;
            }

            let device_id = 100 + joy_id;
            let num_buttons = caps.w_num_buttons.clamp(8, 32);

            let mut buttons = HashMap::new();
            let mut button_names = Vec::new();
            for b in 0..num_buttons {
                let code = format!("Button({b})");
                let is_down = (info.dw_buttons & (1 << b)) != 0;
                buttons.insert(code.clone(), is_down);
                button_names.push(code);
            }

            // D-Pad decoding from POV (Hat switch)
            // Windows WinMM reports 65535 (0xFFFF) when hat is centered / released.
            // 0 is only UP if pov <= 35900!
            let pov = info.dw_pov;
            let has_pov = pov != 65535 && pov <= 35900;
            let dpad_up = has_pov && (pov == 0 || pov == 4500 || pov == 31500);
            let dpad_down = has_pov && (pov == 18000 || pov == 13500 || pov == 22500);
            let dpad_left = has_pov && (pov == 27000 || pov == 22500 || pov == 31500);
            let dpad_right = has_pov && (pov == 9000 || pov == 4500 || pov == 13500);

            buttons.insert("RAW_POV_UP".to_string(), dpad_up);
            buttons.insert("RAW_POV_DOWN".to_string(), dpad_down);
            buttons.insert("RAW_POV_LEFT".to_string(), dpad_left);
            buttons.insert("RAW_POV_RIGHT".to_string(), dpad_right);

            // Axes
            let mut axes = HashMap::new();
            let lx = norm_axis(info.dw_xpos);
            let ly = -norm_axis(info.dw_ypos);
            let rx = norm_axis(info.dw_zpos);
            let ry = -norm_axis(info.dw_rpos);

            axes.insert("AXIS_LX".to_string(), lx);
            axes.insert("AXIS_LY".to_string(), ly);
            axes.insert("AXIS_RX".to_string(), rx);
            axes.insert("AXIS_RY".to_string(), ry);
            axes.insert("Axis(0)".to_string(), lx);
            axes.insert("Axis(1)".to_string(), ly);
            axes.insert("Axis(2)".to_string(), rx);
            axes.insert("Axis(3)".to_string(), ry);

            let raw_name = String::from_utf16_lossy(&caps.sz_pname);
            let mut name = raw_name.trim_matches(char::from(0)).to_string();
            if name.is_empty() || name.contains("Microsoft PC-joystick") {
                // Best-effort brand label from VID/PID.
                let brand = match (caps.w_mid, caps.w_pid) {
                    (0x0079, 0x0006) => "M-Tech",
                    (0x0079, _) => "M-Tech/Generic",
                    _ => "Generic USB",
                };
                name = format!("{} Pad #{} ({:04x}:{:04x})", brand, joy_id + 1, caps.w_mid, caps.w_pid);
            }

            let uuid = format!("winmm_{:04x}_{:04x}_{}", caps.w_mid, caps.w_pid, joy_id);
            let axes_names = vec!["Left Stick X".into(), "Left Stick Y".into(), "Right Stick X".into(), "Right Stick Y".into()];

            snaps.push(PadSnapshot {
                id: device_id,
                axes,
                buttons,
            });
            infos.push(ControllerInfo {
                id: device_id,
                name,
                uuid,
                connected: true,
                battery: None,
                axes: axes_names,
                buttons: button_names,
                is_gamepad: true,
            });
        }

        // --- Gilrs scan (if any other controllers exist) ---
        if let Some(g) = &mut gilrs {
            while g.next_event().is_some() {}

            if let Some(det) = &pending {
                for gid in g.gamepads().map(|(gid, _)| gid) {
                    if !det.before.contains(&gid) {
                        virtual_ids.insert(gid);
                    }
                }
                if det.started.elapsed() > Duration::from_secs(3) {
                    pending = None;
                }
            }

            let mut alive: HashSet<GamepadId> = HashSet::new();
            for (gid, pad) in g.gamepads() {
                alive.insert(gid);
                if virtual_ids.contains(&gid) {
                    continue;
                }
                let raw_id: usize = gid.into();

                // Skip generic joysticks in Gilrs to prevent duplicates (already captured by WinMM)
                let name_lower = pad.name().to_lowercase();
                if name_lower.contains("generic")
                    || name_lower.contains("0079")
                    || name_lower.contains("hid-compliant")
                    || name_lower.contains("gamepad")
                    || name_lower.contains("joystick")
                {
                    continue;
                }

                let mut buttons = HashMap::new();
                let mut present_buttons = Vec::new();
                for (btn, code) in BUTTON_MAP {
                    let pressed = pad.is_pressed(btn);
                    buttons.insert(code.to_string(), pressed);
                    present_buttons.push(code.to_string());
                }

                let mut axes = HashMap::new();
                let mut present_axes = Vec::new();
                for (axis, code) in AXIS_MAP {
                    let v = pad.value(axis);
                    axes.insert(code.to_string(), v);
                    present_axes.push(code.to_string());
                }

                let name = pad.name().to_string();
                let uuid = pad.uuid().iter().map(|b| format!("{:02x}", b)).collect::<String>();

                snaps.push(PadSnapshot { id: raw_id, axes, buttons });
                infos.push(ControllerInfo {
                    id: raw_id,
                    name,
                    uuid,
                    connected: pad.is_connected(),
                    battery: None,
                    axes: present_axes,
                    buttons: present_buttons,
                    is_gamepad: true,
                });
            }
            virtual_ids.retain(|gid| alive.contains(gid));
        }

        // 3. apply mappings for any emulating device
        let emulating: Vec<usize> = shared.emulating.lock().iter().copied().collect();
        let api = shared.api.get();

        for id in emulating {
            let snap = snaps.iter().find(|s| s.id == id);
            let Some(snap) = snap else { continue };

            if !pads.contains_key(&id) {
                match &api {
                    Some(api) => match VirtualPad::connect(api.clone()) {
                        Ok(pad) => {
                            pads.insert(id, pad);
                            turbo.insert(id, TurboState::new());
                            let mut st = shared.status.lock();
                            st.virtual_connected = true;
                            st.error = None;
                        }
                        Err(e) => {
                            let mut st = shared.status.lock();
                            st.error = Some(e);
                            st.virtual_connected = false;
                            shared.emulating.lock().remove(&id);
                            continue;
                        }
                    },
                    None => {
                        let mut st = shared.status.lock();
                        st.error = Some("ViGEmClient.dll not loaded".into());
                        st.virtual_connected = false;
                        shared.emulating.lock().remove(&id);
                        continue;
                    }
                }
            }

            let profile = shared.profiles.lock().get(&id).cloned();
            let Some(profile) = profile else { continue };

            let state = PadState {
                axes: snap.axes.clone(),
                buttons: snap.buttons.clone(),
            };
            let t = turbo.entry(id).or_default();
            let report = build_report(&state, &profile, t);

            if let Some(pad) = pads.get(&id) {
                if let Err(e) = pad.update(&report) {
                    let mut st = shared.status.lock();
                    st.error = Some(e);
                    pads.remove(&id);
                }
            }
        }

        // 4. bookkeeping
        let emulating_now = !shared.emulating.lock().is_empty();
        {
            let mut st = shared.status.lock();
            st.emulating = emulating_now;
            st.virtual_connected = !pads.is_empty();
            if !emulating_now {
                st.error = None;
            }
        }

        let ids: Vec<usize> = infos.iter().map(|i| i.id).collect();
        let ids_changed = ids != last_ids;
        if ids_changed {
            last_ids = ids.clone();
            *shared.controllers.lock() = infos;
            *shared.snapshots.lock() = snaps;
            let _ = app.emit("controllers-changed", shared.controllers.lock().clone());
            let _ = app.emit("input-update", shared.snapshots.lock().clone());
            let _ = app.emit("status-changed", shared.status.lock().clone());
            last_emit = Instant::now();
        } else {
            *shared.snapshots.lock() = snaps;
            if last_emit.elapsed() >= Duration::from_millis(16) {
                last_emit = Instant::now();
                let _ = app.emit("input-update", shared.snapshots.lock().clone());
            }
        }

        std::thread::sleep(Duration::from_millis(8));
    }
}

pub fn probe_driver() -> Option<Arc<VigemApi>> {
    VigemApi::load().ok().map(Arc::new)
}
