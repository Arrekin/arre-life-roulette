use godot::builtin::{Callable, ToVariant};
use godot::engine::{Panel, LineEdit, TextEdit, Button, NodeExt, Engine};
use godot::obj::EngineClass;
use godot::prelude::*;
use crate::godot_classes::globals::{Globals};
use crate::godot_classes::utils::get_singleton;
use crate::item::Item;
use crate::list::List;

enum Mode {
    Add,
    Edit,
}

#[derive(GodotClass)]
#[class(base=Panel)]
pub struct ListModifyView {
    #[base]
    base: Base<Panel>,

    // cached elements
    name_line_edit: Option<Gd<LineEdit>>,
    description_text_edit: Option<Gd<TextEdit>>,
    apply_button: Option<Gd<Button>>,
    close_button: Option<Gd<Button>>,

    // state
    list: Option<List>,
    items_in: Vec<Item>,
    items_out: Vec<Item>,
    mode: Mode,
}

#[godot_api]
impl ListModifyView {
    #[signal]
    fn dialog_closed();

    #[func]
    fn on_apply_list_button_up(&mut self) {
        // let new_name = self.name_line_edit.as_ref().map(|line_edit| line_edit.get_text()).unwrap().to_string();
        // let new_description = self.description_text_edit.as_ref().map(|text_edit| text_edit.get_text()).unwrap().to_string();
        //
        // let globals = get_singleton::<Globals>("Globals");
        // let connection = &globals.bind().connection;
        //
        // match self.mode {
        //     Mode::Add => {
        //         Item::create_new(connection, new_name, new_description).unwrap();
        //     }
        //     Mode::Edit => {
        //         let item_id = self.item.as_mut().map(|item| {
        //             item.name = new_name;
        //             item.description = new_description;
        //             item.update(connection).unwrap();
        //             item.id
        //         }).expect("Edit mode while no item assigned!");
        //         self.item = Some(Item::load(connection, item_id).unwrap());
        //     }
        // }
        // self.refresh_display();
    }

    #[func]
    fn refresh_display(&mut self) {
        // match self.mode {
        //     Mode::Add => {
        //         self.name_line_edit.as_mut().map(|line_edit| line_edit.set_text("".into()));
        //         self.description_text_edit.as_mut().map(|text_edit| text_edit.set_text("".into()));
        //     }
        //     Mode::Edit => {
        //         self.name_line_edit.as_mut().map(|line_edit|
        //             line_edit.set_text(
        //                 self.item.as_ref().map(|item| item.name.clone()).expect("Edit mode while no item assigned!").into()
        //             )
        //         );
        //         self.description_text_edit.as_mut().map(|text_edit|
        //             text_edit.set_text(
        //                 self.item.as_ref().map(|item| item.description.clone()).expect("Edit mode while no item assigned!").into()
        //             )
        //         );
        //     }
        // }
    }

    #[func]
    fn on_dialog_close_button_up(&mut self) {
        self.hide();
        self.emit_signal("dialog_closed".into(), &[]);
    }

    pub fn set_mode_add(&mut self) {
        self.mode = Mode::Add;
        self.list = None;
        self.items_in = vec![];
        self.items_out = vec![];
        self.refresh_display();
    }

    pub fn set_mode_edit(&mut self, list: List) {
        self.mode = Mode::Edit;
        self.list = Some(list);
        self.refresh_display();
    }

}

#[godot_api]
impl GodotExt for ListModifyView {
    fn init(base: Base<Self::Base>) -> Self {
        Self {
            base,
            name_line_edit: None,
            description_text_edit: None,
            apply_button: None,
            close_button: None,

            list: None,
            items_in: vec![],
            items_out: vec![],
            mode: Mode::Add,
        }
    }
    fn ready(&mut self) {
        self.name_line_edit = self.base.try_get_node_as("ListNameLineEdit");
        self.description_text_edit = self.base.try_get_node_as("ListDescriptionTextEdit");
        self.apply_button = self.base.try_get_node_as("ListApplyButton");
        self.apply_button.as_mut().map(|mut button| {
            button.connect(
                "button_up".into(),
                Callable::from_object_method(self.base.share(), "on_apply_list_button_up"),
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