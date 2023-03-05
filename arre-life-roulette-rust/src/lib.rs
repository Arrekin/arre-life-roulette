mod item;
mod item_tag;
mod db_init;
mod test_fixtures;
mod list;
mod utils;
mod godot_classes;

use godot::engine::class_macros::auto_register_classes;
use godot::engine::Engine;
use godot::prelude::*;
use crate::godot_classes::globals::Globals;

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
        Engine::singleton().register_singleton("Globals".into(), Gd::<Globals>::new_default().upcast());
    }

    fn deinitialize(&mut self) {}
}