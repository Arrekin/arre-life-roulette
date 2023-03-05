use rusqlite::{Connection, Result, Row};
use crate::item_tag::ItemTag;
use crate::utils::Id;

pub type ItemId = Id<Item>;
#[derive(Debug, Clone)]
pub struct Item {
    pub id: ItemId,
    pub name: String,
    pub description: Option<String>,
    pub is_suspended: bool,
    pub is_finished: bool,
    pub tags: Vec<ItemTag>,
}

impl Item {
    pub fn create_new(conn: &Connection, name: impl AsRef<str>, description: &Option<impl AsRef<str>>) -> Result<Item> {
        let name = name.as_ref();
        let description = description.as_ref().map(|d| d.as_ref());
        conn.execute(
        "INSERT INTO items (name, description) VALUES (?1, ?2)",
        (name, description),
        )?;
        let mut stmt = conn.prepare("
            SELECT
             item_id, name, description, is_suspended, is_finished
            FROM items
            WHERE item_id = last_insert_rowid()
        ")?;
        Ok(stmt.query_row([], |row| {
            Item::from_row(row)
        })?)
    }
    pub fn from_row(row: &Row) -> Result<Item> {
        Ok(Item {
            id: row.get(0)?,
            name: row.get(1)?,
            description: row.get(2)?,
            is_suspended: row.get(3)?,
            is_finished: row.get(4)?,
            tags: vec![],
        })
    }

    pub fn delete(&self, conn: &Connection) -> Result<()> {
        conn.execute("DELETE FROM items WHERE item_id = ?1", (self.id,))?;
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
    #[case("Glorious Item", None)]
    #[case("Glorious Item", Some("Glorious Item Description".into()))]
    fn create_item_successful_then_delete(
        db_connection: &Connection,
        mut test_factory: TestFactory,
        #[case] item_name: String,
        #[case] item_description: Option<String>,
    ) {
        let item = Item::create_new(db_connection, &item_name, &item_description);
        assert!(item.is_ok(), "Could not create item, error: {:?}", item.err().unwrap());
        let item = item.unwrap();
        assert_eq!(
            item.name, item_name,
            "Item name is wrong. Expected {:?}, got {:?}",
            item_name, item.name
        );
        assert_eq!(
            item.description, item_description,
            "Item description is wrong. Expected {:?}, got {:?}",
            item_description, item.description
        );
        assert_eq!(item.is_suspended, false, "Item suspended after creation");
        assert_eq!(item.is_finished, false, "Item finished after creation");

        // Delete the item and check that there is no items in the table
        item.delete(&db_connection).unwrap();
        test_factory.assert_item_exist(&item, false);
        test_factory.assert_items_number(0);
    }
}
