use rusqlite::{Connection, Result, Row, ToSql};
use crate::item::Item;
use crate::item_tag::ItemTag;
use crate::utils::Id;

pub type ListId = Id<List>;

#[derive(Debug)]
pub struct List {
    pub id: ListId,
    pub name: String,
    pub description: Option<String>,
    pub items: Vec<Item>,
}

impl List {
    pub fn create_new(conn: &Connection, name: impl AsRef<str>, description: &Option<impl AsRef<str>>) -> Result<List> {
        let name = name.as_ref();
        let description = description.as_ref().map(|d| d.as_ref());
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
        let mut stmt = conn.prepare("
            SELECT item_id, name, description
            FROM items i
            JOIN item_list_map ilm ON i.item_id = ilm.item_id
            JOIN lists l ON ilm.list_id = l.list_id
            WHERE list_id = ?1
            ",
        )?;
        self.items = stmt.query_map([self.id], |row| {
            Item::from_row(row)
        })?.collect::<Result<Vec<_>>>()?;
        Ok(())
    }

    /// Updates base properties. Does not manage relations
    pub fn update(&self, conn: &Connection) -> Result<()> {
        conn.execute("
            UPDATE lists SET name = ?1, description = ?2
            WHERE list_id = ?3
            ", (&self.name, &self.description, &self.id)
        )?;
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
        conn.execute("DELETE FROM lists WHERE list_id = ?1", (self.id,))?;
        Ok(())
    }
}


#[cfg(test)]
mod tests {
    use rstest::*;
    use rusqlite::Connection;
    use crate::test_fixtures::{db_connection, TestFactory, test_factory};
    use super::*;

    #[rstest]
    #[case("Glorious List", None)]
    #[case("Glorious List2", Some("Glorious List Description".into()))]
    fn create_list_successful_then_delete(
        db_connection: &Connection,
        #[case] list_name: String,
        #[case] list_description: Option<String>
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

        // Delete the item and check that there is no items in the table
        list.delete(&db_connection).unwrap();
        let mut stmt = db_connection.prepare("SELECT COUNT(*) FROM lists").unwrap();
        assert_eq!( stmt.query_row([], |row| row.get::<usize, i64>(0)).unwrap(), 0, "List was not deleted");
    }

    #[rstest]
    fn list_add_remove_items(db_connection: &Connection, mut test_factory: TestFactory) {
        let mut list = List::create_new(db_connection, "Glorious List", &None::<&str>).unwrap();
        let start_items = test_factory.create_items(5);
        let first_item = start_items[0].clone();
        test_factory.assert_items_number(5);
        start_items.into_iter().for_each(|item| list.add_item(db_connection, item).unwrap());
        test_factory.assert_items_number_in_list(&list, 5);
        list.remove_item(&db_connection, 0).unwrap();
        test_factory.assert_items_number_in_list(&list, 4);
        test_factory.assert_item_in_list(&first_item, &list, false);
    }
}
