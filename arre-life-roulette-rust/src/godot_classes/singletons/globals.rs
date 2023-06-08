use godot::engine::{Node, NodeVirtual};
use godot::prelude::*;
use rusqlite::Connection;
use crate::db_init::{initialize_database, initialized_demo_content_dev};

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
        initialized_demo_content_dev(&connection).unwrap();

        Self {
            base,
            connection,
        }
    }
}