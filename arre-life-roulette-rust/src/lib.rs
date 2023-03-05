mod item;
mod item_tag;
mod db_init;
mod test_fixtures;
mod list;
mod utils;
mod godot_classes;

use godot::prelude::*;

struct LifeRoulette;

#[gdextension]
unsafe impl ExtensionLibrary for LifeRoulette {}