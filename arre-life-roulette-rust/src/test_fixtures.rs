use std::borrow::Borrow;
use rusqlite::{Connection, Result};
#[cfg(test)]
use rstest::*;

use crate::db_init::initialize_database;
use crate::item::{Item, ItemId};
use crate::item_tag::{ItemTag, ItemTagId};
use crate::list::{List, ListId};

#[cfg(test)]
#[fixture]
#[once]
pub fn db_connection() -> Connection {
    let connection = Connection::open_in_memory();
    assert!(connection.is_ok(), "Failed to open connection to in-memory database, error: {}", connection.err().unwrap());
    let connection = connection.unwrap();
    let init_result = initialize_database(&connection);
    assert!(init_result.is_ok(), "Could not initialize database, error: {}", init_result.err().unwrap());
    connection
}

#[cfg(test)]
#[fixture]
pub fn test_factory<'a>(db_connection: &'a Connection) -> TestFactory<'a> {
    TestFactory::new(&db_connection)
}

#[cfg(test)]
pub struct TestFactory<'a> {
    connection: &'a Connection,
    created_items: Vec<ItemId>,
    created_lists: Vec<ListId>,
    created_item_tags: Vec<ItemTagId>,
    items_count: usize,
    lists_count: usize,
    item_tags_count: usize,
}

#[cfg(test)]
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

    pub fn create_items(&mut self, items_nb: usize) -> Vec<Item> {
        (0..items_nb).map(|_| {
            let item = Item::create_new(
                self.connection,
                format!("Item #{}", self.items_count),
                &Some(format!("Item #{} description", self.items_count)),
            ).unwrap();
            self.created_items.push(item.id);
            self.items_count += 1;
            item
        }).collect::<Vec<_>>()
    }

    /// Assert total number of items in the DB
    pub fn assert_items_number(&self, items_nb: usize) {
        let mut stmt = self.connection.prepare("SELECT COUNT(*) FROM items").unwrap();
        let items_count = stmt.query_row([], |row| row.get::<usize, usize>(0)).unwrap();
        assert_eq!(
            items_count, items_nb,
            "Items expected number is not equal to number of items in DB. Expected: {}, Actual: {}", items_nb, items_count
        );
    }
    /// Assert total number of items in the list
    pub fn assert_items_number_in_list(&self, list: impl Borrow<List>, items_nb: usize) {
        let mut stmt = self.connection.prepare("SELECT COUNT(*) FROM item_list_map WHERE list_id = ?1").unwrap();
        let items_count = stmt.query_row([list.borrow().id], |row| row.get::<usize, usize>(0)).unwrap();
        assert_eq!(
            items_count, items_nb,
            "Items expected number is not equal to number of items in list. Expected: {}, Actual: {}", items_nb, items_count
        )
    }
    /// Assert whether item should or not exist in the DB
    pub fn assert_item_exist(&self, item: impl Borrow<Item>, should_exists: bool) {
        let expected = if should_exists { 1 } else { 0 };
        let mut stmt = self.connection.prepare("SELECT COUNT(*) FROM items WHERE item_id = ?1").unwrap();
        let item_count = stmt.query_row([item.borrow().id], |row| row.get::<usize, usize>(0)).unwrap();
        assert_eq!(item_count, expected, "Item existence incorrect, Expected: {}, Actual: {}", expected, item_count);
    }
    /// Assert whether item should or not be in a list
    pub fn assert_item_in_list(&self, item: impl Borrow<Item>, list: impl Borrow<List>, should_exists: bool) {
        let expected = if should_exists { 1 } else { 0 };
        let mut stmt = self.connection.prepare("SELECT COUNT(*) FROM item_list_map WHERE item_id = ?1 AND list_id = ?2").unwrap();
        let item_count = stmt.query_row([*item.borrow().id, *list.borrow().id], |row| row.get::<usize, usize>(0)).unwrap();
        assert_eq!(item_count, expected, "Item existence in list incorrect, Expected: {}, Actual: {}", expected, item_count);
    }

}

#[cfg(test)]
impl Drop for TestFactory<'_> {
    fn drop(&mut self) {
        self.connection.execute("\
            DELETE FROM items;\
            DELETE FROM lists;\
            DELETE FROM item_tags;\
        ", ()).unwrap();
    }
}