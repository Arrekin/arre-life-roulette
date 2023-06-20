#![feature(try_blocks)]
mod item;
mod tag;
mod db;
#[cfg(test)]
mod test_fixtures;
mod list;
mod utils;
mod godot_classes;
mod errors;
mod item_stats;
mod item_details;

use godot::engine::class_macros::auto_register_classes;
use godot::engine::Engine;
use godot::prelude::*;
use crate::db::set_db_connection;
use crate::godot_classes::singletons::buses::Buses;
use crate::godot_classes::singletons::logger::Logger;
use crate::godot_classes::singletons::signals::Signals;

struct LifeRoulette;

#[gdextension]
unsafe impl ExtensionLibrary for LifeRoulette {
    fn load_library(handle: &mut InitHandle) -> bool {
        handle.register_layer(InitLevel::Scene, DefaultLayer);
        true
    }
}

struct DefaultLayer;

impl ExtensionLayer for DefaultLayer {
    fn initialize(&mut self) {
        auto_register_classes();
        Engine::singleton().register_singleton("Buses".into(), Gd::<Buses>::new_default().upcast());
        Engine::singleton().register_singleton("Signals".into(), Gd::<Signals>::new_default().upcast());
        Engine::singleton().register_singleton("Logger".into(), Gd::<Logger>::new_default().upcast());

        set_db_connection();
    }

    fn deinitialize(&mut self) {}
}