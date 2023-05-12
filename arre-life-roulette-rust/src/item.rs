use std::hash::{Hash, Hasher};
use chrono::{Utc};
use rusqlite::{Connection, Result, Row};
use crate::errors::{ArreError, ArreResult};
use crate::utils::{ArreDateTime, Id};

pub fn item_create(conn: &Connection, name: impl AsRef<str>, description: impl AsRef<str>) -> ArreResult<Item> {
    let name = name.as_ref();
    let description = description.as_ref();
    let dt = Utc::now().to_string();
    conn.execute("
        INSERT INTO items (created_date, updated_date, name, description, is_suspended, is_finished) VALUES (?1, ?2, ?3, ?4, false, false);
        ", (dt.clone(), dt.clone(), name, description),
    )?;
    conn.execute("
        INSERT INTO item_stats (item_id, created_date, updated_date) VALUES (last_insert_rowid(), ?1, ?2);
        ", (dt.clone(), dt.clone()),
    )?;
    let mut stmt = conn.prepare("
        SELECT
         item_id, created_date, updated_date, name, description, is_suspended, is_finished
        FROM items
        WHERE item_id = last_insert_rowid()
    ")?;
    Ok(stmt.query_row([], |row| {
        Item::from_row(row)
    })?)
}

pub fn item_persist(conn: &Connection, item: &mut Item) -> ArreResult<()> {
    let dt = Utc::now().to_string();
    conn.execute("
        INSERT INTO items (created_date, updated_date, name, description, is_suspended, is_finished) VALUES (?1, ?2, ?3, ?4, ?5, ?6);
        ", (dt.clone(), dt.clone(), &item.name, &item.description, item.is_suspended, item.is_finished),
    )?;
    conn.execute("
        INSERT INTO item_stats (item_id, created_date, updated_date) VALUES (last_insert_rowid(), ?1, ?2);
        ", (dt.clone(), dt.clone()),
    )?;
    let mut stmt = conn.prepare("
        SELECT
         item_id, created_date, updated_date, name, description, is_suspended, is_finished
        FROM items
        WHERE item_id = last_insert_rowid()
    ")?;
    Ok(stmt.query_row([], |row| {
        item.update_from_row(row)
    })?)
}

pub fn item_update(conn: &Connection, item: &Item) -> ArreResult<()> {
    conn.execute("
        UPDATE items
        SET
         updated_date = ?1, name = ?2, description = ?3, is_suspended = ?4, is_finished = ?5
        WHERE item_id = ?6
    ", (Utc::now().to_string(), &item.name, &item.description, &item.is_suspended, &item.is_finished, item.get_id()?),
    )?;
    Ok(())
}

pub fn item_get(conn: &Connection, id: impl Into<ItemId>) -> ArreResult<Item> {
    let mut stmt = conn.prepare("
        SELECT
         item_id, created_date, updated_date, name, description, is_suspended, is_finished
        FROM items
        WHERE item_id = ?1
    ")?;
    Ok(stmt.query_row([id.into()], |row| {
        Item::from_row(row)
    })?)
}

pub fn item_get_all<C>(conn: &Connection) -> ArreResult<C>
where C: FromIterator<Item>
{
    let mut stmt = conn.prepare("
        SELECT
         item_id, created_date, updated_date, name, description, is_suspended, is_finished
        FROM items
    ")?;
    let result = stmt.query_map([], |row| {
        Item::from_row(row)
    })?.collect::<Result<C>>()?;
    Ok(result)
}

pub fn item_search<C>(conn: &Connection, search_term: impl AsRef<str>) -> ArreResult<C>
where C: FromIterator<Item>
{
    let search_term = search_term.as_ref();
    let mut stmt = conn.prepare("
        SELECT
         i.item_id, i.created_date, i.updated_date, i.name, i.description, i.is_suspended, i.is_finished
        FROM items i
        JOIN (
            SELECT
             rowid, rank
            FROM
             items_search_index
            WHERE
                items_search_index MATCH ?1
        ) search ON i.item_id = search.rowid
        ORDER BY search.rank DESC
    ")?;
    let result = stmt.query_map([search_term], |row| {
        Item::from_row(row)
    })?.collect::<Result<C>>()?;
    Ok(result)
}

pub fn item_delete(conn: &Connection, id: impl Into<ItemId>) -> ArreResult<()> {
    let id = id.into();
    conn.execute("DELETE FROM items WHERE item_id = ?1;", (id,))?;
    conn.execute("DELETE FROM item_stats WHERE item_id = ?1;", (id,))?;
    Ok(())
}

pub fn items_to_ids<C>(items: &[Item]) -> ArreResult<C>
where C: FromIterator<ItemId>
{
    items.iter().map(|item| item.get_id()).collect::<ArreResult<C>>()
}

pub type ItemId = Id<Item>;
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Item {
    pub id: Option<ItemId>,
    pub created_date: ArreDateTime<Utc>,
    pub updated_date: ArreDateTime<Utc>,
    pub name: String,
    pub description: String,
    pub is_suspended: bool,
    pub is_finished: bool,
}

impl Item {
    pub fn from_row(row: &Row) -> Result<Item> {
        Ok(Item {
            id: Some(row.get(0)?),
            created_date: row.get(1)?,
            updated_date: row.get(2)?,
            name: row.get(3)?,
            description: row.get(4)?,
            is_suspended: row.get(5)?,
            is_finished: row.get(6)?,
        })
    }

    pub fn update_from_row(&mut self, row: &Row) -> Result<()> {
        self.id = Some(row.get(0)?);
        self.created_date = row.get(1)?;
        self.updated_date = row.get(2)?;
        self.name = row.get(3)?;
        self.description = row.get(4)?;
        self.is_suspended = row.get(5)?;
        self.is_finished = row.get(6)?;
        Ok(())
    }

    pub fn get_id(&self) -> ArreResult<ItemId> {
        self.id.ok_or(ArreError::ItemNotPersisted().into())
    }
}

impl Default for Item {
    fn default() -> Self {
        Self {
            id: None,
            created_date: Utc::now().into(),
            updated_date: Utc::now().into(),
            name: String::new(),
            description: String::new(),
            is_suspended: false,
            is_finished: false,
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
    use std::fmt::Debug;
    use rstest::*;
    use rusqlite::Connection;
    use crate::test_fixtures::{conn, TestFactory};
    use super::*;

    #[rstest]
    #[case("Glorious Item", "")]
    #[case("Glorious Item", "Glorious Item Description")]
    fn item_create_successful_then_delete(
        conn: Connection,
        #[case] item_name: String,
        #[case] item_description: String,
    ) -> ArreResult<()> {
        let tf = TestFactory::new(&conn);
        let item = item_create(&conn, &item_name, &item_description)?;
        let item_id = item.get_id()?;
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
        item_delete(&conn, item_id)?;
        tf.assert_item_exist(item_id, false)?;
        tf.assert_items_number(0)?;
        Ok(())
    }

    #[rstest]
    fn item_persist_successful(
        conn: Connection,
    ) -> ArreResult<()> {
        let tf = TestFactory::new(&conn);
        let mut item = Item::default();
        match item.get_id() {
            Ok(_) => { panic!("ItemId should not be available in non persisted item")}
            Err(err) => {
                if let Some(&ArreError::ItemNotPersisted()) = err.downcast_ref::<ArreError>() {
                    // The expected outcome. Continue.
                } else {  panic!("Unexpected error: {:?}", err) }
            }
        }
        item_persist(&conn, &mut item)?;
        tf.assert_item_exist(item.get_id()?, true)?;
        Ok(())
    }


    #[rstest]
    #[case("Glorious Item", "", false, false)]
    #[case("Glorious Item", "Glorious Item Description", true, true)]
    fn item_update_successful(
        conn: Connection,
        #[case] expected_item_name: String,
        #[case] expected_item_description: String,
        #[case] expected_is_suspended: bool,
        #[case] expected_is_finished: bool,
    ) -> ArreResult<()> {
        let mut tf = TestFactory::new(&conn);
        let mut item = tf.create_items(1)?.pop().unwrap();
        item.name = expected_item_name.clone();
        item.description = expected_item_description.clone();
        item.is_suspended = expected_is_suspended;
        item.is_finished = expected_is_finished;
        item_update(&conn, &item)?;
        let item = item_get(&conn, item.get_id()?)?;
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
        Ok(())
    }

    #[rstest]
    #[case(ItemId::new(2))]
    #[case(2i64)]
    fn item_get_successful(
        conn: Connection,
        #[case] item_id: impl Into<ItemId> + Debug + Clone,
    ) -> ArreResult<()>{
        let mut tf = TestFactory::new(&conn);
        // create 3 items, get the second one by id and compare its properties
        tf.create_items(3)?;
        let item = item_get(&conn, item_id.clone())?;
        let expected_item_id = Some(item_id.into());
        assert_eq!(
            item.id, expected_item_id,
            "Item id is wrong. Expected {:?}, got {:?}",
            expected_item_id, item.id
        );
        Ok(())
    }

    #[rstest]
    #[case(0)]
    #[case(1)]
    #[case(5)]
    fn item_get_all_successful(
        conn: Connection,
        #[case] expected_number_of_items: usize,
    ) -> ArreResult<()> {
        let mut tf = TestFactory::new(&conn);
        // Create 5 items and check that get_all() returns all items
        tf.create_items(expected_number_of_items)?;
        let items = item_get_all::<Vec<_>>(&conn)?;
        assert_eq!(items.len(), expected_number_of_items);
        Ok(())
    }

    #[rstest]
    #[case("Zero", 0)]
    #[case("onE", 1)]
    #[case("TwO", 2)]
    #[case("three", 3)]
    fn item_search_successful(
        conn: Connection,
        #[case] search_term: String,
        #[case] expected_number_of_items: usize
    ) -> ArreResult<()> {
        item_create(&conn, "One", "Not number three at all")?;
        item_create(&conn, "Not ThRee", "but it's number TWO!")?;
        item_create(&conn, "Finally Three :o", "Not two, it means :(")?;
        let items = item_search::<Vec<_>>(&conn, search_term)?;
        assert_eq!(items.len(), expected_number_of_items);
        Ok(())
    }

    #[rstest]
    fn item_search_after_update(conn: Connection) -> ArreResult<()> {
        let mut item = item_create(&conn, "Glorious Item", "Beyond Comprehension")?;
        assert_eq!(&item_search::<Vec<_>>(&conn, "Glorious")?[0].name as &str, "Glorious Item");
        item.name = "Heavenly Item".into();
        item_update(&conn, &item)?;
        assert_eq!(item_search::<Vec<_>>(&conn, "Glorious")?.len(), 0);
        assert_eq!(&item_search::<Vec<_>>(&conn, "Heavenly")?[0].name as &str, "Heavenly Item");
        Ok(())
    }

    #[rstest]
    fn item_search_index_purge_after_delete(conn: Connection) -> ArreResult<()> {
        let count_fn = |conn: &Connection| -> Result<i64> {
            conn.query_row_and_then(
                "SELECT count(*) FROM items_search_index;",
                (),
                |row| {
                    row.get(0)
                }
            )
        };
        let mut tf = TestFactory::new(&conn);
        assert_eq!(count_fn(&conn)?, 0);
        let items = tf.create_items(10)?;
        assert_eq!(count_fn(&conn)?, 10);
        items.into_iter().for_each(|item| item_delete(&conn, item.get_id().unwrap()).unwrap());
        assert_eq!(count_fn(&conn)?, 0);
        Ok(())
    }


    #[rstest]
    fn hash_test() {
        // Create 2 local items with the same id but different names and descriptions. Hash should be the same.
        let mut item1 = Item::default();
        item1.id = Some(99.into());
        item1.name = "The hash shall be the same".into();
        item1.description = "Even thou the name and desc are different".into();
        let mut item2 = Item::default();
        item2.id = Some(99.into());
        item2.name = "The hash shall be identical".into();
        item2.description = "Even thou the name and desc differ".into();
        // Hash should be the same
        let mut hasher1 = std::collections::hash_map::DefaultHasher::new();
        let mut hasher2 = std::collections::hash_map::DefaultHasher::new();
        item1.hash(&mut hasher1);
        item2.hash(&mut hasher2);
        assert_eq!(hasher1.finish(), hasher2.finish());

        // Make the id different and name & disc same. Hash should be different.
        item2.id = Some(100.into());
        item2.name = "The hash shall be different".into();
        item2.description = "Even thou the name and desc are the same".into();
        item1.id = Some(101.into());
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
