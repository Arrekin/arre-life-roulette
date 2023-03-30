use std::hash::{Hash, Hasher};
use rusqlite::{Connection, Result, Row};
use crate::item_tag::ItemTag;
use crate::utils::Id;

pub type ItemId = Id<Item>;
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Item {
    pub is_new: bool,

    pub id: ItemId,
    pub name: String,
    pub description: String,
    pub is_suspended: bool,
    pub is_finished: bool,
    pub tags: Vec<ItemTag>,
}

impl Item {
    pub fn create_new(conn: &Connection, name: impl AsRef<str>, description: impl AsRef<str>) -> Result<Item> {
        let name = name.as_ref();
        let description = description.as_ref();
        conn.execute(
        "INSERT INTO items (name, description, is_suspended, is_finished) VALUES (?1, ?2, false, false)",
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
            is_new: false,

            id: row.get(0)?,
            name: row.get(1)?,
            description: row.get(2)?,
            is_suspended: row.get(3)?,
            is_finished: row.get(4)?,
            tags: vec![],
        })
    }

    /// Updates base properties. Does not manage relations
    pub fn save(&mut self, conn: &Connection) -> Result<()> {
        if self.is_new {
            conn.execute(
            "INSERT INTO items (name, description, is_suspended, is_finished) VALUES (?1, ?2, false, false)",
            (&self.name, &self.description),
            )?;
        } else {
            conn.execute("
            UPDATE items
            SET name = ?1, description = ?2, is_suspended = ?3, is_finished = ?4
            WHERE item_id = ?5
            ", (&self.name, &self.description, &self.is_suspended, &self.is_finished, self.id),
            )?;
        }
        self.is_new = false;
        Ok(())
    }

    pub fn load(conn: &Connection, id: impl Into<ItemId>) -> Result<Item> {
        let mut stmt = conn.prepare("
            SELECT item_id, name, description, is_suspended, is_finished
            FROM items
            WHERE item_id = ?1
        ")?;
        Ok(stmt.query_row([id.into()], |row| {
            Item::from_row(row)
        })?)
    }

    pub fn delete(&self, conn: &Connection) -> Result<()> {
        conn.execute("DELETE FROM items WHERE item_id = ?1", (self.id,))?;
        Ok(())
    }

    pub fn get_all(conn: &Connection) -> Result<Vec<Item>> {
        let mut stmt = conn.prepare("
            SELECT item_id, name, description, is_suspended, is_finished
            FROM items
        ")?;
        let result = stmt.query_map([], |row| {
            Item::from_row(row)
        })?.collect::<Result<Vec<_>>>()?;
        Ok(result)
    }
}

impl Default for Item {
    fn default() -> Self {
        Self {
            is_new: true,

            id: ItemId::new(0),
            name: String::new(),
            description: String::new(),
            is_suspended: false,
            is_finished: false,
            tags: vec![],
        }
    }
}

impl Hash for Item {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}


#[cfg(test)]
mod tests {
    use std::cmp::Ordering;
    use std::fmt::Debug;
    use rstest::*;
    use rusqlite::Connection;
    use crate::test_fixtures::{db_connection, TestFactory, test_factory};
    use super::*;

    #[rstest]
    #[case("Glorious Item", "")]
    #[case("Glorious Item", "Glorious Item Description")]
    fn create_item_successful_then_delete(
        db_connection: &Connection,
        mut test_factory: TestFactory,
        #[case] item_name: String,
        #[case] item_description: String,
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

    #[rstest]
    #[case("Glorious Item", "", false, false)]
    #[case("Glorious Item", "Glorious Item Description", true, true)]
    fn update_item(
        db_connection: &Connection,
        mut test_factory: TestFactory,
        #[case] expected_item_name: String,
        #[case] expected_item_description: String,
        #[case] expected_is_suspended: bool,
        #[case] expected_is_finished: bool,
    ) {
        let mut item = test_factory.create_items(1).pop().unwrap();
        item.name = expected_item_name.clone();
        item.description = expected_item_description.clone();
        item.is_suspended = expected_is_suspended;
        item.is_finished = expected_is_finished;
        item.save(&db_connection).unwrap();
        let item = Item::load(db_connection, item.id).unwrap();
        assert_eq!(
            item.name, expected_item_name,
            "Item name is wrong. Expected {:?}, got {:?}",
            expected_item_name, item.name
        );
        assert_eq!(
            item.description, expected_item_description,
            "Item description is wrong. Expected {:?}, got {:?}",
            expected_item_description, item.description
        );
        assert_eq!(item.is_suspended, expected_is_suspended, "Item suspended is wrong");
        assert_eq!(item.is_finished, expected_is_finished, "Item finished is wrong");
    }

    #[rstest]
    #[case(ItemId::new(2))]
    #[case(2i64)]
    fn load_item_by_id(
        db_connection: &Connection,
        mut test_factory: TestFactory,
        #[case] item_id: impl Into<ItemId> + Debug + Clone,
    ) {
        // create 3 items, get the second one by id and compare its properties
        test_factory.create_items(3);
        let item = Item::load(db_connection, item_id.clone()).unwrap();
        let expected_item_id = *item_id.into();
        assert_eq!(
            *item.id, expected_item_id,
            "Item id is wrong. Expected {:?}, got {:?}",
            expected_item_id, item.id
        );
        assert_eq!(item.is_new, false, "Item claims to be new");
    }

    #[rstest]
    #[case(0)]
    #[case(1)]
    #[case(5)]
    fn get_all(
        db_connection: &Connection,
        mut test_factory: TestFactory,
        #[case] expected_number_of_items: usize,
    ) {
        // Create 5 items and check that get_all() returns all items
        test_factory.create_items(expected_number_of_items);
        let items = Item::get_all(db_connection).unwrap();
        assert_eq!(items.len(), expected_number_of_items);
    }

    #[rstest]
    fn hash_test() {
        // Create 2 local items with the same id but different names and descriptions. Hash should be the same.
        let mut item1 = Item::default();
        item1.id = 99.into();
        item1.name = "The hash shall be the same".into();
        item1.description = "Even thou the name and desc are different".into();
        let mut item2 = Item::default();
        item2.id = 99.into();
        item2.name = "The hash shall be identical".into();
        item2.description = "Even thou the name and desc differ".into();
        // Hash should be the same
        let mut hasher1 = std::collections::hash_map::DefaultHasher::new();
        let mut hasher2 = std::collections::hash_map::DefaultHasher::new();
        item1.hash(&mut hasher1);
        item2.hash(&mut hasher2);
        assert_eq!(hasher1.finish(), hasher2.finish());

        // Make the id different and name & disc same. Hash should be different.
        item2.id = 100.into();
        item2.name = "The hash shall be different".into();
        item2.description = "Even thou the name and desc are the same".into();
        item1.id = 101.into();
        item1.name = "The hash shall be different".into();
        item1.description = "Even thou the name and desc are the same".into();
        // Hash should be different
        let mut hasher1 = std::collections::hash_map::DefaultHasher::new();
        let mut hasher2 = std::collections::hash_map::DefaultHasher::new();
        item1.hash(&mut hasher1);
        item2.hash(&mut hasher2);
        assert_ne!(hasher1.finish(), hasher2.finish());
    }
}
