use godot::engine::{Node, NodeVirtual};
use godot::prelude::*;
use rusqlite::Connection;
use crate::db_init::initialize_database;
use crate::item::{item_create};
use crate::list::{list_create};

#[derive(GodotClass)]
#[class(base=Node)]
pub struct Globals {
    #[base]
    base: Base<Node>,

    // db connection
    pub connection: Connection

}

#[godot_api]
impl Globals {}

#[godot_api]
impl NodeVirtual for Globals {
    fn init(base: Base<Self::Base>) -> Self {
        let connection = Connection::open_in_memory().unwrap();
        initialize_database(&connection).unwrap();
        // insert some items examples
        item_create(&connection, "Demo Item 1".to_string(), "Demo Item 1 description".to_string()).unwrap();
        item_create(&connection, "Demo Item 2".to_string(), "Demo Item 2 description".to_string()).unwrap();
        item_create(&connection, "Demo Item 3".to_string(), "Demo Item 3 description".to_string()).unwrap();
        // Insert some lists examples
        list_create(&connection, "Demo List 1".to_string(), "Demo List 1 description".to_string()).unwrap();
        list_create(&connection, "Demo List 2".to_string(), "Demo List 2 description".to_string()).unwrap();
        list_create(&connection, "Demo List 3".to_string(), "Demo List 3 description".to_string()).unwrap();

        Self {
            base,
            connection,
        }
    }
}