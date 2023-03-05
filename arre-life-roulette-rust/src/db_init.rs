use rusqlite::{Connection, Result};

pub fn initialize_database(conn: &Connection) -> Result<()> {
    conn.execute(
        "CREATE TABLE items (
            item_id INTEGER PRIMARY KEY,
            name TEXT NOT NULL,
            description TEXT NULL,
            is_suspended BOOLEAN NOT NULL DEFAULT 0 CHECK(is_suspended IN (0, 1)),
            is_finished BOOLEAN NOT NULL DEFAULT 0 CHECK(is_finished IN (0, 1))
        )",
        (),
    )?;
    conn.execute(
        "CREATE TABLE lists (
            list_id INTEGER PRIMARY KEY,
            name TEXT NOT NULL,
            description TEXT NULL
        )",
        (),
    )?;
    conn.execute(
        "CREATE TABLE item_list_map (
            list_id INTEGER,
            item_id INTEGER,
            PRIMARY KEY(list_id, item_id)
        )",
        (),
    )?;
    Ok(())
}