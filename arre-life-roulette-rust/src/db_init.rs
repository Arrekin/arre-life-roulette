use rusqlite::{Connection, Result};

pub fn initialize_database(conn: &Connection) -> Result<()> {
    initialize_items_table(conn)?;
    conn.execute(
        "CREATE TABLE item_stats (
            item_id INTEGER PRIMARY KEY,
            created_date TEXT NOT NULL,
            updated_date TEXT NOT NULL,
            times_worked INTEGER NOT NULL DEFAULT 0,
            time_spent INTEGER NOT NULL DEFAULT 0
        )",
        (),
    )?;
    conn.execute(
        "CREATE TABLE lists (
            list_id INTEGER PRIMARY KEY,
            created_date TEXT NOT NULL,
            updated_date TEXT NOT NULL,
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

fn initialize_items_table(conn: &Connection) -> Result<()> {
    conn.execute_batch("
        CREATE TABLE items (
            item_id INTEGER PRIMARY KEY,
            created_date TEXT NOT NULL,
            updated_date TEXT NOT NULL,
            name TEXT NOT NULL,
            description TEXT NULL,
            is_suspended BOOLEAN NOT NULL DEFAULT 0 CHECK(is_suspended IN (0, 1)),
            is_finished BOOLEAN NOT NULL DEFAULT 0 CHECK(is_finished IN (0, 1))
        );
        CREATE VIRTUAL TABLE items_search_index USING fts5(name, description, tokenize=trigram);
        CREATE TRIGGER after_item_insert AFTER INSERT ON items BEGIN
          INSERT INTO items_search_index (
            rowid,
            name,
            description
          )
          VALUES(
            new.item_id,
            new.name,
            new.description
          );
        END;
        CREATE TRIGGER after_items_update UPDATE OF name, description ON items BEGIN
          UPDATE items_search_index
          SET
            name = new.name,
            description = new.description
          WHERE rowid = old.item_id;
        END;
        CREATE TRIGGER after_items_delete AFTER DELETE ON items BEGIN
            DELETE FROM items_search_index WHERE rowid = old.item_id;
        END;
        "
    )?;
    Ok(())
}