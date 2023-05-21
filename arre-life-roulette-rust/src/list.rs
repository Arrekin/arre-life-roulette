use std::collections::HashSet;
use chrono::Utc;
use rusqlite::{Connection, Result, Row};
use crate::errors::{ArreError, ArreResult};
use crate::item::{Item, ItemId};
use crate::utils::{ArreDateTime, Id};

pub fn list_create(conn: &Connection, name: impl AsRef<str>, description: impl AsRef<str>) -> ArreResult<List> {
    let name = name.as_ref();
    let description = description.as_ref();
    let dt = Utc::now().to_string();
    conn.execute("
        INSERT INTO lists (created_date, updated_date, name, description) VALUES (?1, ?2, ?3, ?4)
        ", (dt.clone(), dt.clone(), name, description),
    )?;
    let mut stmt = conn.prepare("
        SELECT
         list_id, created_date, updated_date, name, description
        FROM lists
        WHERE list_id = last_insert_rowid()
    ")?;
    Ok(stmt.query_row([], |row| {
        List::from_row(row)
    })?)
}

pub fn list_persist(conn: &Connection, list: &mut List) -> ArreResult<()> {
    let dt = Utc::now().to_string();
    conn.execute("
        INSERT INTO lists (created_date, updated_date, name, description) VALUES (?1, ?2, ?3, ?4)
        ", (dt.clone(), dt.clone(), &list.name, &list.description),
    )?;
    let mut stmt = conn.prepare("
        SELECT
         list_id, created_date, updated_date, name, description
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
        SET
         updated_date = ?1, name = ?2, description = ?3
        WHERE list_id = ?4
        ", (Utc::now().to_string(), &list.name, &list.description, list.get_id()?),
    )?;
    Ok(())
}

pub fn list_get(conn: &Connection, id: ListId) -> ArreResult<List> {
    let mut stmt = conn.prepare("
        SELECT
         list_id, created_date, updated_date, name, description
        FROM lists
        WHERE list_id = ?1
    ")?;
    Ok(stmt.query_row([id], |row| {
        List::from_row(row)
    })?)
}

pub fn list_get_all<C>(conn: &Connection) -> Result<C>
where C: FromIterator<List>
{
    let mut stmt = conn.prepare("
        SELECT
         list_id, created_date, updated_date, name, description
        FROM lists
    ")?;
    let results = stmt.query_map([], |row| {
        List::from_row(row)
    })?;
    results.collect::<Result<C>>()
}

pub fn list_search<C>(conn: &Connection, search_term: impl AsRef<str>) -> ArreResult<C>
where C: FromIterator<List>
{
    let search_term = search_term.as_ref();
    let mut stmt = conn.prepare("
        SELECT
         l.list_id, l.created_date, l.updated_date, l.name, l.description
        FROM lists l
        JOIN (
            SELECT
             rowid, rank
            FROM
             lists_search_index
            WHERE
                lists_search_index MATCH ?1
        ) search ON l.list_id = search.rowid
        ORDER BY search.rank DESC
    ")?;
    let result = stmt.query_map([search_term], |row| {
        List::from_row(row)
    })?.collect::<Result<C>>()?;
    Ok(result)
}

pub fn list_delete(conn: &Connection, id: ListId) -> ArreResult<()> {
    conn.execute("DELETE FROM item_list_map WHERE list_id = ?1", (*id,))?;
    conn.execute("DELETE FROM lists WHERE list_id = ?1", (*id,))?;
    Ok(())
}

pub fn list_items_add(
    conn: &Connection,
    list_id: ListId,
    items: impl IntoIterator<Item=ItemId>
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
        SELECT i.item_id, i.created_date, i.updated_date, i.name, i.description, i.is_suspended, i.is_finished
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
        SELECT i.item_id, i.created_date, i.updated_date, i.name, i.description, i.is_suspended, i.is_finished
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

pub type ListId = Id<List>;
/// A list is a collection of items.
#[derive(Debug, Clone)]
pub struct List {
    pub id: Option<ListId>, // None indicates it's not persisted
    pub created_date: ArreDateTime<Utc>,
    pub modified_date: ArreDateTime<Utc>,
    pub name: String,
    pub description: String,
}

impl List {
    pub fn from_row(row: &Row) -> Result<List> {
        Ok(List {
            id: Some(row.get(0)?),
            created_date: row.get(1)?,
            modified_date: row.get(2)?,
            name: row.get(3)?,
            description: row.get(4)?,
        })
    }
    pub fn update_from_row(&mut self, row: &Row) -> Result<()> {
        self.id = Some(row.get(0)?);
        self.created_date = row.get(1)?;
        self.modified_date = row.get(2)?;
        self.name = row.get(3)?;
        self.description = row.get(4)?;
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
            created_date: Utc::now().into(),
            modified_date: Utc::now().into(),
            name: String::new(),
            description: String::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use rstest::*;
    use rusqlite::Connection;
    use crate::item::{items_to_ids, item_create};
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
        let list = list_create(&conn, "Glorious List", "")?;
        let list_id = list.get_id()?;
        let start_items = tf.create_items(5)?;
        let first_item_id = start_items[0].get_id()?;
        tf.assert_items_number(5)?;
        list_items_add(
            &conn, list_id,
            items_to_ids::<_, Vec<_>>(start_items.iter())?,
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
            items_to_ids::<_, Vec<_>>(list_items.iter())?
        )?;
        tf.assert_items_number_in_list(list_id, 3);

        // Create another 3 items and add them to the list while removing 2 of the old ones.
        // The final amount after the save should be 4
        list_items.pop();
        list_items.pop();
        list_items.extend(tf.create_items(3)?);
        list_items_update(&conn, list_id, items_to_ids::<_, Vec<_>>(list_items.iter())?)?;
        tf.assert_items_number_in_list(list_id, 4);
        Ok(())
    }


    #[rstest]
    #[case("Zero", 0)]
    #[case("onE", 1)]
    #[case("TwO", 2)]
    #[case("three", 3)]
    fn list_search_successful(
        conn: Connection,
        #[case] search_term: String,
        #[case] expected_number_of_lists: usize
    ) -> ArreResult<()> {
        list_create(&conn, "One", "Not number three at all")?;
        list_create(&conn, "Not ThRee", "but it's number TWO!")?;
        list_create(&conn, "Finally Three :o", "Not two, it means :(")?;
        let items = list_search::<Vec<_>>(&conn, search_term)?;
        assert_eq!(items.len(), expected_number_of_lists);
        Ok(())
    }

    #[rstest]
    fn list_search_after_update(conn: Connection) -> ArreResult<()> {
        // Create a single item and make sure it does not appear in list search results
        item_create(&conn, "Glorious Item", "Beyond Comprehension")?;
        // Create a list with similar name and description
        let mut list = list_create(&conn, "Glorious List", "Beyond Comprehension")?;

        let search_result = list_search::<Vec<_>>(&conn, "Glorious")?;
        assert_eq!(search_result.len(), 1, "Expected 1 search result, got {:?}", search_result);
        assert_eq!(&search_result[0].name as &str, "Glorious List");

        list.name = "Heavenly List".into();
        list_update(&conn, &list)?;
        assert_eq!(list_search::<Vec<_>>(&conn, "Glorious")?.len(), 0);
        assert_eq!(&list_search::<Vec<_>>(&conn, "Heavenly")?[0].name as &str, "Heavenly List");
        Ok(())
    }

    #[rstest]
    fn list_search_index_purge_after_delete(conn: Connection) -> ArreResult<()> {
        let count_fn = |conn: &Connection| -> Result<i64> {
            conn.query_row_and_then(
                "SELECT count(*) FROM lists_search_index;",
                (),
                |row| {
                    row.get(0)
                }
            )
        };
        let mut tf = TestFactory::new(&conn);
        assert_eq!(count_fn(&conn)?, 0);
        let lists = tf.create_lists(10)?;
        assert_eq!(count_fn(&conn)?, 10);
        lists.into_iter().for_each(|list| list_delete(&conn, list.get_id().unwrap()).unwrap());
        assert_eq!(count_fn(&conn)?, 0);
        Ok(())
    }

}
