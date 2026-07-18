use rusqlite::Connection;
use std::sync::Mutex;
use tauri::Manager;

pub struct DbState(pub Mutex<Connection>);

pub fn init_db(app: &tauri::App) -> rusqlite::Result<()> {
    let db_path = app
        .path()
        .resolve("bible.db", tauri::path::BaseDirectory::Resource)
        .expect("bible.db must ship as a bundled resource");

    let conn = Connection::open(db_path)?;
    conn.execute_batch(
        "PRAGMA journal_mode = WAL;
         PRAGMA synchronous = NORMAL;
         PRAGMA temp_store = MEMORY;",
    )?;

    app.manage(DbState(Mutex::new(conn)));
    Ok(())
}
