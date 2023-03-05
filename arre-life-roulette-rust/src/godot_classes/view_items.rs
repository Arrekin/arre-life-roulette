use godot::builtin::{Callable, ToVariant};
use godot::engine::{Control,  Button};
use godot::obj::EngineClass;
use godot::prelude::*;
use crate::godot_classes::globals::{Globals};
use crate::godot_classes::utils::get_singleton;
use crate::godot_classes::view_item_add::ItemAddView;
use crate::item::Item;

#[derive(GodotClass)]
#[class(base=Control)]
pub struct ItemsView {
    #[base]
    base: Base<Control>,

    // cached elements
    item_add_button: Option<Gd<Button>>,
    item_add_view: Option<Gd<ItemAddView>>,
}

#[godot_api]
impl ItemsView {
    #[func]
    fn on_item_add_button_up(&mut self) {
        self.item_add_view.as_mut().map(|mut view| view.bind_mut().set_visible(true));
    }
}

#[godot_api]
impl GodotExt for ItemsView {
    fn init(base: Base<Self::Base>) -> Self {
        Self {
            base,
            item_add_button: None,
            item_add_view: None,
        }
    }
    fn ready(&mut self) {
        self.item_add_button = self.base.try_get_node_as("ItemAddDialogButton");
        self.item_add_button.as_mut().map(|mut button| {
            button.connect(
                "button_up".into(),
                Callable::from_object_method(self.base.share(), "on_item_add_button_up"),
                0,
            )
        });
        self.item_add_view = self.base.try_get_node_as("../ItemAddView");
    }
}