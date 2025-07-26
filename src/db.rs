
// src/db.rs

use rusqlite::{Connection, params, Result};
use chrono::{Local, Utc};
use crate::app::state::App;
use std::{fs, path::PathBuf};

/// Open (or create) the SQLite DB under XDG data dir:
///   $XDG_DATA_HOME/term-typist/term_typist.db
/// falling back to ~/.local/share.
pub fn open() -> Result<Connection> {
    // Determine the base data directory:
    // On Linux this is $XDG_DATA_HOME or ~/.local/share.
    let mut data_dir = dirs::data_dir()
        .unwrap_or_else(|| PathBuf::from("."));
    // Append your app’s folder
    data_dir.push("term-typist");
    // Make sure it exists
    fs::create_dir_all(&data_dir)
        .expect("could not create data dir");
    // Finally, the database file
    data_dir.push("term_typist.db");

    // Open & run CREATE TABLE IF NOT EXISTS…
    let conn = Connection::open(data_dir)?;
    conn.execute_batch(r#"
        PRAGMA foreign_keys = ON;

        CREATE TABLE IF NOT EXISTS tests (
            id              INTEGER PRIMARY KEY AUTOINCREMENT,
            started_at      TEXT    NOT NULL,
            duration_ms     INTEGER NOT NULL,
            mode            TEXT    NOT NULL,
            target_text     TEXT    NOT NULL,
            target_value    INTEGER NOT NULL,
            correct_chars   INTEGER NOT NULL,
            incorrect_chars INTEGER NOT NULL,
            wpm             REAL    NOT NULL,
            accuracy        REAL    NOT NULL
        );

        CREATE TABLE IF NOT EXISTS samples (
            test_id    INTEGER NOT NULL REFERENCES tests(id) ON DELETE CASCADE,
            elapsed_s  INTEGER NOT NULL,
            wpm        REAL    NOT NULL
        );
    "#)?;
    Ok(conn)
}

/// Inserts a finished test and its samples.
pub fn save_test(conn: &mut Connection, app: &App) -> Result<()> {
    let started_at = Local::now().to_rfc3339();
    let duration_ms = (app.elapsed_secs() * 1000) as i64;
    let mode = match app.selected_tab {
        0 => "time",
        1 => "words",
        _ => "zen",
    };
    let target_value = app.current_options()[app.selected_value] as i64;
    let diff = (app.correct_chars as i64) - (app.incorrect_chars as i64);
    let wpm = if duration_ms>0 {
        (diff.max(0) as f64)/5.0 / (duration_ms as f64/60000.0)
    } else { 0.0 };
    let acc = if app.correct_chars+app.incorrect_chars>0 {
        (app.correct_chars as f64)/( (app.correct_chars+app.incorrect_chars) as f64 )*100.0
    } else { 100.0 };

    let tx = conn.transaction()?;
    tx.execute(
        "INSERT INTO tests
           (started_at,duration_ms,mode,target_text,target_value,
            correct_chars,incorrect_chars,wpm,accuracy)
         VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9)",
        params![
            &started_at,
            &duration_ms,
            &mode,
            &app.target,
            &target_value,
            &(app.correct_chars as i64),
            &(app.incorrect_chars as i64),
            &wpm,
            &acc
        ],
    )?;
    let test_id = tx.last_insert_rowid();

    for &(elapsed_s, sample_wpm) in &app.samples {
        tx.execute(
            "INSERT INTO samples (test_id, elapsed_s, wpm) VALUES (?1,?2,?3)",
            params![&test_id, &(elapsed_s as i64), &sample_wpm],
        )?;
    }

    tx.commit()
}
