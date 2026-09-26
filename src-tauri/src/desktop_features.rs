use crate::core::Core;
use std::sync::Arc;
use tauri::{Emitter, Manager};

static ACCENT_ICON: std::sync::OnceLock<std::sync::Mutex<Option<isize>>> =
    std::sync::OnceLock::new();

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

#[tauri::command]
pub fn window_accent_icon(png: Vec<u8>, app: tauri::AppHandle) -> Result<(), String> {
    validate_window_icon(&png)?;
    set_window_accent_icon(&png, &app)
}

fn validate_window_icon(png: &[u8]) -> Result<(), String> {
    if png.is_empty() || png.len() > 128 * 1024 {
        return Err("window_icon_invalid".into());
    }
    let decoder = png::Decoder::new(std::io::Cursor::new(&png));
    let mut reader = decoder.read_info().map_err(|_| "window_icon_invalid")?;
    if reader.info().width != 64 || reader.info().height != 64 {
        return Err("window_icon_invalid".into());
    }
    let mut pixels = vec![0; reader.output_buffer_size()];
    reader
        .next_frame(&mut pixels)
        .map_err(|_| "window_icon_invalid")?;
    Ok(())
}

fn set_window_accent_icon(png: &[u8], app: &tauri::AppHandle) -> Result<(), String> {
    let mut owned = ACCENT_ICON
        .get_or_init(|| std::sync::Mutex::new(None))
        .lock()
        .map_err(|_| "window_icon_update")?;
    let window = app
        .get_webview_window("main")
        .ok_or("window_icon_update")?;
    let hwnd = window.hwnd().map_err(|_| "window_icon_update")?.0 as _;
    use windows_sys::Win32::UI::WindowsAndMessaging::*;
    let icon = unsafe {
        CreateIconFromResourceEx(
            png.as_ptr(),
            png.len() as u32,
            1,
            0x0003_0000,
            64,
            64,
            LR_DEFAULTCOLOR,
        )
    };
    if icon.is_null() {
        return Err("window_icon_update".into());
    }
    unsafe {
        SendMessageW(hwnd, WM_SETICON, ICON_BIG as usize, icon as isize);
        SendMessageW(hwnd, WM_SETICON, ICON_SMALL as usize, icon as isize);
        SendMessageW(hwnd, WM_SETICON, ICON_SMALL2 as usize, icon as isize);
        if SendMessageW(hwnd, WM_GETICON, ICON_BIG as usize, 0) != icon as isize {
            DestroyIcon(icon);
            return Err("window_icon_update".into());
        }
    }
    if let Some(previous) = owned.replace(icon as isize) {
        unsafe { DestroyIcon(previous as _) };
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn icon(width: u32, height: u32) -> Vec<u8> {
        let mut bytes = Vec::new();
        let mut encoder = png::Encoder::new(&mut bytes, width, height);
        encoder.set_color(png::ColorType::Rgba);
        encoder.set_depth(png::BitDepth::Eight);
        encoder
            .write_header()
            .unwrap()
            .write_image_data(&vec![255; (width * height * 4) as usize])
            .unwrap();
        bytes
    }

    #[test]
    fn taskbar_icon_accepts_only_a_bounded_decodable_64_pixel_png() {
        assert!(validate_window_icon(&icon(64, 64)).is_ok());
        assert_eq!(validate_window_icon(&icon(32, 32)).unwrap_err(), "window_icon_invalid");
        assert_eq!(validate_window_icon(&[0; 16]).unwrap_err(), "window_icon_invalid");
        assert_eq!(
            validate_window_icon(&vec![0; 128 * 1024 + 1]).unwrap_err(),
            "window_icon_invalid"
        );
    }
}
