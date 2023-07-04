use godot::engine::{Button, DisplayServer, HBoxContainer};
use godot::engine::{HBoxContainerVirtual};
use godot::prelude::*;
use crate::errors::{BoxedError};
use crate::godot_classes::singletons::logger::log_error;
use crate::godot_classes::singletons::signals::Signals;
use crate::godot_classes::utils::{GdHolder, get_singleton};

#[derive(GodotClass)]
#[class(base=HBoxContainer)]
pub struct TabViewSelector {
    #[base]
    base: Base<HBoxContainer>,

    // cached UI elements
    items_view_button: GdHolder<Button>,
    lists_view_button: GdHolder<Button>,
    tags_view_button: GdHolder<Button>,

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

            items_view_button: GdHolder::default(),
            lists_view_button: GdHolder::default(),
            tags_view_button: GdHolder::default(),
        }
    }
    fn ready(&mut self) {
        match try {
            // TODO: Find a better place for global config
            DisplayServer::singleton().window_set_min_size(Vector2i::new(1024, 768));

            self.add_theme_constant_override("separation".into(), 20);

            let base = &self.base;
            self.items_view_button = GdHolder::from_path(base, "ItemsViewButton");
            self.items_view_button.ok_mut()?.connect(
                "button_up".into(),
                base.callable("on_item_view_button_up"),
            );
            self.lists_view_button = GdHolder::from_path(base, "ListsViewButton");
            self.lists_view_button.ok_mut()?.connect(
                "button_up".into(),
                base.callable("on_list_view_button_up"),
            );
            self.tags_view_button = GdHolder::from_path(base, "TagsViewButton");
            self.tags_view_button.ok_mut()?.connect(
                "button_up".into(),
                base.callable("on_tag_view_button_up"),
            );
        } {
            Ok(_) => {}
            Err::<_, BoxedError>(e) => log_error(e)
        }
    }
}