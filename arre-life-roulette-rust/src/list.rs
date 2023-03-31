use std::collections::HashSet;
use rusqlite::{Connection, Result, Row};
use crate::item::Item;
use crate::utils::Id;

pub type ListId = Id<List>;

/// A list is a collection of items.
/// If `is_new` is true, the id field is invalid as we had not asked yet the DB to assign a new id.
#[derive(Debug, Clone)]
pub struct List {
    pub is_new: bool, // Whether the list is saved in DB

    pub id: ListId,
    pub name: String,
    pub description: String,
    pub items: Vec<Item>,
}

impl List {
    pub fn create_new(conn: &Connection, name: impl AsRef<str>, description: impl AsRef<str>) -> Result<List> {
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
    pub fn from_row(row: &Row) -> Result<List> {
        Ok(List {
            is_new: false,

            id: row.get(0)?,
            name: row.get(1)?,
            description: row.get(2)?,
            items: vec![],
        })
    }

    pub fn load(conn: &Connection, id: i64) -> Result<List> {
        let mut stmt = conn.prepare("
            SELECT list_id, name, description
            FROM lists
            WHERE list_id = ?1
            ",
        )?;
        Ok(stmt.query_row([id], |row| {
            List::from_row(row)
        })?)
    }

    pub fn load_items(&mut self, conn: &Connection) -> Result<()> {
        if self.is_new {
            return Ok(());
        }
        let mut stmt = conn.prepare("
            SELECT i.item_id, i.name, i.description, i.is_suspended, i.is_finished
            FROM items i
            JOIN item_list_map ilm ON i.item_id = ilm.item_id
            WHERE ilm.list_id = ?1
            ",
        )?;
        self.items = stmt.query_map([self.id], |row| {
            Item::from_row(row)
        })?.collect::<Result<Vec<_>>>()?;
        Ok(())
    }

    pub fn get_items_not_on_list(&self, conn: &Connection) -> Result<Vec<Item>> {
        if self.is_new {
            return Ok(Item::get_all(conn)?)
        }
        let mut stmt = conn.prepare("
            SELECT i.item_id, i.name, i.description, i.is_suspended, i.is_finished
            FROM items i
            WHERE i.item_id NOT IN (
              SELECT ilp.item_id
              FROM item_list_map ilp
              WHERE ilp.list_id = ?1
            )",
        )?;
        let result = stmt.query_map([self.id], |row| {
            Item::from_row(row)
        })?.collect::<Result<Vec<_>>>()?;
        Ok(result)
    }

    pub fn save(&mut self, conn: &Connection) -> Result<()> {
        if self.is_new {
            conn.execute(
            "INSERT INTO lists (name, description) VALUES (?1, ?2)",
            (&self.name, &self.description),
            )?;
        } else {
            conn.execute("
            UPDATE lists SET name = ?1, description = ?2
            WHERE list_id = ?3
            ", (&self.name, &self.description, &self.id)
            )?;
        }
        // Copy current list of items and reload the original ones to make comparison of what changed
        let new_items = std::mem::replace(&mut self.items, vec![]).into_iter().collect::<HashSet<_>>();
        self.load_items(conn)?;
        let old_items = std::mem::replace(&mut self.items, vec![]).into_iter().collect::<HashSet<_>>();
        for item in new_items.difference(&old_items) {
            conn.execute(
            "INSERT INTO item_list_map (list_id, item_id) VALUES (?1, ?2)",
            (&self.id, item.id),
            )?;
        }
        for item in old_items.difference(&new_items) {
            conn.execute(
            "DELETE FROM item_list_map WHERE list_id = ?1 AND item_id = ?2",
            (&self.id, item.id),
            )?;
        }
        self.load_items(conn)?;
        self.is_new = false;
        Ok(())
    }

    /// Add item to list. Does auto-save the change to the DB.
    pub fn add_item(&mut self, conn: &Connection, item: Item) -> Result<()> {
        let mut stmt = conn.prepare("INSERT INTO item_list_map (list_id, item_id) VALUES (?1, ?2)")?;
        stmt.execute([*self.id, *item.id])?;
        self.items.push(item);
        Ok(())
    }

    /// Remove item in given position from list. Does auto-save the change to the DB.
    pub fn remove_item(&mut self, conn: &Connection, id: usize) -> Result<Item> {
        let item = self.items.remove(id);
        let mut stmt = conn.prepare("
            DELETE FROM item_list_map
            WHERE list_id = ?1 AND item_id = ?2
        ")?;
        stmt.execute([*self.id, *item.id])?;
        Ok(item)
    }

    pub fn delete(&self, conn: &Connection) -> Result<()> {
        // First delete all related items
        conn.execute("DELETE FROM item_list_map WHERE list_id = ?1", (self.id,))?;
        conn.execute("DELETE FROM lists WHERE list_id = ?1", (self.id,))?;
        Ok(())
    }
}

impl Default for List {
    fn default() -> Self {
        Self {
            is_new: true,

            id: 0.into(),
            name: String::new(),
            description: String::new(),
            items: vec![],
        }
    }
}

#[cfg(test)]
mod tests {
    use rstest::*;
    use rusqlite::Connection;
    use crate::test_fixtures::{db_connection, TestFactory, test_factory};
    use super::*;

    #[rstest]
    #[case("Glorious List", "")]
    #[case("Glorious List2", "Glorious List Description")]
    fn create_list_successful_then_delete(
        db_connection: &Connection,
        #[case] list_name: String,
        #[case] list_description: String,
    ) {
        let list = List::create_new(db_connection, &list_name, &list_description);
        assert!(list.is_ok(), "Could not create item, error: {:?}", list.err().unwrap());
        let list = list.unwrap();
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
        assert_eq!(list.is_new, false, "Item from DB claims to be new");

        // Delete the item and check that there is no items in the table
        list.delete(&db_connection).unwrap();
        let mut stmt = db_connection.prepare("SELECT COUNT(*) FROM lists").unwrap();
        assert_eq!( stmt.query_row([], |row| row.get::<usize, i64>(0)).unwrap(), 0, "List was not deleted");
    }

    #[rstest]
    fn list_add_remove_items(db_connection: &Connection, mut test_factory: TestFactory) {
        let mut list = List::create_new(db_connection, "Glorious List", "").unwrap();
        let start_items = test_factory.create_items(5);
        let first_item = start_items[0].clone();
        test_factory.assert_items_number(5);
        start_items.into_iter().for_each(|item| list.add_item(db_connection, item).unwrap());
        test_factory.assert_items_number_in_list(&list, 5);
        list.remove_item(&db_connection, 0).unwrap();
        test_factory.assert_items_number_in_list(&list, 4);
        test_factory.assert_item_in_list(&first_item, &list, false);
    }

    #[rstest]
    fn save_default(db_connection: &Connection) {
        let mut list = List::default();
        assert_eq!(list.is_new, true, "Item claims to be not new");

        list.save(db_connection).unwrap();
        assert_eq!(list.is_new, false, "Item claims to be new after save");
    }

    #[rstest]
    fn save(db_connection: &Connection, mut test_factory: TestFactory) {
        let mut list = List::create_new(db_connection, "Glorious List", "").unwrap();
        list.items = test_factory.create_items(3);
        test_factory.assert_items_number_in_list(&list, 0);
        list.save(db_connection).unwrap();
        test_factory.assert_items_number_in_list(&list, 3);

        // Create another 3 items and add them to the list while removing 2 of the old ones.
        // The final amount after the save should be 4
        list.items.pop();
        list.items.pop();
        list.items.extend(test_factory.create_items(3));
        list.save(db_connection).unwrap();
        test_factory.assert_items_number_in_list(&list, 4);
    }

}
