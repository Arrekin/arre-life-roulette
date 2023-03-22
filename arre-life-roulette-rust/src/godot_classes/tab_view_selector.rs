use godot::builtin::{Callable, ToVariant};
use godot::engine::{Control, Button, HBoxContainer};
use godot::engine::{HBoxContainerVirtual};
use godot::engine::node::InternalMode;
use godot::engine::packed_scene::GenEditState;
use godot::obj::EngineClass;
use godot::prelude::*;
use crate::godot_classes::globals::{Globals};
use crate::godot_classes::utils::get_singleton;
use crate::godot_classes::view_item_modify::ItemModifyView;
use crate::item::Item;
use crate::list::List;

#[derive(GodotClass)]
#[class(base=HBoxContainer)]
pub struct TabViewSelector {
    #[base]
    base: Base<HBoxContainer>,

    // cached UI elements
    items_view_button: Option<Gd<Button>>,
    lists_view_button: Option<Gd<Button>>,
    tags_view_button: Option<Gd<Button>>,

}

#[godot_api]
impl TabViewSelector {
    #[func]
    fn on_item_view_button_up(&mut self) {
        let mut globals = get_singleton::<Globals>("Globals");
        globals.bind_mut().emit_signal("item_view_tab_selected".into(), &[]);
    }
    #[func]
    fn on_list_view_button_up(&mut self) {
        let mut globals = get_singleton::<Globals>("Globals");
        globals.bind_mut().emit_signal("list_view_tab_selected".into(), &[]);
    }
    #[func]
    fn on_tag_view_button_up(&mut self) {
        let mut globals = get_singleton::<Globals>("Globals");
        globals.bind_mut().emit_signal("tag_view_tab_selected".into(), &[]);
    }
}

#[godot_api]
impl HBoxContainerVirtual for TabViewSelector {
    fn init(base: Base<Self::Base>) -> Self {
        Self {
            base,

            items_view_button: None,
            lists_view_button: None,
            tags_view_button: None,
        }
    }
    fn ready(&mut self) {
        self.items_view_button = self.base.try_get_node_as("ItemsViewButton");
        self.items_view_button.as_mut().map(|button|
            button.connect(
                "button_up".into(),
                Callable::from_object_method(self.base.share(), "on_item_view_button_up"),
            0,
            )
        );
        self.lists_view_button = self.base.try_get_node_as("ListsViewButton");
        self.lists_view_button.as_mut().map(|button|
            button.connect(
                "button_up".into(),
                Callable::from_object_method(self.base.share(), "on_list_view_button_up"),
                0,
            )
        );
        self.tags_view_button = self.base.try_get_node_as("TagsViewButton");
        self.tags_view_button.as_mut().map(|button|
            button.connect(
                "button_up".into(),
                Callable::from_object_method(self.base.share(), "on_tag_view_button_up"),
                0,
            )
        );
    }
}