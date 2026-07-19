mod auto_lock;
mod commands;
mod global_shortcut;
mod state;

use commands::apply_auto_lock_on_start;
use global_shortcut::GlobalShortcutState;
use state::AppState;
use tauri::{
    menu::{Menu, MenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    Manager, WindowEvent,
};

#[cfg(target_os = "linux")]
use std::process::{Command, Stdio};

#[cfg(target_os = "linux")]
const KWIN_WATCHER_NAME: &str = "onq-tray-watch";

#[cfg(target_os = "linux")]
const KWIN_WATCHER_SCRIPT: &str = r#"
const ids = ["com.visorcraft.onq", "onQ", "onq-app"];

function isOnq(window) {
  const values = ["desktopFileName", "resourceClass", "resourceName", "caption"]
    .map(key => String(window[key] || "").toLowerCase());
  return values.some(value => ids.includes(value));
}

function syncTaskbar(window) {
  if (isOnq(window)) {
    window.skipTaskbar = window.minimized;
  }
}

function watch(window) {
  if (!isOnq(window)) return;
  syncTaskbar(window);
  if (window.minimizedChanged) {
    window.minimizedChanged.connect(() => syncTaskbar(window));
  }
}

const windows = workspace.windowList ? workspace.windowList() : workspace.clientList();
for (const window of windows) watch(window);
if (workspace.windowAdded) workspace.windowAdded.connect(watch);
else if (workspace.clientAdded) workspace.clientAdded.connect(watch);
"#;

#[cfg(target_os = "linux")]
const KWIN_RESTORE_SCRIPT: &str = r#"
const ids = ["com.visorcraft.onq", "onQ", "onq-app"];
const windows = workspace.windowList ? workspace.windowList() : workspace.clientList();
for (const window of windows) {
  const values = ["desktopFileName", "resourceClass", "resourceName", "caption"]
    .map(key => String(window[key] || "").toLowerCase());
  if (values.some(value => ids.includes(value))) {
    window.skipTaskbar = false;
    window.minimized = false;
    workspace.activeWindow = window;
  }
}
"#;

#[cfg(target_os = "linux")]
struct PlasmaTrayWatcher;

#[cfg(target_os = "linux")]
impl PlasmaTrayWatcher {
    fn install() -> Option<Self> {
        if !is_kde_session() {
            return None;
        }
        let path = std::env::temp_dir().join(format!("{KWIN_WATCHER_NAME}.js"));
        std::fs::write(&path, KWIN_WATCHER_SCRIPT).ok()?;
        unload_kwin_script(KWIN_WATCHER_NAME);
        let loaded = Command::new("qdbus6")
            .args([
                "org.kde.KWin",
                "/Scripting",
                "org.kde.kwin.Scripting.loadScript",
            ])
            .arg(&path)
            .arg(KWIN_WATCHER_NAME)
            .output()
            .is_ok_and(|output| output.status.success());
        if !loaded {
            let _ = std::fs::remove_file(path);
            return None;
        }
        let started = Command::new("qdbus6")
            .args(["org.kde.KWin", "/Scripting", "org.kde.kwin.Scripting.start"])
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .is_ok_and(|status| status.success());
        let _ = std::fs::remove_file(path);
        started.then_some(Self)
    }
}

#[cfg(target_os = "linux")]
impl Drop for PlasmaTrayWatcher {
    fn drop(&mut self) {
        unload_kwin_script(KWIN_WATCHER_NAME);
    }
}

#[cfg(target_os = "linux")]
fn unload_kwin_script(name: &str) {
    let _ = Command::new("qdbus6")
        .args([
            "org.kde.KWin",
            "/Scripting",
            "org.kde.kwin.Scripting.unloadScript",
        ])
        .arg(name)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status();
}

#[cfg(target_os = "linux")]
fn is_kde_session() -> bool {
    std::env::var("XDG_CURRENT_DESKTOP")
        .is_ok_and(|desktop| desktop.to_ascii_lowercase().contains("kde"))
}

#[cfg(target_os = "linux")]
fn restore_plasma_window() {
    if !is_kde_session() {
        return;
    }
    let name = format!("onq-restore-{}", std::process::id());
    let path = std::env::temp_dir().join(format!("{name}.js"));
    if std::fs::write(&path, KWIN_RESTORE_SCRIPT).is_err() {
        return;
    }
    unload_kwin_script(&name);
    let loaded = Command::new("qdbus6")
        .args([
            "org.kde.KWin",
            "/Scripting",
            "org.kde.kwin.Scripting.loadScript",
        ])
        .arg(&path)
        .arg(&name)
        .output()
        .is_ok_and(|output| output.status.success());
    if loaded {
        let _ = Command::new("qdbus6")
            .args(["org.kde.KWin", "/Scripting", "org.kde.kwin.Scripting.start"])
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status();
        unload_kwin_script(&name);
    }
    let _ = std::fs::remove_file(path);
}

fn hide_main_window(window: &tauri::Window) {
    #[cfg(target_os = "linux")]
    if is_kde_session() {
        let _ = window.minimize();
        return;
    }
    let _ = window.set_skip_taskbar(true);
    let _ = window.unminimize();
    let _ = window.hide();
}

pub(crate) fn show_main_window(app: &tauri::AppHandle) {
    #[cfg(target_os = "linux")]
    restore_plasma_window();
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.set_skip_taskbar(false);
        let _ = window.show();
        let _ = window.unminimize();
        let _ = window.set_focus();
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let builder = tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_updater::Builder::new().build());
    #[cfg(all(desktop, not(target_os = "linux")))]
    let builder = builder.plugin(tauri_plugin_global_shortcut::Builder::new().build());

    builder
        .manage(AppState::default())
        .manage(GlobalShortcutState::default())
        .setup(|app| {
            tracing_subscriber::fmt::init();
            tracing::info!("onQ launched");
            #[cfg(target_os = "linux")]
            if let Some(watcher) = PlasmaTrayWatcher::install() {
                app.manage(watcher);
            }
            // M5.5: evaluate the active auto-lock policy at launch. Activity
            // tracking is not yet wired, so this is a no-op today except for
            // serving as the entry point the future input tracker hooks into.
            let state = app.state::<AppState>();
            apply_auto_lock_on_start(&state);
            let shortcut_state = app.state::<GlobalShortcutState>();
            global_shortcut::start_input_listener(app.handle(), &shortcut_state);

            let show = MenuItem::with_id(app, "show", "Show onQ", true, None::<&str>)?;
            let quit = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;
            let menu = Menu::with_items(app, &[&show, &quit])?;
            TrayIconBuilder::new()
                .icon(app.default_window_icon().expect("application icon").clone())
                .menu(&menu)
                .on_menu_event(|app, event| match event.id.as_ref() {
                    "show" => show_main_window(app),
                    "quit" => app.exit(0),
                    _ => {}
                })
                .on_tray_icon_event(|tray, event| {
                    if let TrayIconEvent::Click {
                        button: MouseButton::Left,
                        button_state: MouseButtonState::Up,
                        ..
                    } = event
                    {
                        show_main_window(tray.app_handle());
                    }
                })
                .build(app)?;
            Ok(())
        })
        .on_window_event(|window, event| match event {
            WindowEvent::CloseRequested { api, .. } => {
                api.prevent_close();
                hide_main_window(window);
            }
            WindowEvent::Resized(_) if window.is_minimized().unwrap_or(false) => {
                hide_main_window(window);
            }
            _ => {}
        })
        .invoke_handler(tauri::generate_handler![
            global_shortcut::set_global_shortcut,
            global_shortcut::capture_global_shortcut,
            commands::ping,
            commands::setup_new_vault,
            commands::unlock_vault,
            commands::recover_vault,
            commands::open_last_vault,
            commands::get_vault_auth_mode,
            commands::retrieve_vault_key,
            commands::open_vault,
            commands::list_prompts,
            commands::read_prompt,
            commands::create_prompt,
            commands::save_prompt,
            commands::delete_prompt,
            commands::lock_prompt,
            commands::unlock_prompt,
            commands::search,
            commands::run_smart_folder,
            commands::more_like_this,
            commands::create_smart_folder,
            commands::list_smart_folders,
            commands::update_smart_folder,
            commands::delete_smart_folder,
            commands::set_auto_lock_policy,
            commands::get_auto_lock_policy,
            commands::get_app_setting,
            commands::set_app_setting,
            commands::set_embedding_quant,
            commands::get_beta_channel,
            commands::set_beta_channel,
            commands::record_search,
            commands::record_open,
            commands::install_plugin,
            commands::list_plugins,
            commands::set_plugin_enabled,
            commands::uninstall_plugin,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[cfg(test)]
mod tests {
    #[test]
    fn updater_uses_production_release_endpoint_and_frontend_permission() {
        let config: serde_json::Value =
            serde_json::from_str(include_str!("../tauri.conf.json")).expect("valid Tauri config");
        assert_eq!(
            config["plugins"]["updater"]["endpoints"][0],
            "https://github.com/visorcraft/onQ/releases/latest/download/latest.json"
        );
        assert_eq!(config["bundle"]["createUpdaterArtifacts"], true);

        let capability: serde_json::Value =
            serde_json::from_str(include_str!("../capabilities/default.json"))
                .expect("valid default capability");
        assert!(capability["permissions"]
            .as_array()
            .expect("permissions array")
            .iter()
            .any(|permission| permission == "updater:allow-check"));
    }
}
