use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

pub const BUTTONS: [&str; 17] = [
    "BTN_A",
    "BTN_B",
    "BTN_X",
    "BTN_Y",
    "BTN_LB",
    "BTN_RB",
    "BTN_LT",
    "BTN_RT",
    "BTN_BACK",
    "BTN_START",
    "BTN_GUIDE",
    "BTN_LTHUMB",
    "BTN_RTHUMB",
    "BTN_DPAD_UP",
    "BTN_DPAD_DOWN",
    "BTN_DPAD_LEFT",
    "BTN_DPAD_RIGHT",
];

pub const AXES: [&str; 6] = [
    "AXIS_LX",
    "AXIS_LY",
    "AXIS_RX",
    "AXIS_RY",
    "AXIS_LT",
    "AXIS_RT",
];

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AxisMapping {
    pub physical: String,
    pub target: String,
    #[serde(default)]
    pub invert: bool,
    #[serde(default = "default_deadzone")]
    pub deadzone: f32,
}

fn default_deadzone() -> f32 {
    0.1
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Profile {
    #[serde(default = "default_name")]
    pub name: String,
    #[serde(default = "default_layout")]
    pub layout: String,
    #[serde(default = "default_controller")]
    pub controller_type: String,
    #[serde(default)]
    pub buttons: HashMap<String, String>,
    #[serde(default)]
    pub axes: Vec<AxisMapping>,
    #[serde(default)]
    pub turbo: Vec<String>,
}

fn default_name() -> String {
    "Default".into()
}
fn default_layout() -> String {
    "playstation".into()
}
fn default_controller() -> String {
    "xbox360".into()
}

impl Default for Profile {
    fn default() -> Self {
        // Buttons start EMPTY — the user binds each physical button explicitly.
        // No auto 1:1 mapping, otherwise every press fires twice (once from the
        // numeric key, once from the Button(n) key).
        let axes = AXES
            .iter()
            .map(|a| AxisMapping {
                physical: a.to_string(),
                target: a.to_string(),
                invert: *a == "AXIS_LY" || *a == "AXIS_RY",
                deadzone: default_deadzone(),
            })
            .collect();
        Profile {
            name: default_name(),
            layout: default_layout(),
            controller_type: default_controller(),
            buttons: HashMap::new(),
            axes,
            turbo: Vec::new(),
        }
    }
}

impl Profile {
    /// Fill in anything the frontend omitted so the engine never sees holes.
    /// NOTE: we deliberately do NOT auto-fill buttons — if the user cleared a
    /// button mapping it must stay unmapped. Only axes get defaults so sticks
    /// and triggers keep working out of the box.
    /// Also canonicalise button keys (`3` -> `Button(3)`) and drop alias
    /// duplicates so a physical button can never fire twice.
    pub fn normalized(mut self) -> Self {
        let mut canon: HashMap<String, String> = HashMap::new();
        for (k, v) in &self.buttons {
            let key = if k.chars().all(|c| c.is_ascii_digit()) && !k.is_empty() {
                format!("Button({k})")
            } else {
                k.clone()
            };
            canon.entry(key).or_insert_with(|| v.clone());
        }
        self.buttons = canon;
        for a in AXES {
            if !self.axes.iter().any(|m| m.physical == a) {
                self.axes.push(AxisMapping {
                    physical: a.to_string(),
                    target: a.to_string(),
                    invert: a == "AXIS_LY" || a == "AXIS_RY",
                    deadzone: default_deadzone(),
                });
            }
        }
        self
    }
}

pub struct ProfileStore {
    dir: PathBuf,
}

fn safe_name(raw: &str) -> String {
    raw.chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || c == '_' || c == '-' || c == '.' {
                c
            } else {
                '_'
            }
        })
        .collect()
}

impl ProfileStore {
    pub fn new(dir: PathBuf) -> Self {
        let _ = fs::create_dir_all(&dir);
        ProfileStore { dir }
    }

    pub fn dir(&self) -> &Path {
        &self.dir
    }

    fn auto_path(&self, uuid: &str) -> PathBuf {
        self.dir.join(format!("auto_{}.json", safe_name(uuid)))
    }

    pub fn load_for(&self, uuid: &str) -> Option<Profile> {
        let data = fs::read_to_string(self.auto_path(uuid)).ok()?;
        serde_json::from_str::<Profile>(&data).ok()
    }

    pub fn save_for(&self, uuid: &str, profile: &Profile) -> Result<(), String> {
        let data = serde_json::to_string_pretty(profile).map_err(|e| e.to_string())?;
        fs::write(self.auto_path(uuid), data).map_err(|e| e.to_string())
    }

    /// All named profiles (`*.json` that are not `auto_*`).
    pub fn list_named(&self) -> HashMap<String, Profile> {
        let mut out = HashMap::new();
        let Ok(entries) = fs::read_dir(&self.dir) else {
            return out;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) != Some("json") {
                continue;
            }
            let Some(stem) = path.file_stem().and_then(|s| s.to_str()) else {
                continue;
            };
            if stem.starts_with("auto_") {
                continue;
            }
            if let Ok(data) = fs::read_to_string(&path) {
                if let Ok(p) = serde_json::from_str::<Profile>(&data) {
                    out.insert(stem.to_string(), p);
                }
            }
        }
        out
    }
}
