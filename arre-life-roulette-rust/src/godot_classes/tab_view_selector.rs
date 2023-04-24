use godot::builtin::{Callable};
use godot::engine::{Button, DisplayServer, HBoxContainer};
use godot::engine::{HBoxContainerVirtual};
use godot::prelude::*;
use crate::errors::ArreError;
use crate::godot_classes::singletons::logger::log_error;
use crate::godot_classes::singletons::signals::Signals;
use crate::godot_classes::utils::get_singleton;

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
        let mut signals = get_singleton::<Signals>("Signals");
        signals.bind_mut().emit_signal("item_view_tab_selected".into(), &[]);
    }
    #[func]
    fn on_list_view_button_up(&mut self) {
        let mut signals = get_singleton::<Signals>("Signals");
        signals.bind_mut().emit_signal("list_view_tab_selected".into(), &[]);
    }
    #[func]
    fn on_tag_view_button_up(&mut self) {
        let mut signals = get_singleton::<Signals>("Signals");
        signals.bind_mut().emit_signal("tag_view_tab_selected".into(), &[]);
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
        // TODO: Find a better place for global config
        DisplayServer::singleton().window_set_min_size(Vector2i::new(1024, 768), 0);

        self.add_theme_constant_override("separation".into(), 20);

        self.items_view_button = self.base.try_get_node_as("ItemsViewButton");
        self.items_view_button.as_mut().map_or_else(
            || log_error(ArreError::NullGd("TabViewSelector::ready::items_view_button".into())),
            |button| {
                button.connect(
                    "button_up".into(),
                    Callable::from_object_method(self.base.share(), "on_item_view_button_up"),
                0,
                );
            }
        );
        self.lists_view_button = self.base.try_get_node_as("ListsViewButton");
        self.lists_view_button.as_mut().map_or_else(
            || log_error(ArreError::NullGd("TabViewSelector::ready::lists_view_button".into())),
            |button| {
                button.connect(
                    "button_up".into(),
                    Callable::from_object_method(self.base.share(), "on_list_view_button_up"),
                    0,
                );
            }
        );
        self.tags_view_button = self.base.try_get_node_as("TagsViewButton");
        self.tags_view_button.as_mut().map_or_else(
            || log_error(ArreError::NullGd("TabViewSelector::ready::tags_view_button".into())),
            |button| {
                button.connect(
                    "button_up".into(),
                    Callable::from_object_method(self.base.share(), "on_tag_view_button_up"),
                    0,
                );
            }
        );
    }
}