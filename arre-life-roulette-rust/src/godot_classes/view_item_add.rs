use godot::builtin::{Callable, ToVariant};
use godot::engine::{Panel, LineEdit, TextEdit, Button, NodeExt, Engine};
use godot::obj::EngineClass;
use godot::prelude::*;
use crate::godot_classes::globals::{Globals};
use crate::godot_classes::utils::get_singleton;
use crate::item::Item;

#[derive(GodotClass)]
#[class(base=Panel)]
pub struct ItemAddView {
    #[base]
    base: Base<Panel>,

    // cached elements
    name_line_edit: Option<Gd<LineEdit>>,
    description_text_edit: Option<Gd<TextEdit>>,
    add_button: Option<Gd<Button>>,
    close_button: Option<Gd<Button>>,
}

#[godot_api]
impl ItemAddView {
    #[signal]
    fn dialog_closed();

    #[func]
    fn on_add_item_button_up(&mut self) {
        print(&["In function! ".to_variant()]);
        let name = self.name_line_edit.as_ref().map(|line_edit| line_edit.get_text()).unwrap();
        let description = self.description_text_edit.as_ref().map(|text_edit| text_edit.get_text()).unwrap();

        let globals = get_singleton::<Globals>("Globals");
        let connection = &globals.bind().connection;
        print(&["Got singleton by name! ".to_variant()]);
        Item::create_new(connection, name.to_string(), description.to_string()).unwrap();


        print(&["ItemAddButton clicked! ".to_variant(), name.to_variant(), description.to_variant()]);
    }

    #[func]
    fn on_dialog_close_button_up(&mut self) {
        self.set_visible(false);
        self.emit_signal("dialog_closed".into(), &[]);
    }
}

#[godot_api]
impl GodotExt for ItemAddView {
    fn init(base: Base<Self::Base>) -> Self {
        Self {
            base,
            name_line_edit: None,
            description_text_edit: None,
            add_button: None,
            close_button: None,
        }
    }
    fn ready(&mut self) {
        self.name_line_edit = self.base.try_get_node_as("ItemNameLineEdit");
        self.description_text_edit = self.base.try_get_node_as("ItemDescriptionTextEdit");
        self.add_button = self.base.try_get_node_as("ItemAddButton");
        self.add_button.as_mut().map(|mut button| {
            button.connect(
                "button_up".into(),
                Callable::from_object_method(self.base.share(), "on_add_item_button_up"),
                0,
            )
        });
        self.close_button = self.base.try_get_node_as("DialogCloseButton");
        self.close_button.as_mut().map(|mut button| {
            button.connect(
                "button_up".into(),
                Callable::from_object_method(self.base.share(), "on_dialog_close_button_up"),
                0,
            )
        });
    }
}