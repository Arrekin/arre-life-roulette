use godot::builtin::{Callable, ToVariant};
use godot::engine::{Panel, LineEdit, TextEdit, Button, NodeExt, Engine};
use godot::obj::EngineClass;
use godot::prelude::*;

#[derive(GodotClass)]
#[class(base=Panel)]
pub struct ItemAddView {
    #[base]
    base: Base<Panel>,

    // cached elements
    name_line_edit: Option<Gd<LineEdit>>,
    description_text_edit: Option<Gd<TextEdit>>,
    add_button: Option<Gd<Button>>,
}

#[godot_api]
impl ItemAddView {
    #[func]
    fn on_add_item_button_up(&mut self) {
        let name = self.name_line_edit.as_ref().map(|line_edit| line_edit.get_text()).unwrap();
        let description = self.description_text_edit.as_ref().map(|text_edit| text_edit.get_text()).unwrap();

        //let globals = Engine::singleton().get_singleton();

        print(&["ItemAddButton clicked!".to_variant(), name.to_variant(), description.to_variant()]);
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
        }
    }
    fn ready(&mut self) {
        self.name_line_edit = self.base.try_get_node_as("ItemNameLineEdit");
        self.description_text_edit = self.base.try_get_node_as("ItemDescriptionTextEdit");
        self.add_button = self.base.try_get_node_as::<Button>("ItemAddButton");
        self.add_button.as_mut().map(|mut button| {
            button.connect(
                "button_up".into(),
                Callable::from_object_method(self.base.share(), "on_add_item_button_up"),
                0,
            )
        });
    }
}