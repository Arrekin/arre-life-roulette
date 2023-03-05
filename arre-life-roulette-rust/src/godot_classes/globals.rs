use godot::builtin::{Callable, ToVariant};
use godot::engine::{Node};
use godot::prelude::*;
use rusqlite::Connection;
use crate::db_init::initialize_database;

#[derive(GodotClass)]
#[class(base=Node)]
pub struct Globals {
    #[base]
    base: Base<Node>,

    // db connection
    pub connection: Connection
}

#[godot_api]
impl GodotExt for Globals {
    fn init(base: Base<Self::Base>) -> Self {
        let connection = Connection::open_in_memory().unwrap();
        initialize_database(&connection).unwrap();
        Self {
            base,
            connection,
        }
    }
}