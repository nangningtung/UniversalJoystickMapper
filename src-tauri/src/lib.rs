pub mod input;
pub mod mapping;
pub mod profile;
pub mod vigem;

use std::sync::mpsc::Sender;
use std::sync::Arc;
use std::time::Duration;

use parking_lot::Mutex;
use tauri::menu::{Menu, MenuItem};
use tauri::tray::TrayIconBuilder;
use tauri::{AppHandle, Manager, State};

use input::{Command, ControllerInfo, MappingStatus, Shared};
use profile::{Profile, ProfileStore};

pub struct AppCtx {
    pub shared: Arc<Shared>,
    pub tx: Mutex<Sender<Command>>,
    pub store: Mutex<ProfileStore>,
}

fn uuid_for(ctx: &AppCtx, id: usize) -> Option<String> {
    ctx.shared
        .controllers
        .lock()
        .iter()
        .find(|c| c.id == id)
        .map(|c| c.uuid.clone())
}

#[tauri::command]
fn list_controllers(ctx: State<'_, AppCtx>) -> Vec<ControllerInfo> {
    ctx.shared.controllers.lock().clone()
}

#[tauri::command]
fn get_status(ctx: State<'_, AppCtx>) -> MappingStatus {
    ctx.shared.status.lock().clone()
}

#[tauri::command]
fn list_profiles(ctx: State<'_, AppCtx>) -> std::collections::HashMap<String, Profile> {
    ctx.store.lock().list_named()
}

#[tauri::command]
fn get_profile_for_controller(ctx: State<'_, AppCtx>, id: usize) -> Option<Profile> {
    let uuid = uuid_for(&ctx, id)?;
    ctx.store.lock().load_for(&uuid)
}

#[tauri::command]
fn save_profile_for_controller(
    ctx: State<'_, AppCtx>,
    id: usize,
    profile: Profile,
) -> Result<(), String> {
    let uuid = uuid_for(&ctx, id).ok_or_else(|| "controller not found".to_string())?;
    ctx.store.lock().save_for(&uuid, &profile)
}

#[tauri::command]
fn start_emulation(
    ctx: State<'_, AppCtx>,
    id: usize,
    profile: Profile,
) -> Result<MappingStatus, String> {
    if profile.controller_type != "xbox360" {
        return Err("DualShock 4 output is not wired up yet — use Xbox 360 for now.".to_string());
    }

    let uuid = uuid_for(&ctx, id).unwrap_or_else(|| format!("device{id}"));
    let profile = profile.normalized();

    let _ = ctx.store.lock().save_for(&uuid, &profile);
    ctx.shared.profiles.lock().insert(id, profile.clone());

    ctx.tx
        .lock()
        .send(Command::Start { id, uuid, profile })
        .map_err(|e| e.to_string())?;

    std::thread::sleep(Duration::from_millis(180));
    Ok(ctx.shared.status.lock().clone())
}

#[tauri::command]
fn stop_emulation(ctx: State<'_, AppCtx>, id: usize) -> Result<MappingStatus, String> {
    ctx.tx
        .lock()
        .send(Command::Stop { id })
        .map_err(|e| e.to_string())?;
    std::thread::sleep(Duration::from_millis(120));
    Ok(ctx.shared.status.lock().clone())
}

#[tauri::command]
fn rescan_controllers(ctx: State<'_, AppCtx>) -> Result<Vec<ControllerInfo>, String> {
    ctx.tx.lock().send(Command::Rescan).map_err(|e| e.to_string())?;
    std::thread::sleep(Duration::from_millis(100));
    Ok(ctx.shared.controllers.lock().clone())
}

#[tauri::command]
fn clear_disconnected(ctx: State<'_, AppCtx>) -> Vec<ControllerInfo> {
    let mut list = ctx.shared.controllers.lock();
    list.retain(|c| c.connected);
    list.clone()
}

fn build_tray(app: &AppHandle) -> tauri::Result<()> {
    let show = MenuItem::with_id(app, "show", "Show GhostPad", true, None::<&str>)?;
    let quit = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;
    let menu = Menu::with_items(app, &[&show, &quit])?;

    let mut builder = TrayIconBuilder::with_id("ghostpad-tray")
        .tooltip("GhostPad")
        .menu(&menu)
        .show_menu_on_left_click(false)
        .on_menu_event(|app, event| match event.id().as_ref() {
            "show" => {
                if let Some(win) = app.get_webview_window("main") {
                    let _ = win.show();
                    let _ = win.set_focus();
                }
            }
            "quit" => app.exit(0),
            _ => {}
        });

    if let Some(icon) = app.default_window_icon().cloned() {
        builder = builder.icon(icon);
    }

    // Left click shows the window (right click opens menu)
    builder = builder.on_tray_icon_event(|tray, event| {
        if let tauri::tray::TrayIconEvent::Click { .. } = event {
            if let Some(win) = tray.app_handle().get_webview_window("main") {
                let _ = win.show();
                let _ = win.set_focus();
            }
        }
    });

    builder.build(app)?;
    Ok(())
}

pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            let handle = app.handle().clone();

            let shared = Arc::new(Shared::new());
            let (tx, rx) = input::channel_pair();

            let api = input::probe_driver();
            {
                let mut st = shared.status.lock();
                st.driver_available = api.is_some();
                if api.is_none() {
                    st.error = Some(
                        "ViGEmClient.dll not found. Install ViGEmBus to enable virtual output."
                            .into(),
                    );
                }
            }
            *shared.api.0.lock() = api;

            let profiles_dir = app
                .path()
                .app_data_dir()
                .unwrap_or_else(|_| std::path::PathBuf::from("."))
                .join("profiles");
            let store = ProfileStore::new(profiles_dir);

            input::spawn(handle.clone(), shared.clone(), rx);

            app.manage(AppCtx {
                shared,
                tx: Mutex::new(tx),
                store: Mutex::new(store),
            });

            if let Err(e) = build_tray(&handle) {
                eprintln!("tray init failed: {e}");
            }

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            list_controllers,
            rescan_controllers,
            clear_disconnected,
            get_status,
            list_profiles,
            get_profile_for_controller,
            save_profile_for_controller,
            start_emulation,
            stop_emulation,
        ])
        .run(tauri::generate_context!())
        .expect("error while running GhostPad");
}
