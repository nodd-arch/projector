mod commands;
mod db;
mod history;
mod models;
mod reference;

use commands::{
    add_history_entry, apply_background, apply_projection_font, get_books, get_chapter_count,
    get_chapter_verses, get_history, get_system_fonts, get_translations, get_verse_by_ids,
    hide_projection_window, navigate_projection, search_scripture, switch_projection_translation, apply_display_settings
};

use db::init_db;
use tauri::Manager;

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
pub struct ProjectionPayload {
    pub verse: models::Verse,
    pub translation_abbr: String,
}

#[tauri::command]
fn push_to_projection(
    app_handle: tauri::AppHandle,
    verse: models::Verse,
    translation_abbr: String,
) -> Result<(), String> {
    use tauri::Emitter;
    let payload = ProjectionPayload {
        verse,
        translation_abbr,
    };

    app_handle
        .emit_to("projection-screen", "verse-update", &payload)
        .map_err(|e| e.to_string())?;
    app_handle
        .emit_to("control-panel", "preview-sync", &payload)
        .map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
fn panic_clear(app_handle: tauri::AppHandle) -> Result<(), String> {
    use tauri::Emitter;
    app_handle
        .emit_to("projection-screen", "panic-clear", ())
        .map_err(|e| e.to_string())
}

#[tauri::command]
fn panic_restore(app_handle: tauri::AppHandle) -> Result<(), String> {
    use tauri::Emitter;
    app_handle
        .emit_to("projection-screen", "panic-restore", ())
        .map_err(|e| e.to_string())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            init_db(app).expect("failed to initialize database");
            crate::history::init_history_db(app).expect("failed to initialize history database");

            if let Some(win) = app.get_webview_window("projection-screen") {
                let win_clone = win.clone();
                win.on_window_event(move |event| {
                    if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                        api.prevent_close();
                        win_clone.hide().ok();
                    }
                });
            }

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            search_scripture,
            get_translations,
            push_to_projection,
            panic_clear,
            panic_restore,
            open_projection_window,
            get_verse_by_ids,
            switch_projection_translation,
            navigate_projection,
            locate_and_project,
            get_system_fonts,
            apply_projection_font,
            hide_projection_window,
            add_history_entry,
            get_history,
            get_books,
            get_chapter_count,
            get_chapter_verses,
            apply_background,
            apply_display_settings,
            apply_output_resolution
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[tauri::command]
fn open_projection_window(app_handle: tauri::AppHandle) -> Result<(), String> {
    use tauri::Manager;
    let win = app_handle
        .get_webview_window("projection-screen")
        .ok_or("projection-screen window not found")?;
    win.show().map_err(|e| e.to_string())?;
    win.set_focus().map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
fn locate_and_project(app_handle: tauri::AppHandle, width: i64, height: i64) -> Result<String, String> {
    let monitors = app_handle.available_monitors().map_err(|e| e.to_string())?;
    let win = app_handle
        .get_webview_window("projection-screen")
        .ok_or("projection-screen window not found")?;

    if monitors.len() > 1 {
        let target = &monitors[1];
        win.set_position(*target.position()).map_err(|e| e.to_string())?;
        win.set_size(*target.size()).map_err(|e| e.to_string())?;
        win.show().map_err(|e| e.to_string())?;
        win.set_fullscreen(true).map_err(|e| e.to_string())?;
        Ok(format!("Projecting to secondary monitor ({}x{})", target.size().width, target.size().height))
    } else {
        win.set_fullscreen(false).ok();
        win.set_size(tauri::Size::Physical(tauri::PhysicalSize {
            width: width as u32,
            height: height as u32,
        }))
        .map_err(|e| e.to_string())?;
        win.set_position(tauri::Position::Physical(tauri::PhysicalPosition { x: 100, y: 100 }))
            .map_err(|e| e.to_string())?;
        win.show().map_err(|e| e.to_string())?;
        win.set_focus().map_err(|e| e.to_string())?;
        Ok(format!("Opened at {}x{}. Share this window in Google Meet.", width, height))
    }
}

#[tauri::command]
fn apply_output_resolution(app_handle: tauri::AppHandle, width: i64, height: i64) -> Result<(), String> {
    let win = app_handle
        .get_webview_window("projection-screen")
        .ok_or("projection-screen window not found")?;
    win.set_fullscreen(false).ok();
    win.set_size(tauri::Size::Physical(tauri::PhysicalSize {
        width: width as u32,
        height: height as u32,
    }))
    .map_err(|e| e.to_string())
}
