use crate::core::Core;
use std::sync::Arc;
use tauri::{Emitter, Manager};

fn menu(app: &tauri::AppHandle, de: bool) -> tauri::Result<tauri::menu::Menu<tauri::Wry>> {
    use tauri::menu::{Menu, MenuItem};
    let show = MenuItem::with_id(
        app,
        "tray-show",
        if de {
            "Local Studio öffnen"
        } else {
            "Open Local Studio"
        },
        true,
        None::<&str>,
    )?;
    let exit = MenuItem::with_id(
        app,
        "tray-exit",
        if de { "Beenden …" } else { "Exit …" },
        true,
        None::<&str>,
    )?;
    Menu::with_items(app, &[&show, &exit])
}
pub fn refresh(app: &tauri::AppHandle, de: bool) -> Result<(), String> {
    if let Some(tray) = app.tray_by_id("background") {
        tray.set_menu(Some(menu(app, de).map_err(|e| e.to_string())?))
            .map_err(|e| e.to_string())?;
    }
    Ok(())
}

pub fn install(app: &tauri::App) -> Result<(), Box<dyn std::error::Error>> {
    use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};
    let de = app.state::<Arc<Core>>().current_settings()?.language == "de";
    let menu = menu(app.handle(), de)?;
    let icon = app
        .default_window_icon()
        .ok_or("Missing application icon")?
        .clone();
    let tray = TrayIconBuilder::with_id("background")
        .icon(icon)
        .tooltip("Local Studio")
        .menu(&menu)
        .show_menu_on_left_click(false)
        .on_menu_event(|app, event| match event.id.as_ref() {
            "tray-show" => restore(app),
            "tray-exit" => {
                restore(app);
                let _ = app.emit_to("main", "app-exit-request", ());
            }
            _ => {}
        })
        .on_tray_icon_event(|tray, event| {
            if matches!(
                event,
                TrayIconEvent::Click {
                    button: MouseButton::Left,
                    button_state: MouseButtonState::Up,
                    ..
                }
            ) {
                restore(tray.app_handle());
            }
        })
        .build(app)?;
    tray.set_visible(false)?;
    Ok(())
}
pub(crate) fn restore(app: &tauri::AppHandle) {
    if let Some(w) = app.get_webview_window("main") {
        let _ = w.show();
        let _ = w.unminimize();
        let _ = w.set_focus();
    }
    if let Some(t) = app.tray_by_id("background") {
        let _ = t.set_visible(false);
    }
}
#[tauri::command]
pub fn background_hide(
    app: tauri::AppHandle,
    core: tauri::State<'_, Arc<Core>>,
) -> Result<bool, String> {
    if !core.current_settings()?.minimize_to_tray {
        return Ok(false);
    }
    let t = app.tray_by_id("background").ok_or("tray_unavailable")?;
    t.set_visible(true).map_err(|e| e.to_string())?;
    app.get_webview_window("main")
        .ok_or("tray_unavailable")?
        .hide()
        .map_err(|e| e.to_string())?;
    Ok(true)
}
#[tauri::command]
pub fn desktop_accent() -> Option<String> {
    let mut color = 0u32;
    let mut opaque = 0i32;
    let result = unsafe {
        windows_sys::Win32::Graphics::Dwm::DwmGetColorizationColor(&mut color, &mut opaque)
    };
    if result < 0 {
        None
    } else {
        Some(format!("#{:06x}", color & 0xffffff))
    }
}
