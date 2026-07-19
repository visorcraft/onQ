use std::sync::Mutex;

use serde::Serialize;
use tauri::{AppHandle, Emitter, State};

#[cfg(not(target_os = "linux"))]
use tauri_plugin_global_shortcut::{GlobalShortcutExt, ShortcutState as NativeShortcutEventState};

const PRESSED_EVENT: &str = "global-shortcut-pressed";

#[derive(Default)]
pub struct GlobalShortcutState {
    #[cfg(not(target_os = "linux"))]
    native: Mutex<Option<String>>,
    #[cfg(target_os = "linux")]
    linux: LinuxInputState,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ShortcutStatus {
    backend: &'static str,
    shortcut: String,
}

fn activate(app: &AppHandle) {
    crate::show_main_window(app);
    let _ = app.emit(PRESSED_EVENT, ());
}

#[cfg(not(target_os = "linux"))]
fn native_status(state: &GlobalShortcutState) -> Result<ShortcutStatus, String> {
    Ok(ShortcutStatus {
        backend: "native",
        shortcut: state
            .native
            .lock()
            .map_err(|error| error.to_string())?
            .clone()
            .unwrap_or_default(),
    })
}

#[cfg(not(target_os = "linux"))]
fn set_native(
    app: &AppHandle,
    state: &GlobalShortcutState,
    shortcut: Option<String>,
) -> Result<ShortcutStatus, String> {
    let Some(shortcut) = shortcut.filter(|value| !value.is_empty()) else {
        return native_status(state);
    };

    let mut current = state.native.lock().map_err(|error| error.to_string())?;
    if current.as_deref() == Some(&shortcut) {
        return Ok(ShortcutStatus {
            backend: "native",
            shortcut,
        });
    }

    if let Some(previous) = current.as_deref() {
        app.global_shortcut()
            .unregister(previous)
            .map_err(|error| error.to_string())?;
    }

    if let Err(error) = app
        .global_shortcut()
        .on_shortcut(shortcut.as_str(), |app, _, event| {
            if event.state == NativeShortcutEventState::Pressed {
                activate(app);
            }
        })
    {
        if let Some(previous) = current.as_deref() {
            let _ = app
                .global_shortcut()
                .on_shortcut(previous, |app, _, event| {
                    if event.state == NativeShortcutEventState::Pressed {
                        activate(app);
                    }
                });
        }
        return Err(error.to_string());
    }

    *current = Some(shortcut.clone());
    Ok(ShortcutStatus {
        backend: "native",
        shortcut,
    })
}

#[tauri::command]
pub fn set_global_shortcut(
    app: AppHandle,
    state: State<'_, GlobalShortcutState>,
    shortcut: Option<String>,
    _interactive: bool,
) -> Result<ShortcutStatus, String> {
    #[cfg(target_os = "linux")]
    {
        let _ = app;
        let mut current = state
            .linux
            .shortcut
            .lock()
            .map_err(|error| error.to_string())?;
        if let Some(shortcut) = shortcut.filter(|value| !value.is_empty()) {
            *current = Some(shortcut);
        }
        Ok(ShortcutStatus {
            backend: "linux-input",
            shortcut: current.clone().unwrap_or_default(),
        })
    }

    #[cfg(not(target_os = "linux"))]
    set_native(&app, &state, shortcut)
}

#[tauri::command]
pub async fn capture_global_shortcut(
    state: State<'_, GlobalShortcutState>,
) -> Result<ShortcutStatus, String> {
    #[cfg(target_os = "linux")]
    {
        if !state
            .linux
            .available
            .load(std::sync::atomic::Ordering::SeqCst)
        {
            return Err(
                "Could not read Linux input devices. Add this user to the input group.".into(),
            );
        }
        let (sender, receiver) = tokio::sync::oneshot::channel();
        *state
            .linux
            .capture
            .lock()
            .map_err(|error| error.to_string())? = Some(sender);
        let captured = receiver.await.map_err(|error| error.to_string())?;
        Ok(ShortcutStatus {
            backend: "linux-input",
            shortcut: captured.unwrap_or_else(|| {
                state
                    .linux
                    .shortcut
                    .lock()
                    .ok()
                    .and_then(|shortcut| shortcut.clone())
                    .unwrap_or_default()
            }),
        })
    }

    #[cfg(not(target_os = "linux"))]
    {
        // Command arg is required for the Tauri handler signature; native
        // capture is driven from the frontend window, not this IPC path.
        let _ = state;
        Err("native shortcut capture happens in the app window".into())
    }
}

#[cfg(target_os = "linux")]
#[derive(Default)]
struct LinuxInputState {
    shortcut: std::sync::Arc<Mutex<Option<String>>>,
    capture: std::sync::Arc<Mutex<Option<tokio::sync::oneshot::Sender<Option<String>>>>>,
    available: std::sync::Arc<std::sync::atomic::AtomicBool>,
}

#[cfg(target_os = "linux")]
pub fn start_input_listener(app: &AppHandle, state: &GlobalShortcutState) {
    use std::sync::atomic::Ordering;

    let mut opened = 0usize;
    for device in linux_input::discover_input_devices() {
        match std::fs::File::open(&device) {
            Ok(file) => {
                opened += 1;
                let app = app.clone();
                let shortcut = state.linux.shortcut.clone();
                let capture = state.linux.capture.clone();
                std::thread::spawn(move || {
                    linux_input::read_device_loop(file, app, shortcut, capture);
                });
            }
            Err(error) => {
                tracing::debug!(path = %device.display(), %error, "input device unavailable")
            }
        }
    }
    state.linux.available.store(opened > 0, Ordering::SeqCst);
    if opened == 0 {
        tracing::warn!("could not open any Linux input devices");
    } else {
        tracing::info!(opened, "Linux global shortcut listener started");
    }
}

#[cfg(not(target_os = "linux"))]
pub fn start_input_listener(_app: &AppHandle, _state: &GlobalShortcutState) {}

#[cfg(target_os = "linux")]
mod linux_input {
    use std::{
        fs::File,
        io::Read,
        os::unix::fs::FileTypeExt,
        path::PathBuf,
        sync::{Arc, Mutex},
    };

    use tauri::AppHandle;

    const EV_KEY: u16 = 0x01;
    const KEY_RELEASE: i32 = 0;
    const KEY_PRESS: i32 = 1;

    #[cfg(target_pointer_width = "64")]
    const INPUT_EVENT_SIZE: usize = 24;
    #[cfg(target_pointer_width = "64")]
    const EVENT_TYPE_OFFSET: usize = 16;
    #[cfg(target_pointer_width = "32")]
    const INPUT_EVENT_SIZE: usize = 16;
    #[cfg(target_pointer_width = "32")]
    const EVENT_TYPE_OFFSET: usize = 8;

    pub fn discover_input_devices() -> Vec<PathBuf> {
        let Ok(contents) = std::fs::read_to_string("/proc/bus/input/devices") else {
            return Vec::new();
        };
        contents
            .split("\n\n")
            .filter_map(|block| {
                let handlers = block
                    .lines()
                    .find_map(|line| line.strip_prefix("H: Handlers=").map(str::trim))?;
                if !handlers.split_whitespace().any(|value| value == "kbd") {
                    return None;
                }
                let event = handlers
                    .split_whitespace()
                    .find(|value| value.starts_with("event"))?;
                Some(PathBuf::from("/dev/input").join(event))
            })
            .filter(|path| {
                std::fs::metadata(path)
                    .map(|metadata| metadata.file_type().is_char_device())
                    .unwrap_or(false)
            })
            .collect()
    }

    pub fn read_device_loop(
        mut file: File,
        app: AppHandle,
        shortcut: Arc<Mutex<Option<String>>>,
        capture: Arc<Mutex<Option<tokio::sync::oneshot::Sender<Option<String>>>>>,
    ) {
        let mut buffer = [0u8; INPUT_EVENT_SIZE];
        let mut modifiers = Modifiers::default();

        while file.read_exact(&mut buffer).is_ok() {
            let event_type =
                u16::from_ne_bytes([buffer[EVENT_TYPE_OFFSET], buffer[EVENT_TYPE_OFFSET + 1]]);
            let code =
                u16::from_ne_bytes([buffer[EVENT_TYPE_OFFSET + 2], buffer[EVENT_TYPE_OFFSET + 3]]);
            let value = i32::from_ne_bytes([
                buffer[EVENT_TYPE_OFFSET + 4],
                buffer[EVENT_TYPE_OFFSET + 5],
                buffer[EVENT_TYPE_OFFSET + 6],
                buffer[EVENT_TYPE_OFFSET + 7],
            ]);

            if event_type != EV_KEY {
                continue;
            }
            if modifiers.update(code, value != KEY_RELEASE) {
                continue;
            }
            if value != KEY_PRESS {
                continue;
            }
            if code == 1 {
                if let Some(sender) = capture.lock().ok().and_then(|mut slot| slot.take()) {
                    let _ = sender.send(None);
                }
                continue;
            }
            let Some(label) = shortcut_label(code, modifiers) else {
                continue;
            };

            if let Some(sender) = capture.lock().ok().and_then(|mut slot| slot.take()) {
                if let Ok(mut current) = shortcut.lock() {
                    *current = Some(label.clone());
                }
                let _ = sender.send(Some(label));
                continue;
            }

            if shortcut
                .lock()
                .is_ok_and(|current| current.as_deref() == Some(&label))
            {
                super::activate(&app);
            }
        }
    }

    #[derive(Clone, Copy, Default)]
    struct Modifiers(u8);

    impl Modifiers {
        fn update(&mut self, code: u16, pressed: bool) -> bool {
            let bit = match code {
                29 | 97 => 1,
                56 | 100 => 2,
                42 | 54 => 4,
                125 | 126 => 8,
                _ => return false,
            };
            if pressed {
                self.0 |= bit;
            } else {
                self.0 &= !bit;
            }
            true
        }

        fn any(self) -> bool {
            self.0 != 0
        }
    }

    fn shortcut_label(code: u16, modifiers: Modifiers) -> Option<String> {
        if !modifiers.any() {
            return None;
        }
        let key = key_label(code)?;
        let mut parts = Vec::with_capacity(5);
        for (bit, label) in [(1, "Ctrl"), (2, "Alt"), (4, "Shift"), (8, "Super")] {
            if modifiers.0 & bit != 0 {
                parts.push(label);
            }
        }
        parts.push(key);
        Some(parts.join("+"))
    }

    fn key_label(code: u16) -> Option<&'static str> {
        Some(match code {
            2 => "1",
            3 => "2",
            4 => "3",
            5 => "4",
            6 => "5",
            7 => "6",
            8 => "7",
            9 => "8",
            10 => "9",
            11 => "0",
            12 => "Minus",
            13 => "Equal",
            14 => "Backspace",
            15 => "Tab",
            16 => "Q",
            17 => "W",
            18 => "E",
            19 => "R",
            20 => "T",
            21 => "Y",
            22 => "U",
            23 => "I",
            24 => "O",
            25 => "P",
            26 => "BracketLeft",
            27 => "BracketRight",
            28 => "Enter",
            30 => "A",
            31 => "S",
            32 => "D",
            33 => "F",
            34 => "G",
            35 => "H",
            36 => "J",
            37 => "K",
            38 => "L",
            39 => "Semicolon",
            40 => "Quote",
            41 => "Backquote",
            43 => "Backslash",
            44 => "Z",
            45 => "X",
            46 => "C",
            47 => "V",
            48 => "B",
            49 => "N",
            50 => "M",
            51 => "Comma",
            52 => "Period",
            53 => "Slash",
            57 => "Space",
            59 => "F1",
            60 => "F2",
            61 => "F3",
            62 => "F4",
            63 => "F5",
            64 => "F6",
            65 => "F7",
            66 => "F8",
            67 => "F9",
            68 => "F10",
            87 => "F11",
            88 => "F12",
            102 => "Home",
            103 => "ArrowUp",
            104 => "PageUp",
            105 => "ArrowLeft",
            106 => "ArrowRight",
            107 => "End",
            108 => "ArrowDown",
            109 => "PageDown",
            110 => "Insert",
            111 => "Delete",
            _ => return None,
        })
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn formats_ctrl_and_super_shortcuts() {
            assert_eq!(shortcut_label(37, Modifiers(1)).as_deref(), Some("Ctrl+K"));
            assert_eq!(shortcut_label(37, Modifiers(8)).as_deref(), Some("Super+K"));
        }
    }
}
