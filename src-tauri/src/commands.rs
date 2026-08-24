use crate::db::DbState;
use crate::models::Verse;
use crate::reference::{parse_query, ParsedQuery};
use rusqlite::params;
use tauri::{Manager, State};

#[tauri::command]
pub fn search_scripture(
    state: State<DbState>,
    query: String,
    translation_id: i64,
) -> Result<Vec<Verse>, String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;

    match parse_query(&query) {
        ParsedQuery::Reference {
            book,
            chapter,
            verse,
        } => get_reference(&conn, &book, chapter, verse, translation_id),
        ParsedQuery::Keyword { term, book } => keyword_search(&conn, &term, translation_id, book),
    }
}

fn get_reference(
    conn: &rusqlite::Connection,
    book: &str,
    chapter: i64,
    verse: Option<String>,
    translation_id: i64,
) -> Result<Vec<Verse>, String> {
    let sql = if verse.is_some() {
        "SELECT v.verseid, v.translationid, v.bookid, b.name, v.chapternumber,
                v.versenumber, v.versetext, v.haswj, v.hasfootnotes, v.segments, v.footnotes
         FROM verses v JOIN books b ON b.bookid = v.bookid
         WHERE LOWER(b.name) = LOWER(?1) AND v.chapternumber = ?2 AND v.versenumber = ?3 AND v.translationid = ?4
           AND v.translationid = ?4"
    } else {
        "SELECT v.verseid, v.translationid, v.bookid, b.name, v.chapternumber,
                v.versenumber, v.versetext, v.haswj, v.hasfootnotes, v.segments, v.footnotes
         FROM verses v JOIN books b ON b.bookid = v.bookid
         WHERE LOWER(b.name) = LOWER(?1) AND v.chapternumber = ?2 AND v.translationid = ?4
         ORDER BY CAST(v.versenumber AS INTEGER)"
    };

    let mut stmt = conn.prepare(sql).map_err(|e| e.to_string())?;
    let verse_param = verse.unwrap_or_default();

    let rows = stmt
        .query_map(
            params![book, chapter, verse_param, translation_id],
            row_to_verse,
        )
        .map_err(|e| e.to_string())?;

    collect_verses(rows)
}

fn keyword_search(
    conn: &rusqlite::Connection,
    term: &str,
    translation_id: i64,
    book: Option<String>,
) -> Result<Vec<Verse>, String> {
    let sql = if book.is_some() {
        "SELECT v.verseid, v.translationid, v.bookid, b.name, v.chapternumber,
                v.versenumber, v.versetext, v.haswj, v.hasfootnotes, v.segments, v.footnotes
         FROM verses_fts f
         JOIN verses v ON v.verseid = f.rowid
         JOIN books b ON b.bookid = v.bookid
         WHERE verses_fts MATCH ?1 AND v.translationid = ?2 AND LOWER(b.name) = LOWER(?3)
         LIMIT 50"
    } else {
        "SELECT v.verseid, v.translationid, v.bookid, b.name, v.chapternumber,
                v.versenumber, v.versetext, v.haswj, v.hasfootnotes, v.segments, v.footnotes
         FROM verses_fts f
         JOIN verses v ON v.verseid = f.rowid
         JOIN books b ON b.bookid = v.bookid
         WHERE verses_fts MATCH ?1 AND v.translationid = ?2
         LIMIT 50"
    };

    let mut stmt = conn.prepare(sql).map_err(|e| e.to_string())?;

    let rows = if let Some(book_name) = book {
        stmt.query_map(params![term, translation_id, book_name], row_to_verse)
    } else {
        stmt.query_map(params![term, translation_id], row_to_verse)
    }
    .map_err(|e| e.to_string())?;

    collect_verses(rows)
}

fn row_to_verse(row: &rusqlite::Row) -> rusqlite::Result<Verse> {
    let segments_json: Option<String> = row.get(9)?;
    let footnotes_json: Option<String> = row.get(10)?;

    Ok(Verse {
        verseid: row.get(0)?,
        translationid: row.get(1)?,
        bookid: row.get(2)?,
        book_name: row.get(3)?,
        chapternumber: row.get(4)?,
        versenumber: row.get(5)?,
        versetext: row.get(6)?,
        haswj: row.get::<_, i64>(7)? != 0,
        hasfootnotes: row.get::<_, i64>(8)? != 0,
        segments: segments_json.and_then(|s| serde_json::from_str(&s).ok()),
        footnotes: footnotes_json.and_then(|s| serde_json::from_str(&s).ok()),
    })
}

fn collect_verses(
    rows: impl Iterator<Item = rusqlite::Result<Verse>>,
) -> Result<Vec<Verse>, String> {
    rows.collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_translations(state: State<DbState>) -> Result<Vec<crate::models::Translation>, String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    let mut stmt = conn
        .prepare(
            "SELECT translationid, abbreviation, name FROM translations ORDER BY translationid",
        )
        .map_err(|e| e.to_string())?;

    let rows = stmt
        .query_map([], |row| {
            Ok(crate::models::Translation {
                translationid: row.get(0)?,
                abbreviation: row.get(1)?,
                name: row.get(2)?,
            })
        })
        .map_err(|e| e.to_string())?;

    rows.collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_verse_by_ids(
    state: State<DbState>,
    bookid: i64,
    chapternumber: i64,
    versenumber: String,
    translation_id: i64,
) -> Result<Option<Verse>, String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;

    let sql = "SELECT v.verseid, v.translationid, v.bookid, b.name, v.chapternumber,
                      v.versenumber, v.versetext, v.haswj, v.hasfootnotes, v.segments, v.footnotes
               FROM verses v JOIN books b ON b.bookid = v.bookid
               WHERE v.bookid = ?1 AND v.chapternumber = ?2 AND v.versenumber = ?3
                 AND v.translationid = ?4";

    let mut stmt = conn.prepare(sql).map_err(|e| e.to_string())?;
    let mut rows = stmt
        .query_map(
            params![bookid, chapternumber, versenumber, translation_id],
            row_to_verse,
        )
        .map_err(|e| e.to_string())?;

    match rows.next() {
        Some(r) => r.map(Some).map_err(|e| e.to_string()),
        None => Ok(None),
    }
}

#[tauri::command]
pub fn switch_projection_translation(
    app_handle: tauri::AppHandle,
    state: State<DbState>,
    bookid: i64,
    chapternumber: i64,
    versenumber: String,
    translation_id: i64,
    translation_abbr: String,
) -> Result<(), String> {
    use tauri::Emitter;

    let verse = {
        let conn = state.0.lock().map_err(|e| e.to_string())?;
        let sql = "SELECT v.verseid, v.translationid, v.bookid, b.name, v.chapternumber,
                          v.versenumber, v.versetext, v.haswj, v.hasfootnotes, v.segments, v.footnotes
                   FROM verses v JOIN books b ON b.bookid = v.bookid
                   WHERE v.bookid = ?1 AND v.chapternumber = ?2 AND v.versenumber = ?3
                     AND v.translationid = ?4";
        let mut stmt = conn.prepare(sql).map_err(|e| e.to_string())?;
        let mut rows = stmt
            .query_map(
                params![bookid, chapternumber, versenumber, translation_id],
                row_to_verse,
            )
            .map_err(|e| e.to_string())?;
        match rows.next() {
            Some(r) => r.map_err(|e| e.to_string())?,
            None => return Err("verse not found for that translation".into()),
        }
    };

    let payload = crate::ProjectionPayload {
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
pub fn navigate_projection(
    app_handle: tauri::AppHandle,
    state: State<DbState>,
    bookid: i64,
    chapternumber: i64,
    versenumber: String,
    translation_id: i64,
    translation_abbr: String,
    direction: i64, // 1 = next, -1 = previous
) -> Result<Option<Verse>, String> {
    use tauri::Emitter;

    let conn = state.0.lock().map_err(|e| e.to_string())?;

    let same_chapter_sql = if direction > 0 {
        "SELECT v.verseid, v.translationid, v.bookid, b.name, v.chapternumber,
                v.versenumber, v.versetext, v.haswj, v.hasfootnotes, v.segments, v.footnotes
         FROM verses v JOIN books b ON b.bookid = v.bookid
         WHERE v.bookid = ?1 AND v.chapternumber = ?2 AND v.translationid = ?3
           AND CAST(v.versenumber AS INTEGER) > CAST(?4 AS INTEGER)
         ORDER BY CAST(v.versenumber AS INTEGER) ASC LIMIT 1"
    } else {
        "SELECT v.verseid, v.translationid, v.bookid, b.name, v.chapternumber,
                v.versenumber, v.versetext, v.haswj, v.hasfootnotes, v.segments, v.footnotes
         FROM verses v JOIN books b ON b.bookid = v.bookid
         WHERE v.bookid = ?1 AND v.chapternumber = ?2 AND v.translationid = ?3
           AND CAST(v.versenumber AS INTEGER) < CAST(?4 AS INTEGER)
         ORDER BY CAST(v.versenumber AS INTEGER) DESC LIMIT 1"
    };

    let mut stmt = conn.prepare(same_chapter_sql).map_err(|e| e.to_string())?;
    let mut rows = stmt
        .query_map(
            params![bookid, chapternumber, translation_id, versenumber],
            row_to_verse,
        )
        .map_err(|e| e.to_string())?;

    let found = match rows.next() {
        Some(r) => Some(r.map_err(|e| e.to_string())?),
        None => {
            // Roll into the adjacent chapter (same book only — no cross-book rollover for now)
            let next_chapter = chapternumber + direction;
            let edge_sql = if direction > 0 {
                "SELECT v.verseid, v.translationid, v.bookid, b.name, v.chapternumber,
                        v.versenumber, v.versetext, v.haswj, v.hasfootnotes, v.segments, v.footnotes
                 FROM verses v JOIN books b ON b.bookid = v.bookid
                 WHERE v.bookid = ?1 AND v.chapternumber = ?2 AND v.translationid = ?3
                 ORDER BY CAST(v.versenumber AS INTEGER) ASC LIMIT 1"
            } else {
                "SELECT v.verseid, v.translationid, v.bookid, b.name, v.chapternumber,
                        v.versenumber, v.versetext, v.haswj, v.hasfootnotes, v.segments, v.footnotes
                 FROM verses v JOIN books b ON b.bookid = v.bookid
                 WHERE v.bookid = ?1 AND v.chapternumber = ?2 AND v.translationid = ?3
                 ORDER BY CAST(v.versenumber AS INTEGER) DESC LIMIT 1"
            };
            let mut estmt = conn.prepare(edge_sql).map_err(|e| e.to_string())?;
            let mut erows = estmt
                .query_map(params![bookid, next_chapter, translation_id], row_to_verse)
                .map_err(|e| e.to_string())?;
            match erows.next() {
                Some(r) => Some(r.map_err(|e| e.to_string())?),
                None => None, // hit the start/end of the book — nothing further to do
            }
        }
    };

    if let Some(ref verse) = found {
        let payload = crate::ProjectionPayload {
            verse: verse.clone(),
            translation_abbr,
        };
        app_handle
            .emit_to("projection-screen", "verse-update", &payload)
            .map_err(|e| e.to_string())?;
        app_handle
            .emit_to("control-panel", "preview-sync", &payload)
            .map_err(|e| e.to_string())?;
    }

    Ok(found)
}

#[tauri::command]
pub fn apply_projection_font(app_handle: tauri::AppHandle, font: String) -> Result<(), String> {
    use tauri::Emitter;
    app_handle
        .emit_to("projection-screen", "font-update", &font)
        .map_err(|e| e.to_string())?;
    app_handle
        .emit_to("control-panel", "font-update", &font)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_system_fonts() -> Vec<String> {
    use font_kit::source::SystemSource;
    let source = SystemSource::new();
    let mut names: Vec<String> = source
        .all_families()
        .unwrap_or_default()
        .into_iter()
        .collect();
    names.sort();
    names.dedup();
    names
}

#[tauri::command]
pub fn hide_projection_window(app_handle: tauri::AppHandle) -> Result<(), String> {
    let win = app_handle
        .get_webview_window("projection-screen")
        .ok_or("projection-screen window not found")?;
    win.hide().map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_history(
    history: State<crate::history::HistoryState>,
) -> Result<Vec<serde_json::Value>, String> {
    let conn = history.0.lock().map_err(|e| e.to_string())?;
    let mut stmt = conn
        .prepare("SELECT verse_json, translation_abbr, created_at, session_id FROM history ORDER BY id ASC LIMIT 500")
        .map_err(|e| e.to_string())?;
    let rows = stmt
        .query_map([], |row| {
            let verse_json: String = row.get(0)?;
            let translation_abbr: String = row.get(1)?;
            let created_at: String = row.get(2)?;
            let session_id: i64 = row.get(3)?;
            Ok(serde_json::json!({
                "verse": serde_json::from_str::<serde_json::Value>(&verse_json).unwrap_or_default(),
                "translation_abbr": translation_abbr,
                "created_at": created_at,
                "session_id": session_id,
            }))
        })
        .map_err(|e| e.to_string())?;
    rows.collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_books(state: State<DbState>) -> Result<Vec<crate::models::Book>, String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    let mut stmt = conn
        .prepare("SELECT bookid, testamentid, name FROM books ORDER BY bookid")
        .map_err(|e| e.to_string())?;
    let rows = stmt
        .query_map([], |row| {
            Ok(crate::models::Book {
                bookid: row.get(0)?,
                testamentid: row.get(1)?,
                name: row.get(2)?,
            })
        })
        .map_err(|e| e.to_string())?;
    rows.collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_chapter_count(
    state: State<DbState>,
    bookid: i64,
    translation_id: i64,
) -> Result<i64, String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    conn.query_row(
        "SELECT COALESCE(MAX(chapternumber), 0) FROM verses WHERE bookid = ?1 AND translationid = ?2",
        rusqlite::params![bookid, translation_id],
        |row| row.get(0),
    )
    .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_chapter_verses(
    state: State<DbState>,
    bookid: i64,
    chapternumber: i64,
    translation_id: i64,
) -> Result<Vec<Verse>, String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    let sql = "SELECT v.verseid, v.translationid, v.bookid, b.name, v.chapternumber,
                      v.versenumber, v.versetext, v.haswj, v.hasfootnotes, v.segments, v.footnotes
               FROM verses v JOIN books b ON b.bookid = v.bookid
               WHERE v.bookid = ?1 AND v.chapternumber = ?2 AND v.translationid = ?3
               ORDER BY CAST(v.versenumber AS INTEGER)";
    let mut stmt = conn.prepare(sql).map_err(|e| e.to_string())?;
    let rows = stmt
        .query_map(params![bookid, chapternumber, translation_id], row_to_verse)
        .map_err(|e| e.to_string())?;
    collect_verses(rows)
}

#[tauri::command]
pub fn apply_background(
    app_handle: tauri::AppHandle,
    config: BackgroundConfig,
) -> Result<(), String> {
    use tauri::Emitter;
    app_handle
        .emit_to("projection-screen", "background-update", &config)
        .map_err(|e| e.to_string())?;
    app_handle
        .emit_to("control-panel", "background-update", &config)
        .map_err(|e| e.to_string())
}

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
pub struct DisplaySettings {
    pub font_family: String,
    pub weight_normal: String,
    pub weight_wj: String,
    pub color_normal: String,
    pub color_wj: String,
    pub color_bracket: String,
}

#[tauri::command]
pub fn apply_display_settings(
    app_handle: tauri::AppHandle,
    settings: DisplaySettings,
) -> Result<(), String> {
    use tauri::Emitter;
    app_handle
        .emit_to("projection-screen", "display-settings-update", &settings)
        .map_err(|e| e.to_string())?;
    app_handle
        .emit_to("control-panel", "display-settings-update", &settings)
        .map_err(|e| e.to_string())
}

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
pub struct BackgroundConfig {
    pub kind: String,
    pub value: String,
    pub opacity: f64,
    pub position: String, // "left" | "center" | "right"
}

#[tauri::command]
pub fn add_history_entry(
    history: State<crate::history::HistoryState>,
    verse: Verse,
    translation_abbr: String,
    session_id: i64,
) -> Result<(), String> {
    let conn = history.0.lock().map_err(|e| e.to_string())?;

    let last: Option<(i64, String, i64)> = conn
        .query_row(
            "SELECT json_extract(verse_json, '$.verseid'), translation_abbr, session_id
             FROM history ORDER BY id DESC LIMIT 1",
            [],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
        )
        .ok();

    if let Some((last_verseid, last_abbr, last_session)) = last {
        if last_verseid == verse.verseid
            && last_abbr == translation_abbr
            && last_session == session_id
        {
            return Ok(()); // identical to the most recent entry in this session, skip
        }
    }

    let verse_json = serde_json::to_string(&verse).map_err(|e| e.to_string())?;
    conn.execute(
        "INSERT INTO history (verse_json, translation_abbr, session_id) VALUES (?1, ?2, ?3)",
        rusqlite::params![verse_json, translation_abbr, session_id],
    )
    .map_err(|e| e.to_string())?;
    Ok(())
}
