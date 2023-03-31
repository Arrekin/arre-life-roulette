use godot::engine::Engine;
use godot::obj::dom;
use godot::prelude::*;


pub fn get_singleton<T>(name: impl Into<StringName>) -> Gd<T>
where
    T: GodotClass<Declarer = dom::UserDomain> + Inherits<Object>
{
    let name = name.into();
    Engine::singleton().get_singleton(name).unwrap().cast::<T>()
}