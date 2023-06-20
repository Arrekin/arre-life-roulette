use std::sync::{Mutex, MutexGuard, OnceLock};
use chrono::Duration;
use rand::prelude::SliceRandom;
use rand::Rng;
use rusqlite::{Connection, Result};
use crate::errors::{ArreError, ArreResult};
use crate::item::{item_create};
use crate::item_stats::{item_stats_get, item_stats_update};
use crate::list::{list_create, list_items_add};

pub struct DbConnectionWrapper(pub OnceLock<Mutex<Connection>>);
impl DbConnectionWrapper {
    pub const fn new() -> Self {
        Self(OnceLock::new())
    }

    pub fn init(&self, conn: Connection) {
        self.0.set(Mutex::new(conn)).unwrap();
    }

    pub fn ok(&self) -> ArreResult<MutexGuard<Connection>> {
        self.0
            .get()
            .ok_or(ArreError::DatabaseConnectionNotEstablished())?
            .lock()
            .map_err(|_| ArreError::DatabaseConnectionMutexFailed().into())
    }

}

pub static DB: DbConnectionWrapper = DbConnectionWrapper::new();

pub fn set_db_connection() {
    let connection = Connection::open_in_memory().unwrap();
    initialize_database(&connection).unwrap();
    initialized_demo_content_dev(&connection).unwrap();
    DB.init(connection);
}

pub fn initialize_database(conn: &Connection) -> Result<()> {
    initialize_items_table(conn)?;
    initialize_items_stats_table(conn)?;
    initialize_items_details_table(conn)?;
    initialize_lists_table(conn)?;
    conn.execute(
        "CREATE TABLE item_list_map (
            list_id INTEGER,
            item_id INTEGER,
            PRIMARY KEY(list_id, item_id),
            FOREIGN KEY(list_id) REFERENCES lists(list_id) ON DELETE CASCADE,
            FOREIGN KEY(item_id) REFERENCES items(item_id) ON DELETE CASCADE
        )",
        (),
    )?;
    initialize_tags_table(conn)?;
    Ok(())
}

fn initialize_items_table(conn: &Connection) -> Result<()> {
    conn.execute_batch("
        CREATE TABLE items (
            item_id INTEGER PRIMARY KEY,
            created_date TEXT NOT NULL,
            updated_date TEXT NOT NULL,
            name TEXT NOT NULL,
            description TEXT NULL,
            is_suspended BOOLEAN NOT NULL DEFAULT 0 CHECK(is_suspended IN (0, 1)),
            is_finished BOOLEAN NOT NULL DEFAULT 0 CHECK(is_finished IN (0, 1))
        );
        CREATE VIRTUAL TABLE items_search_index USING fts5(name, description, tokenize=trigram);
        CREATE TRIGGER after_item_insert__insert_search AFTER INSERT ON items BEGIN
          INSERT INTO items_search_index (
            rowid,
            name,
            description
          )
          VALUES(
            new.item_id,
            new.name,
            new.description
          );
        END;
        CREATE TRIGGER after_items_update__update_search UPDATE OF name, description ON items BEGIN
          UPDATE items_search_index
          SET
            name = new.name,
            description = new.description
          WHERE rowid = old.item_id;
        END;
        CREATE TRIGGER after_items_delete AFTER DELETE ON items BEGIN
            DELETE FROM items_search_index WHERE rowid = old.item_id;
        END;
        "
    )
}

pub fn initialize_items_stats_table(conn: &Connection) -> Result<()> {
    conn.execute_batch("
        CREATE TABLE item_stats (
            item_id INTEGER PRIMARY KEY,
            updated_date TEXT NOT NULL,
            times_worked INTEGER NOT NULL DEFAULT 0,
            time_spent INTEGER NOT NULL DEFAULT 0,
            FOREIGN KEY(item_id) REFERENCES items(item_id) ON DELETE CASCADE
        );
        CREATE TRIGGER after_item_insert__insert_stats AFTER INSERT ON items BEGIN
          INSERT INTO item_stats (item_id, updated_date)
          VALUES(new.item_id, new.updated_date);
        END;
        "
    )
}

pub fn initialize_items_details_table(conn: &Connection) -> Result<()> {
    conn.execute_batch("
        CREATE TABLE item_details (
            item_id INTEGER PRIMARY KEY,
            updated_date TEXT NOT NULL,
            session_duration INTEGER NULL,
            FOREIGN KEY(item_id) REFERENCES items(item_id) ON DELETE CASCADE
        );
        CREATE TRIGGER after_item_insert__insert_details AFTER INSERT ON items BEGIN
          INSERT INTO item_details (item_id, updated_date)
          VALUES(new.item_id, new.updated_date);
        END;
        "
    )
}

pub fn initialize_lists_table(conn: &Connection) -> Result<()> {
    conn.execute_batch("
        CREATE TABLE lists (
            list_id INTEGER PRIMARY KEY,
            created_date TEXT NOT NULL,
            updated_date TEXT NOT NULL,
            name TEXT NOT NULL,
            description TEXT NULL
        );
        CREATE VIRTUAL TABLE lists_search_index USING fts5(name, description, tokenize=trigram);
        CREATE TRIGGER after_list_insert__insert_search AFTER INSERT ON lists BEGIN
          INSERT INTO lists_search_index (
            rowid,
            name,
            description
          )
          VALUES(
            new.list_id,
            new.name,
            new.description
          );
        END;
        CREATE TRIGGER after_lists_update__update_search AFTER UPDATE OF name, description ON lists BEGIN
          UPDATE lists_search_index
          SET
            name = new.name,
            description = new.description
          WHERE rowid = old.list_id;
        END;
        CREATE TRIGGER after_lists_delete AFTER DELETE ON lists BEGIN
            DELETE FROM lists_search_index WHERE rowid = old.list_id;
        END;
        "
    )
}

pub fn initialize_tags_table(conn: &Connection) -> Result<()> {
    conn.execute_batch("
        CREATE TABLE tags (
            tag_id INTEGER PRIMARY KEY,
            name TEXT NOT NULL,
            color TEXT NOT NULL
        );
        CREATE TABLE item_tag_map (
            tag_id INTEGER,
            item_id INTEGER,
            PRIMARY KEY(tag_id, item_id),
            FOREIGN KEY(tag_id) REFERENCES tags(tag_id) ON DELETE CASCADE,
            FOREIGN KEY(item_id) REFERENCES items(item_id) ON DELETE CASCADE
        );
        CREATE TABLE list_tag_map (
            tag_id INTEGER,
            list_id INTEGER,
            PRIMARY KEY(tag_id, list_id),
            FOREIGN KEY(tag_id) REFERENCES tags(tag_id) ON DELETE CASCADE,
            FOREIGN KEY(list_id) REFERENCES lists(list_id) ON DELETE CASCADE
        );
        "
    )
}

pub fn initialized_demo_content_dev(c: &Connection) -> ArreResult<()> {
    let items = [
        item_create(&c, "Empower Elves", "Remember, an elf's power is directly proportional to the shininess of their shoes.")?,
        item_create(&c, "Organize Orcs", "Like herding cats, but the cats are 6ft tall and have battleaxes.")?,
        item_create(&c, "Dispatch Dwarves", "Keep your head low, avoid the low ceiling!")?,
        item_create(&c, "Treat Trolls", "A spa day for trolls; includes optional bridge maintenance workshop.")?,
        item_create(&c, "Govern Goblins", "Handle with care, they're notorious for pulling pranks.")?,
        item_create(&c, "Garner Golems", "For when you need a problem solved with a bit more...gravitas.")?,
        item_create(&c, "Select Skeletons", "Handpicked from the finest, creepiest catacombs.")?,
        item_create(&c, "Assemble Archangels", "It's like a choir practice, but with more smiting.")?,
        item_create(&c, "Court Celestials", "Fancy a dance among the stars? Just don't step on any cosmic toes.")?,
        item_create(&c, "Lead Liches", "No need to be a bonehead, just keep them away from the necromancy section.")?,
        item_create(&c, "Muster Minotaurs", "Less 'charging bull' more 'herd of stubborn cows'.")?,
        item_create(&c, "Hail Hydras", "You know what they say, many heads are better than one!")?,
        item_create(&c, "Enlist Elementals", "The ultimate mix for a cocktail party - fire, water, earth, and air.")?,
        item_create(&c, "Marshal Manticores", "Don't forget the lint roller; those wings can shed.")?,
        item_create(&c, "Command Cyclops", "Eye contact is key. Really, there's no way to avoid it.")?,
        item_create(&c, "Entreat Ents", "Patience is a virtue. Especially when dealing with walking, talking trees.")?,
        item_create(&c, "Conscript Cockatrices", "Careful, their glare is worse than their peck!")?,
        item_create(&c, "Delegate Djinns", "Free wishes with every third meeting. (Some restrictions may apply.)")?,
        item_create(&c, "Designate Dragons", "Better stock up on fire extinguishers.")?,
        item_create(&c, "Beckon Basilisks", "Wear your mirrored sunglasses; safety first!")?,
        item_create(&c, "Instruct Ifrits", "The hottest teaching gig around.")?,
        item_create(&c, "Summon Sirens", "Earplugs not included.")?,
        item_create(&c, "Gather Gargoyles", "It's like a stone-cold reunion up in here.")?,
        item_create(&c, "Rally Rakshasas", "Tigers and tricksters - what could possibly go wrong?")?,
        item_create(&c, "Rise Revenants", "Just keep the revenge plots to a minimum, okay?")?,
        item_create(&c, "Conjure Chimeras", "A little bit of lion, a dash of goat, and voila!")?,
        item_create(&c, "Herald Harpies", "Cacophonous squawking is a small price to pay for flight... right?")?,
        item_create(&c, "Request Rocs", "Ensure to have giant birdseed on hand.")?,
        item_create(&c, "Amass Arachne", "It's the only time you'll actually want more spiders.")?,
        item_create(&c, "Mobilize Mermaids", "Just remember, they're much better in water than on land.")?,
        item_create(&c, "Nurture Nymphs", "Nature's caretakers, and the best gardeners around.")?,
    ];
    let mut rng = rand::thread_rng();
    for (idx, item) in items.iter().enumerate() {
        let mut stats = item_stats_get(&c, item.get_id()?)?;
        stats.time_spent = Duration::hours(idx as i64) + Duration::seconds(rng.gen_range(0..3600));
        stats.times_worked = idx;
        item_stats_update(&c, &stats)?
    }

    let lists = [
        list_create(&c, "Weekly Chores", "Tasks that require a divine touch. Just make sure the golems haven't been assigned to dusting...")?,
        list_create(&c, "Command Duties", "Tasks only fit for a leader. Remember, it's all about delegating - not every problem needs to be solved with a fireball.")?,
        list_create(&c, "Boring Meetings", "Even mythical creatures need to coordinate. Just try not to fall asleep when the Ents start discussing photosynthesis rates...")?,
        list_create(&c, "Upcoming Events", "Whether it's the annual Hydra Huddle or the monthly Manticore Meet, keep track of all important dates. (Gifts optional)")?,
        list_create(&c, "To Dos", "All those little errands that even a wizard can't magic away. If you have to pick up griffin feed and unicorn glitter, this is where you note it down.")?,
    ];
    let item_ids = items.iter().map(|i| Ok(i.get_id()?)).collect::<ArreResult<Vec<_>>>()?;
    for list in lists {
        let items_nb = rng.gen_range(0..items.len());
        let chosen_items = item_ids.choose_multiple(&mut rng, items_nb);
        list_items_add(&c, list.get_id()?, chosen_items)?;
    }
    Ok(())
}