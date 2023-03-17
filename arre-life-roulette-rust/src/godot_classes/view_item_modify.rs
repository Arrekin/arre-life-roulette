use godot::builtin::{Callable, ToVariant};
use godot::engine::{Panel, LineEdit, TextEdit, Button, NodeExt, Engine};
use godot::obj::EngineClass;
use godot::prelude::*;
use crate::godot_classes::globals::{Globals};
use crate::godot_classes::utils::get_singleton;
use crate::item::Item;

enum Mode {
    Add,
    Edit,
}

#[derive(GodotClass)]
#[class(base=Panel)]
pub struct ItemModifyView {
    #[base]
    base: Base<Panel>,

    // cached elements
    name_line_edit: Option<Gd<LineEdit>>,
    description_text_edit: Option<Gd<TextEdit>>,
    apply_button: Option<Gd<Button>>,
    close_button: Option<Gd<Button>>,

    // state
    item: Item,
    mode: Mode,
}

#[godot_api]
impl ItemModifyView {
    #[signal]
    fn dialog_closed();

    #[func]
    fn on_apply_item_button_up(&mut self) {
        let new_name = self.name_line_edit.as_ref().map(|line_edit| line_edit.get_text()).unwrap().to_string();
        let new_description = self.description_text_edit.as_ref().map(|text_edit| text_edit.get_text()).unwrap().to_string();

        let globals = get_singleton::<Globals>("Globals");
        let connection = &globals.bind().connection;

        self.item.name = new_name;
        self.item.description = new_description;
        self.item.save(connection).unwrap();

        self.refresh_display();
    }

    #[func]
    fn refresh_display(&mut self) {
        self.name_line_edit.as_mut().map(|line_edit|
            line_edit.set_text(self.item.name.clone().into())
        );
        self.description_text_edit.as_mut().map(|text_edit|
            text_edit.set_text(self.item.description.clone().into())
        );
    }

    #[func]
    fn on_dialog_close_button_up(&mut self) {
        self.hide();
        self.emit_signal("dialog_closed".into(), &[]);
    }

    pub fn set_mode_add(&mut self) {
        self.mode = Mode::Add;
        self.item = Item::default();
        self.refresh_display();
    }

    pub fn set_mode_edit(&mut self, item: Item) {
        self.mode = Mode::Edit;
        self.item = item;
        self.refresh_display();
    }

}

#[godot_api]
impl GodotExt for ItemModifyView {
    fn init(base: Base<Self::Base>) -> Self {
        Self {
            base,
            name_line_edit: None,
            description_text_edit: None,
            apply_button: None,
            close_button: None,

            item: Item::default(),
            mode: Mode::Add,
        }
    }
    fn ready(&mut self) {
        self.name_line_edit = self.base.try_get_node_as("ItemNameLineEdit");
        self.description_text_edit = self.base.try_get_node_as("ItemDescriptionTextEdit");
        self.apply_button = self.base.try_get_node_as("ItemApplyButton");
        self.apply_button.as_mut().map(|mut button| {
            button.connect(
                "button_up".into(),
                Callable::from_object_method(self.base.share(), "on_apply_item_button_up"),
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