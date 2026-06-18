#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::{
    env, fs,
    io::{Read, Write},
    net::{TcpListener, TcpStream},
    path::PathBuf,
    sync::{
        atomic::{AtomicU64, Ordering},
        Arc, Mutex,
    },
    thread,
    time::{Duration, SystemTime, UNIX_EPOCH},
};
use tauri::{
    menu::MenuBuilder, Emitter, Manager, PhysicalPosition, PhysicalSize, Position, Size,
    WebviewWindow, WindowEvent,
};

const HOST: &str = "127.0.0.1";
const PORT: u16 = 37421;
const SNAP_DISTANCE: i32 = 24;
const APP_NAME: &str = "Agent Status Light";
const REGISTRY_KEY: &str = "LightForAllAgent";

// ── Modes ──────────────────────────────────────────────────────────
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
enum LightMode {
    Idle,
    Working,
    Waiting,
    Error,
}

impl LightMode {
    fn as_str(&self) -> &'static str {
        match self {
            Self::Idle => "idle",
            Self::Working => "working",
            Self::Waiting => "waiting",
            Self::Error => "error",
        }
    }

    fn from_str(s: &str) -> Self {
        match s.to_ascii_lowercase().as_str() {
            "working" => Self::Working,
            "waiting" => Self::Waiting,
            "error" => Self::Error,
            _ => Self::Idle,
        }
    }
}

// ── Orientation ─────────────────────────────────────────────────────
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
enum Orientation {
    Horizontal,
    Vertical,
}

impl Default for Orientation {
    fn default() -> Self {
        Self::Horizontal
    }
}

impl Orientation {
    fn as_str(&self) -> &'static str {
        match self {
            Self::Horizontal => "horizontal",
            Self::Vertical => "vertical",
        }
    }
}

// ── State types ─────────────────────────────────────────────────────
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct LightState {
    mode: String,
    received_at: String,
    message: String,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct StatePayload {
    current_state: LightState,
    orientation: String,
    auto_start: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct Settings {
    orientation: Orientation,
    auto_start: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            orientation: Orientation::Horizontal,
            auto_start: false,
        }
    }
}

struct AppData {
    current_state: LightState,
    settings: Settings,
}

type SharedState = Arc<Mutex<AppData>>;

// ── Helpers ─────────────────────────────────────────────────────────
fn now_iso() -> String {
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    format!("{secs}")
}

fn default_light_state() -> LightState {
    LightState {
        mode: "idle".to_string(),
        received_at: now_iso(),
        message: "等待 Agent 连接".to_string(),
    }
}

// ── Windows auto-start ──────────────────────────────────────────────
#[cfg(target_os = "windows")]
fn register_auto_start(enabled: bool) -> Result<(), String> {
    use winreg::enums::*;
    use winreg::RegKey;

    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let path = r"Software\Microsoft\Windows\CurrentVersion\Run";
    let key = hkcu.open_subkey_with_flags(path, KEY_SET_VALUE)
        .map_err(|e| format!("无法打开注册表: {e}"))?;

    if enabled {
        let exe = env::current_exe().map_err(|e| e.to_string())?;
        let exe_str = exe.to_string_lossy().to_string();
        key.set_value(REGISTRY_KEY, &exe_str)
            .map_err(|e| format!("无法写入注册表: {e}"))?;
    } else {
        key.delete_value(REGISTRY_KEY)
            .map_err(|e| format!("无法删除注册表项: {e}"))?;
    }
    Ok(())
}

#[cfg(not(target_os = "windows"))]
fn register_auto_start(_enabled: bool) -> Result<(), String> {
    Err("仅支持 Windows 自动启动".to_string())
}

fn check_auto_start() -> bool {
    #[cfg(target_os = "windows")]
    {
        use winreg::enums::*;
        use winreg::RegKey;
        if let Ok(hkcu) = RegKey::predef(HKEY_CURRENT_USER)
            .open_subkey_with_flags(r"Software\Microsoft\Windows\CurrentVersion\Run", KEY_READ)
        {
            return hkcu.get_value::<String, _>(REGISTRY_KEY).is_ok();
        }
    }
    false
}

// ── HTTP server ─────────────────────────────────────────────────────
fn handle_http(state: &SharedState, app: &tauri::AppHandle, payload: Value) -> Value {
    let mode_raw = payload
        .get("mode")
        .and_then(Value::as_str)
        .unwrap_or("idle");
    let mode = LightMode::from_str(mode_raw);
    let message = payload
        .get("message")
        .and_then(Value::as_str)
        .unwrap_or("")
        .to_string();

    let light_state = LightState {
        mode: mode.as_str().to_string(),
        received_at: now_iso(),
        message,
    };

    let event = {
        let mut data = state.lock().expect("state poisoned");
        data.current_state = light_state.clone();
        StatePayload {
            current_state: data.current_state.clone(),
            orientation: data.settings.orientation.as_str().to_string(),
            auto_start: check_auto_start(),
        }
    };

    let _ = app.emit("status-changed", event);

    json!({ "ok": true, "mode": light_state.mode })
}

fn read_http_body(stream: &mut TcpStream) -> Option<Value> {
    let _ = stream.set_read_timeout(Some(Duration::from_millis(500)));
    let mut buffer = Vec::new();
    let mut chunk = [0u8; 8192];
    loop {
        let read = stream.read(&mut chunk).ok()?;
        if read == 0 {
            break;
        }
        buffer.extend_from_slice(&chunk[..read]);
        if buffer.windows(4).any(|w| w == b"\r\n\r\n") {
            let text = String::from_utf8_lossy(&buffer);
            let cl: usize = text
                .lines()
                .find_map(|l| {
                    l.to_ascii_lowercase()
                        .strip_prefix("content-length:")
                        .and_then(|v| v.trim().parse().ok())
                })
                .unwrap_or(0);
            let header_end = text.find("\r\n\r\n")? + 4;
            while buffer.len() < header_end + cl {
                let r = stream.read(&mut chunk).ok()?;
                if r == 0 {
                    break;
                }
                buffer.extend_from_slice(&chunk[..r]);
            }
            let full = String::from_utf8_lossy(&buffer).to_string();
            let body = full[header_end..].to_string();
            return serde_json::from_str(&body).ok();
        }
    }
    None
}

fn write_response(stream: &mut TcpStream, status: &str, body: Value) {
    let body = body.to_string();
    let resp = format!(
        "HTTP/1.1 {status}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\nAccess-Control-Allow-Origin: *\r\n\r\n{}",
        body.len(),
        body
    );
    let _ = stream.write_all(resp.as_bytes());
}

fn start_http_server(app: tauri::AppHandle, state: SharedState) {
    thread::spawn(move || {
        let listener = match TcpListener::bind((HOST, PORT)) {
            Ok(l) => l,
            Err(e) => {
                eprintln!("Agent Status Light HTTP server 启动失败: {e}");
                return;
            }
        };

        for stream in listener.incoming() {
            let Ok(mut stream) = stream else {
                continue;
            };
            let app = app.clone();
            let state = state.clone();
            thread::spawn(move || {
                let (method, path) = {
                    let mut buf = [0u8; 1024];
                    let n = match stream.peek(&mut buf) {
                        Ok(n) => n,
                        Err(_) => return,
                    };
                    let text = String::from_utf8_lossy(&buf[..n]);
                    let first = text.lines().next().unwrap_or("");
                    let parts: Vec<&str> = first.split_whitespace().collect();
                    (
                        parts.first().unwrap_or(&"").to_string(),
                        parts.get(1).unwrap_or(&"").to_string(),
                    )
                };

                // GET /status — 查询当前状态
                if method == "GET" && path == "/status" {
                    let data = state.lock().unwrap();
                    write_response(
                        &mut stream,
                        "200 OK",
                        json!({
                            "ok": true,
                            "mode": data.current_state.mode,
                            "message": data.current_state.message,
                            "orientation": data.settings.orientation.as_str(),
                            "autoStart": check_auto_start()
                        }),
                    );
                    return;
                }

                // POST /status — 更新状态
                if method == "POST" && path == "/status" {
                    let Some(payload) = read_http_body(&mut stream) else {
                        write_response(&mut stream, "400 Bad Request", json!({"ok":false,"error":"无效的 JSON 请求体"}));
                        return;
                    };
                    let result = handle_http(&state, &app, payload);
                    write_response(&mut stream, "200 OK", result);
                    return;
                }

                write_response(&mut stream, "404 Not Found", json!({"ok":false,"error":"未找到路由"}));
            });
        }
    });
}

// ── Settings persistence ────────────────────────────────────────────
fn settings_path() -> PathBuf {
    dirs::data_local_dir()
        .unwrap_or_else(env::temp_dir)
        .join(APP_NAME)
        .join("settings.json")
}

fn load_settings() -> Settings {
    fs::read_to_string(settings_path())
        .ok()
        .and_then(|v| serde_json::from_str(&v).ok())
        .unwrap_or_default()
}

fn save_settings(s: &Settings) -> Result<(), String> {
    let path = settings_path();
    if let Some(p) = path.parent() {
        fs::create_dir_all(p).map_err(|e| e.to_string())?;
    }
    fs::write(path, serde_json::to_string_pretty(s).map_err(|e| e.to_string())?)
        .map_err(|e| e.to_string())
}

// ── Window helpers ──────────────────────────────────────────────────
fn widget_size(window: &WebviewWindow, orientation: &Orientation) -> Result<(u32, u32), String> {
    let monitor = window
        .current_monitor()
        .map_err(|e| e.to_string())?
        .or_else(|| window.primary_monitor().ok().flatten())
        .ok_or_else(|| "无法获取显示器信息".to_string())?;
    let size = monitor.size();
    Ok(match orientation {
        Orientation::Horizontal => ((size.width / 10).max(120), (size.height / 15).max(56)),
        Orientation::Vertical => (
            (size.width / 24).max(44),
            ((size.height as f64 / 5.5) as u32).max(180),
        ),
    })
}

fn snap_window_inner(window: &WebviewWindow, state: &SharedState) -> Result<(), String> {
    let orientation = {
        state
            .lock()
            .map_err(|_| "state poisoned".to_string())?
            .settings
            .orientation
            .clone()
    };
    let monitor = window
        .current_monitor()
        .map_err(|e| e.to_string())?
        .or_else(|| window.primary_monitor().ok().flatten())
        .ok_or_else(|| "无法获取显示器信息".to_string())?;
    let ms = monitor.size();
    let mp = monitor.position();
    let (w, h) = widget_size(window, &orientation)?;
    let pos = window.outer_position().map_err(|e| e.to_string())?;

    let min_x = mp.x;
    let min_y = mp.y;
    let max_x = mp.x + ms.width as i32 - w as i32;
    let max_y = mp.y + ms.height as i32 - h as i32;

    let mut x = pos.x.clamp(min_x, max_x);
    let mut y = pos.y.clamp(min_y, max_y);

    if (x - min_x).abs() <= SNAP_DISTANCE {
        x = min_x;
    }
    if (y - min_y).abs() <= SNAP_DISTANCE {
        y = min_y;
    }
    if (x + w as i32 - (mp.x + ms.width as i32)).abs() <= SNAP_DISTANCE {
        x = max_x;
    }
    if (y + h as i32 - (mp.y + ms.height as i32)).abs() <= SNAP_DISTANCE {
        y = max_y;
    }

    window
        .set_size(Size::Physical(PhysicalSize {
            width: w,
            height: h,
        }))
        .map_err(|e| e.to_string())?;
    window
        .set_position(Position::Physical(PhysicalPosition { x, y }))
        .map_err(|e| e.to_string())?;
    let _ = window.set_always_on_top(true);
    Ok(())
}

fn apply_orientation(
    window: &WebviewWindow,
    state: &SharedState,
    orientation: String,
) -> Result<(), String> {
    let next = match orientation.as_str() {
        "horizontal" => Orientation::Horizontal,
        "vertical" => Orientation::Vertical,
        _ => return Err("无效方向".to_string()),
    };

    {
        let mut data = state.lock().map_err(|_| "state poisoned".to_string())?;
        data.settings.orientation = next;
        save_settings(&data.settings)?;
        window
            .emit("orientation-changed", data.settings.orientation.as_str())
            .map_err(|e| e.to_string())?;
    }

    snap_window_inner(window, state)
}

fn schedule_snap(app: tauri::AppHandle, state: SharedState, counter: Arc<AtomicU64>) {
    let current = counter.fetch_add(1, Ordering::Relaxed) + 1;
    thread::spawn(move || {
        thread::sleep(Duration::from_millis(420));
        if counter.load(Ordering::Relaxed) != current {
            return;
        }
        if let Some(w) = app.get_webview_window("main") {
            let _ = snap_window_inner(&w, &state);
        }
    });
}

// ── Tauri commands ──────────────────────────────────────────────────
#[tauri::command]
fn get_state(state: tauri::State<'_, SharedState>) -> Result<StatePayload, String> {
    let data = state.lock().map_err(|_| "state poisoned".to_string())?;
    Ok(StatePayload {
        current_state: data.current_state.clone(),
        orientation: data.settings.orientation.as_str().to_string(),
        auto_start: check_auto_start(),
    })
}

#[tauri::command]
fn set_orientation(
    orientation: String,
    window: WebviewWindow,
    state: tauri::State<'_, SharedState>,
) -> Result<(), String> {
    apply_orientation(&window, &state, orientation)
}

#[tauri::command]
fn set_auto_start(enabled: bool) -> Result<(), String> {
    if cfg!(target_os = "windows") {
        register_auto_start(enabled)
    } else {
        Err("仅支持 Windows 自动启动".to_string())
    }
}

#[tauri::command]
fn snap_window(
    window: WebviewWindow,
    state: tauri::State<'_, SharedState>,
) -> Result<(), String> {
    snap_window_inner(&window, &state)
}

#[tauri::command]
fn show_context_menu(
    app: tauri::AppHandle,
    window: WebviewWindow,
    x: f64,
    y: f64,
) -> Result<(), String> {
    let menu = MenuBuilder::new(&app)
        .text("horizontal", "横向")
        .text("vertical", "竖向")
        .separator()
        .text("auto_start", "开机自启")
        .separator()
        .text("quit", "退出")
        .build()
        .map_err(|e| e.to_string())?;

    window
        .popup_menu_at(
            &menu,
            Position::Physical(PhysicalPosition {
                x: x.round() as i32,
                y: y.round() as i32,
            }),
        )
        .map_err(|e| e.to_string())
}

#[tauri::command]
fn quit_app(app: tauri::AppHandle) {
    app.exit(0);
}

// ── Main ────────────────────────────────────────────────────────────
fn build_state() -> SharedState {
    Arc::new(Mutex::new(AppData {
        current_state: default_light_state(),
        settings: load_settings(),
    }))
}

fn main() {
    let state = build_state();
    let managed_state = state.clone();
    let menu_state = state.clone();
    let snap_state = state.clone();
    let move_counter = Arc::new(AtomicU64::new(0));
    let snap_counter = move_counter.clone();

    tauri::Builder::default()
        .manage(managed_state)
        .invoke_handler(tauri::generate_handler![
            get_state,
            set_orientation,
            set_auto_start,
            snap_window,
            show_context_menu,
            quit_app,
        ])
        .on_menu_event(move |app, event| {
            let id = event.id().as_ref();
            match id {
                "quit" => app.exit(0),
                "horizontal" | "vertical" => {
                    if let Some(w) = app.get_webview_window("main") {
                        let _ = apply_orientation(&w, &menu_state, id.to_string());
                    }
                }
                "auto_start" => {
                    let new_val = !check_auto_start();
                    let _ = register_auto_start(new_val);
                }
                _ => {}
            }
        })
        .on_window_event(move |window, event| {
            if window.label() != "main" {
                return;
            }
            if matches!(event, WindowEvent::Moved(_)) {
                schedule_snap(
                    window.app_handle().clone(),
                    snap_state.clone(),
                    snap_counter.clone(),
                );
            }
        })
        .setup(move |app| {
            let window = app
                .get_webview_window("main")
                .ok_or("main window missing")?;
            let _ = window.set_always_on_top(true);
            let app_handle = app.handle().clone();
            start_http_server(app_handle, state.clone());
            let _ = snap_window_inner(&window, &state);
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("运行 Agent Status Light 时出错");
}
