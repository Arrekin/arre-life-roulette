use std::borrow::Borrow;
use std::collections::HashSet;
use rusqlite::{Connection, Result, Row};
use crate::errors::{ArreError, ArreResult};
use crate::item::{Item, item_get_all, ItemId};
use crate::utils::Id;

pub type ListId = Id<List>;

/// A list is a collection of items.
#[derive(Debug, Clone)]
pub struct List {
    pub id: Option<ListId>, // None indicates it's not persisted
    pub name: String,
    pub description: String,
}

pub fn list_create(conn: &Connection, name: impl AsRef<str>, description: impl AsRef<str>) -> ArreResult<List> {
    let name = name.as_ref();
    let description = description.as_ref();
    conn.execute(
        "INSERT INTO lists (name, description) VALUES (?1, ?2)",
        (name, description),
    )?;
    let mut stmt = conn.prepare("
        SELECT
         list_id, name, description
        FROM lists
        WHERE list_id = last_insert_rowid()
    ")?;
    Ok(stmt.query_row([], |row| {
        List::from_row(row)
    })?)
}

pub fn list_persist(conn: &Connection, list: &mut List) -> ArreResult<()> {
    conn.execute(
        "INSERT INTO lists (name, description) VALUES (?1, ?2)",
        (&list.name, &list.description),
    )?;
    let mut stmt = conn.prepare("
        SELECT
         list_id, name, description
        FROM lists
        WHERE list_id = last_insert_rowid()
    ")?;
    Ok(stmt.query_row([], |row| {
        list.update_from_row(row)
    })?)
}

pub fn list_update(conn: &Connection, list: &List) -> ArreResult<()> {
    conn.execute(
        "
        UPDATE lists
        SET name = ?1, description = ?2
        WHERE list_id = ?3
        ",
        (&list.name, &list.description, list.get_id()?),
    )?;
    Ok(())
}

pub fn list_get(conn: &Connection, id: ListId) -> ArreResult<List> {
    let mut stmt = conn.prepare("
        SELECT
         list_id, name, description
        FROM lists
        WHERE list_id = ?1
    ")?;
    Ok(stmt.query_row([id], |row| {
        List::from_row(row)
    })?)
}

pub fn list_get_all(conn: &Connection) -> Result<Vec<List>> {
    let mut stmt = conn.prepare("
        SELECT
         list_id, name, description
        FROM lists
    ")?;
    let results = stmt.query_map([], |row| {
        List::from_row(row)
    })?;
    results.collect::<Result<Vec<_>>>()
}

pub fn list_delete(conn: &Connection, id: ListId) -> ArreResult<()> {
    conn.execute("DELETE FROM item_list_map WHERE list_id = ?1", (*id,))?;
    conn.execute("DELETE FROM lists WHERE list_id = ?1", (*id,))?;
    Ok(())
}

pub fn list_items_add(
    conn: &Connection,
    list_id: ListId,
    mut items: impl IntoIterator<Item=ItemId>
) -> ArreResult<()> {
    let mut stmt = conn.prepare("INSERT INTO item_list_map (list_id, item_id) VALUES (?1, ?2)")?;
    for item_id in items {
        stmt.execute([*list_id, *item_id])?;
    }
    Ok(())
}

pub fn list_items_get<C>(conn: &Connection, list_id: ListId) -> Result<C>
where C: FromIterator<Item>
{
    let mut stmt = conn.prepare("
        SELECT i.item_id, i.name, i.description, i.is_suspended, i.is_finished
        FROM items i
        JOIN item_list_map ilm ON i.item_id = ilm.item_id
        WHERE ilm.list_id = ?1",
    )?;
    let results = stmt.query_map([*list_id], |row| {
        Item::from_row(row)
    })?;
    results.collect::<Result<C>>()
}

pub fn list_items_id_get<C>(conn: &Connection, list_id: ListId) -> Result<C>
where C: FromIterator<ItemId>
{
    let mut stmt = conn.prepare("
        SELECT i.item_id
        FROM items i
        JOIN item_list_map ilm ON i.item_id = ilm.item_id
        WHERE ilm.list_id = ?1",
    )?;
    let results = stmt.query_map([*list_id], |row| {
        Ok(ItemId::new(row.get(0)?))
    })?;
    results.collect::<Result<C>>()
}

/// Get all items that are not on the list
pub fn list_items_get_complement<C>(conn: &Connection, list_id: ListId) -> Result<C>
where C: FromIterator<Item>
{
    let mut stmt = conn.prepare("
        SELECT i.item_id, i.name, i.description, i.is_suspended, i.is_finished
        FROM items i
        WHERE i.item_id NOT IN (
          SELECT ilp.item_id
          FROM item_list_map ilp
          WHERE ilp.list_id = ?1
        )",
    )?;
    let results = stmt.query_map([*list_id], |row| {
        Item::from_row(row)
    })?;
    results.collect::<Result<C>>()
}

pub fn list_items_update(
    conn: &Connection,
    list_id: ListId,
    items: impl IntoIterator<Item=ItemId>
) -> ArreResult<()> {
    let items = items.into_iter().collect::<HashSet<ItemId>>();
    // First get current items
    let curr_items = list_items_id_get::<HashSet<_>>(conn, list_id)?;
    // Then add entries that are in items but not in curr_items as those are new
    list_items_add(conn, list_id, items.difference(&curr_items).copied())?;
    // Then delete entries that are in curr_items but not in items
    list_items_delete(conn, list_id, curr_items.difference(&items).copied())?;
    Ok(())
}

// delete specific items
pub fn list_items_delete(conn: &Connection, list_id: ListId, items: impl Iterator<Item=ItemId>) -> ArreResult<()> {
    let mut stmt = conn.prepare("
        DELETE FROM item_list_map
        WHERE list_id = ?1 AND item_id = ?2"
    )?;
    for item_id in items {
        stmt.execute([*list_id, *item_id])?;
    }
    Ok(())
}

impl List {
    pub fn from_row(row: &Row) -> Result<List> {
        Ok(List {
            id: Some(row.get(0)?),
            name: row.get(1)?,
            description: row.get(2)?,
        })
    }
    pub fn update_from_row(&mut self, row: &Row) -> Result<()> {
        self.id = Some(row.get(0)?);
        self.name = row.get(1)?;
        self.description = row.get(2)?;
        Ok(())
    }

    pub fn get_id(&self) -> ArreResult<ListId> {
        self.id.ok_or(ArreError::ItemNotPersisted().into())
    }
}

impl Default for List {
    fn default() -> Self {
        Self {
            id: None,
            name: String::new(),
            description: String::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use rstest::*;
    use rusqlite::Connection;
    use crate::item::items_to_ids;
    use crate::test_fixtures::{conn, TestFactory};
    use super::*;

    #[rstest]
    #[case("Glorious List", "")]
    #[case("Glorious List2", "Glorious List Description")]
    fn list_create_successful_then_delete(
        conn: Connection,
        #[case] list_name: String,
        #[case] list_description: String,
    ) -> ArreResult<()> {
        let list = list_create(&conn, &list_name, &list_description)?;
        assert_eq!(
            list.name, list_name,
            "Item name is wrong. Expected {:?}, got {:?}",
            list_name, list.name
        );
        assert_eq!(
            list.description, list_description,
            "Item description is wrong. Expected {:?}, got {:?}",
            list_description, list.description
        );
        assert!(list.id.is_some(), "Item from DB claims to not have id");

        // Delete the item and check that there is no items in the table
        list_delete(&conn, list.get_id()?)?;
        let mut stmt = conn.prepare("SELECT COUNT(*) FROM lists")?;
        assert_eq!( stmt.query_row([], |row| row.get::<usize, i64>(0))?, 0, "List was not deleted");
        Ok(())
    }

    #[rstest]
    fn list_add_remove_items(conn: Connection) -> ArreResult<()> {
        let mut tf = TestFactory::new(&conn);
        let mut list = list_create(&conn, "Glorious List", "")?;
        let mut list_id = list.get_id()?;
        let start_items = tf.create_items(5)?;
        let first_item_id = start_items[0].get_id()?;
        tf.assert_items_number(5)?;
        list_items_add(
            &conn, list_id,
            items_to_ids(&start_items)?,
        )?;
        tf.assert_items_number_in_list(list_id, 5);
        list_items_delete(&conn, list_id, std::iter::once(first_item_id))?;
        tf.assert_items_number_in_list(list_id, 4);
        tf.assert_item_in_list(first_item_id, list_id, false)?;
        Ok(())
    }

    #[rstest]
    fn persist_default(conn: Connection) -> ArreResult<()> {
        let mut list = List::default();
        assert_eq!(list.id.is_none(), true, "Item claims to have ID while not saved in DB");
        assert_eq!(list.get_id().is_err(), true, "get_id() should error when not saved in DB");

        list_persist(&conn, &mut list)?;
        assert_eq!(list.id.is_some(), true, "Item claims to not have ID after list_persist()");
        assert_eq!(list.get_id().is_ok(), true, "get_id() should work after list_persist()");
        Ok(())
    }

    #[rstest]
    fn list_items_update_successful(conn: Connection) -> ArreResult<()> {
        let mut tf = TestFactory::new(&conn);
        let list = list_create(&conn, "Glorious List", "")?;
        let list_id = list.get_id()?;
        let mut list_items = tf.create_items(3)?;
        tf.assert_items_number_in_list(list_id, 0);
        list_items_update(
            &conn, list_id,
            items_to_ids(&list_items)?
        )?;
        tf.assert_items_number_in_list(list_id, 3);

        // Create another 3 items and add them to the list while removing 2 of the old ones.
        // The final amount after the save should be 4
        list_items.pop();
        list_items.pop();
        list_items.extend(tf.create_items(3)?);
        list_items_update(&conn, list_id, items_to_ids(&list_items)?)?;
        tf.assert_items_number_in_list(list_id, 4);
        Ok(())
    }

}
