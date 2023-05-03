use rusqlite::{Connection};
use rstest::*;

use crate::db_init::initialize_database;
use crate::item::{Item, item_create, ItemId};
use crate::item_tag::{ItemTagId};
use crate::list::ListId;
use crate::errors::ArreResult;

#[fixture]
pub fn conn() -> Connection {
    let connection = Connection::open_in_memory();
    assert!(connection.is_ok(), "Failed to open connection to in-memory database, error: {}", connection.err().unwrap());
    let connection = connection.unwrap();
    let init_result = initialize_database(&connection);
    assert!(init_result.is_ok(), "Could not initialize database, error: {}", init_result.err().unwrap());
    connection
}

pub struct TestFactory<'a> {
    connection: &'a Connection,
    created_items: Vec<ItemId>,
    created_lists: Vec<ListId>,
    created_item_tags: Vec<ItemTagId>,
    items_count: usize,
    lists_count: usize,
    item_tags_count: usize,
}

impl TestFactory<'_> {
    pub fn new<'a>(connection: &'a Connection) -> TestFactory {
        TestFactory {
            connection,
            created_items: vec![],
            created_lists: vec![],
            created_item_tags: vec![],
            items_count: 0,
            lists_count: 0,
            item_tags_count: 0,
        }
    }

    pub fn create_items(&mut self, items_nb: usize) -> ArreResult<Vec<Item>> {
        (0..items_nb).map(|_| {
            let item = item_create(
                self.connection,
                format!("Item #{}", self.items_count),
                format!("Item #{} description", self.items_count),
            )?;
            self.created_items.push(item.get_id()?);
            self.items_count += 1;
            Ok(item)
        }).collect::<ArreResult<Vec<_>>>().into()
    }

    /// Assert total number of items in the DB
    pub fn assert_items_number(&self, items_nb: usize) -> ArreResult<()> {
        let mut stmt = self.connection.prepare("SELECT COUNT(*) FROM items")?;
        let items_count = stmt.query_row([], |row| row.get::<usize, usize>(0))?;
        assert_eq!(
            items_count, items_nb,
            "Items expected number is not equal to number of items in DB. Expected: {}, Actual: {}", items_nb, items_count
        );
        Ok(())
    }
    /// Assert total number of items in the list
    pub fn assert_items_number_in_list(&self, list: impl Into<ListId>, items_nb: usize) {
        let mut stmt = self.connection.prepare("SELECT COUNT(*) FROM item_list_map WHERE list_id = ?1").unwrap();
        let items_count = stmt.query_row([*list.into()], |row| row.get::<usize, usize>(0)).unwrap();
        assert_eq!(
            items_count, items_nb,
            "Items expected number is not equal to number of items in list. Expected: {}, Actual: {}", items_nb, items_count
        )
    }
    /// Assert whether item should or not exist in the DB
    pub fn assert_item_exist(&self, item: impl Into<ItemId>, should_exists: bool) -> ArreResult<()> {
        let expected = if should_exists { 1 } else { 0 };
        let mut stmt = self.connection.prepare("SELECT COUNT(*) FROM items WHERE item_id = ?1")?;
        let item_count = stmt.query_row([*item.into()], |row| row.get::<usize, usize>(0))?;
        assert_eq!(item_count, expected, "Item existence incorrect, Expected: {}, Actual: {}", expected, item_count);
        Ok(())
    }
    /// Assert whether item should or not be in a list
    pub fn assert_item_in_list(&self, item: impl Into<ItemId>, list: impl Into<ListId>, should_exists: bool) -> ArreResult<()> {
        let expected = if should_exists { 1 } else { 0 };
        let mut stmt = self.connection.prepare("SELECT COUNT(*) FROM item_list_map WHERE item_id = ?1 AND list_id = ?2")?;
        let item_count = stmt.query_row([*item.into(), *list.into()], |row| row.get::<usize, usize>(0))?;
        assert_eq!(item_count, expected, "Item existence in list incorrect, Expected: {}, Actual: {}", expected, item_count);
        Ok(())
    }

}