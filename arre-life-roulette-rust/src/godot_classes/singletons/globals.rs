use chrono::Duration;
use godot::engine::{Node, NodeVirtual};
use godot::prelude::*;
use rand::prelude::IteratorRandom;
use rand::Rng;
use rusqlite::Connection;
use crate::db_init::initialize_database;
use crate::errors::ArreResult;
use crate::item::{item_create, ItemId};
use crate::item_stats::{item_stats_get, item_stats_update};
use crate::list::{list_create, list_items_add};

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
        default_setup(&connection).unwrap();

        Self {
            base,
            connection,
        }
    }
}

fn default_setup(connection: &Connection) -> ArreResult<()> {
    // insert items examples
    let mut rng = rand::thread_rng();
    for nb in 0..20 {
        //let item = item_create(&connection, "Demo Item 1".to_string(), "Demo Item 1 description".to_string()).unwrap();
        let item = item_create(&connection, format!("Demo Item {}", nb), format!("Demo Item {} description", nb))?;
        let mut stats = item_stats_get(&connection, item.get_id()?)?;
        stats.time_spent = Duration::seconds(rng.gen_range(0..60000));
        stats.times_worked = rng.gen_range(0..30);
        item_stats_update(&connection, &stats)?
    }
    // Insert some lists examples
    for nb in 0..5 {
        let list_id = list_create(&connection, format!("Demo List {}", nb), format!("Demo List {} description", nb))?.get_id()?;
        let items_nb = rng.gen_range(0..10);
        let chosen_items = (0..20).choose_multiple(&mut rng, items_nb);
        list_items_add(&connection, list_id, chosen_items.into_iter().map(ItemId::new))?;
    }
    Ok(())
}