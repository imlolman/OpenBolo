use tauri::{AppHandle, Emitter, Manager, WebviewUrl, WebviewWindowBuilder};

pub fn ensure_overlay(app: &AppHandle) {
    if app.get_webview_window("overlay").is_some() {
        return;
    }
    let _ = WebviewWindowBuilder::new(app, "overlay", WebviewUrl::App("overlay.html".into()))
        .title("")
        .inner_size(90.0, 26.0)
        .always_on_top(true)
        .decorations(false)
        .transparent(true)
        .skip_taskbar(true)
        .visible(false)
        .resizable(false)
        .build();
}

pub fn show(app: &AppHandle, mode: &str) {
    ensure_overlay(app);
    if let Some(window) = app.get_webview_window("overlay") {
        // Position bottom-center of primary monitor
        if let Ok(Some(monitor)) = window.primary_monitor() {
            let screen = monitor.size();
            let scale = monitor.scale_factor();
            let w = 90.0;
            let h = 26.0;
            let x = (screen.width as f64 / scale - w) / 2.0;
            let y = screen.height as f64 / scale - h - 40.0;
            window.set_position(tauri::Position::Logical(tauri::LogicalPosition::new(x, y))).ok();
        }
        window.show().ok();
        window.emit(&format!("show-{}", mode), ()).ok();
    }
}

pub fn show_processing(app: &AppHandle) {
    if let Some(window) = app.get_webview_window("overlay") {
        window.emit("show-processing", ()).ok();
    }
}

pub fn hide(app: &AppHandle) {
    if let Some(window) = app.get_webview_window("overlay") {
        window.hide().ok();
    }
}

pub fn emit_amplitude(app: &AppHandle, rms: f32) {
    let a = (rms * 20.0).min(1.0);
    if let Some(window) = app.get_webview_window("overlay") {
        window.emit("amplitude-update", a).ok();
    }
}
