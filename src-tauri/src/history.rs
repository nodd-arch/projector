use rusqlite::Connection;
use std::sync::Mutex;
use tauri::Manager;

pub struct HistoryState(pub Mutex<Connection>);

pub fn init_history_db(app: &tauri::App) -> rusqlite::Result<()> {
    let dir = app.path().app_data_dir().expect("no app data dir");
    std::fs::create_dir_all(&dir).ok();
    let conn = Connection::open(dir.join("history.db"))?;
    conn.execute_batch(
        "DROP TABLE IF EXISTS history;
         CREATE TABLE history (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            verse_json TEXT NOT NULL,
            translation_abbr TEXT NOT NULL,
            session_id INTEGER NOT NULL,
            created_at TEXT DEFAULT (datetime('now'))
        );",
    )?;
    app.manage(HistoryState(Mutex::new(conn)));
    Ok(())
}
