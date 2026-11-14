use std::collections::HashMap;

use crate::profile::Profile;
use crate::vigem::{xusb_flag, XusbReport};

#[derive(Clone, Debug, Default, serde::Serialize)]
pub struct PadState {
    pub axes: HashMap<String, f32>,
    pub buttons: HashMap<String, bool>,
}

fn deadzone(value: f32, dz: f32) -> f32 {
    if value.abs() <= dz {
        0.0
    } else {
        let sign = if value > 0.0 { 1.0 } else { -1.0 };
        sign * ((value.abs() - dz) / (1.0 - dz)).clamp(0.0, 1.0)
    }
}

/// Normalise a raw trigger axis into 0..1.
///
/// Devices differ: some report -1..1 with -1 released, others report 0..1.
fn trigger_norm(raw: f32) -> f32 {
    if raw < -0.01 {
        ((raw + 1.0) / 2.0).clamp(0.0, 1.0)
    } else {
        raw.clamp(0.0, 1.0)
    }
}

fn to_i16(v: f32) -> i16 {
    (v.clamp(-1.0, 1.0) * 32767.0) as i16
}

pub struct TurboState {
    last: HashMap<String, u64>,
    on: HashMap<String, bool>,
}

impl TurboState {
    pub fn new() -> Self {
        TurboState {
            last: HashMap::new(),
            on: HashMap::new(),
        }
    }

    fn tick(&mut self, key: &str, pressed: bool, interval_ms: u64) -> bool {
        if !pressed {
            self.on.insert(key.to_string(), false);
            return false;
        }
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_millis() as u64)
            .unwrap_or(0);
        let last = *self.last.get(key).unwrap_or(&0);
        if now.saturating_sub(last) >= interval_ms {
            self.last.insert(key.to_string(), now);
            let next = !*self.on.get(key).unwrap_or(&false);
            self.on.insert(key.to_string(), next);
        }
        *self.on.get(key).unwrap_or(&false)
    }
}

impl Default for TurboState {
    fn default() -> Self {
        Self::new()
    }
}

fn extract_num(s: &str) -> Option<String> {
    if let (Some(start), Some(end)) = (s.find('('), s.find(')')) {
        if start < end {
            return Some(s[start + 1..end].to_string());
        }
    }
    None
}

/// Check whether a physical key from the profile is currently pressed in the
/// raw state. Handles the canonical `Button(n)` form, the bare numeric form
/// (`n`), and the alias between them — exactly one answer per physical key.
fn physical_pressed(state: &PadState, physical: &str) -> bool {
    if *state.buttons.get(physical).unwrap_or(&false) {
        return true;
    }
    if let Some(n) = extract_num(physical) {
        if *state.buttons.get(&n).unwrap_or(&false) {
            return true;
        }
    }
    if physical.chars().all(|c| c.is_ascii_digit()) && !physical.is_empty() {
        if *state
            .buttons
            .get(&format!("Button({physical})"))
            .unwrap_or(&false)
        {
            return true;
        }
    }
    false
}

/// Turn one raw pad state into a virtual Xbox 360 report.
pub fn build_report(state: &PadState, profile: &Profile, turbo: &mut TurboState) -> XusbReport {
    let mut report = XusbReport::default();

    let turbo_enabled: std::collections::HashSet<&str> =
        profile.turbo.iter().map(|s| s.as_str()).collect();

    // Dedupe: `Button(3)` and `3` are the same physical key — resolve to one
    // canonical key so a button can't fire the same target twice, and two
    // aliases of the same physical button can't fight over different targets.
    let mut seen_phys: std::collections::HashSet<String> = std::collections::HashSet::new();
    // Anti-collision: first physical button to claim a target wins; later
    // bindings to the same target are ignored so buttons never double-fire.
    let mut claimed_targets: std::collections::HashSet<String> = std::collections::HashSet::new();

    for (physical, target) in &profile.buttons {
        let canon = if physical.chars().all(|c| c.is_ascii_digit()) && !physical.is_empty() {
            format!("Button({physical})")
        } else {
            physical.clone()
        };
        if !seen_phys.insert(canon) {
            continue;
        }

        let pressed = physical_pressed(state, physical);

        let active = if turbo_enabled.contains(physical.as_str()) {
            turbo.tick(physical, pressed, 80)
        } else {
            pressed
        };

        if !active {
            continue;
        }
        if !claimed_targets.insert(target.clone()) {
            continue;
        }

        match target.as_str() {
            "BTN_LT" => report.b_left_trigger = 255,
            "BTN_RT" => report.b_right_trigger = 255,
            other => {
                report.w_buttons |= xusb_flag(other);
            }
        }
    }

    // If D-Pad isn't mapped to physical buttons, forward raw POV hat
    let dpad_overridden = profile.buttons.values().any(|t| t.starts_with("BTN_DPAD_"));
    if !dpad_overridden {
        if *state.buttons.get("RAW_POV_UP").unwrap_or(&false) {
            report.w_buttons |= xusb_flag("BTN_DPAD_UP");
        }
        if *state.buttons.get("RAW_POV_DOWN").unwrap_or(&false) {
            report.w_buttons |= xusb_flag("BTN_DPAD_DOWN");
        }
        if *state.buttons.get("RAW_POV_LEFT").unwrap_or(&false) {
            report.w_buttons |= xusb_flag("BTN_DPAD_LEFT");
        }
        if *state.buttons.get("RAW_POV_RIGHT").unwrap_or(&false) {
            report.w_buttons |= xusb_flag("BTN_DPAD_RIGHT");
        }
    }

    let mut lt_axis = 0.0f32;
    let mut rt_axis = 0.0f32;

    for am in &profile.axes {
        let raw = state
            .axes
            .get(&am.physical)
            .copied()
            .or_else(|| match am.physical.as_str() {
                "AXIS_LX" => state.axes.get("Axis(0)").or_else(|| state.axes.get("0")).copied(),
                "AXIS_LY" => state.axes.get("Axis(1)").or_else(|| state.axes.get("1")).copied(),
                "AXIS_RX" => state.axes.get("Axis(2)").or_else(|| state.axes.get("2")).copied(),
                "AXIS_RY" => state.axes.get("Axis(3)").or_else(|| state.axes.get("3")).copied(),
                "AXIS_LT" => state.axes.get("Axis(4)").or_else(|| state.axes.get("4")).copied(),
                "AXIS_RT" => state.axes.get("Axis(5)").or_else(|| state.axes.get("5")).copied(),
                _ => None,
            })
            .unwrap_or(0.0);

        let is_trigger = am.physical == "AXIS_LT" || am.physical == "AXIS_RT";

        let value = if is_trigger {
            trigger_norm(raw)
        } else {
            let v = if am.invert { -raw } else { raw };
            deadzone(v, am.deadzone)
        };

        match am.target.as_str() {
            "AXIS_LX" => report.s_thumb_lx = to_i16(value),
            "AXIS_LY" => report.s_thumb_ly = to_i16(value),
            "AXIS_RX" => report.s_thumb_rx = to_i16(value),
            "AXIS_RY" => report.s_thumb_ry = to_i16(value),
            "AXIS_LT" => lt_axis = value,
            "AXIS_RT" => rt_axis = value,
            _ => {}
        }
    }

    let lt = (lt_axis * 255.0).round().clamp(0.0, 255.0) as u8;
    let rt = (rt_axis * 255.0).round().clamp(0.0, 255.0) as u8;
    report.b_left_trigger = report.b_left_trigger.max(lt);
    report.b_right_trigger = report.b_right_trigger.max(rt);

    report
}
