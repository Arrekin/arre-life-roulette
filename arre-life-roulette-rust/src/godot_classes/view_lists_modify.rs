use godot::builtin::{Callable, ToVariant};
use godot::engine::{Panel, LineEdit, TextEdit, Button, NodeExt, Engine, GridContainer};
use godot::engine::node::InternalMode;
use godot::engine::packed_scene::GenEditState;
use godot::obj::EngineClass;
use godot::prelude::*;
use crate::godot_classes::globals::{Globals};
use crate::godot_classes::item_button::ItemSelectionButton;
use crate::godot_classes::utils::get_singleton;
use crate::item::Item;
use crate::list::List;

enum Mode {
    Add,
    Edit,
}

/// View allowing List modifications
/// items_in: Items in the list
/// items_out: Items not on the list
#[derive(GodotClass)]
#[class(base=Panel)]
pub struct ListModifyView {
    #[base]
    base: Base<Panel>,

    // cached sub-scenes
    item_selection_button: Gd<PackedScene>,

    // cached elements
    name_line_edit: Option<Gd<LineEdit>>,
    description_text_edit: Option<Gd<TextEdit>>,
    items_in_grid: Option<Gd<GridContainer>>,
    items_in_grid_elements: Vec<Gd<ItemSelectionButton>>,
    items_out_grid: Option<Gd<GridContainer>>,
    items_out_grid_elements: Vec<Gd<ItemSelectionButton>>,
    apply_button: Option<Gd<Button>>,
    close_button: Option<Gd<Button>>,

    // state
    list: List,
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
        self.refresh_name_and_description_display();
        self.refresh_items_in_display();
        self.refresh_items_out_display();
    }

    fn refresh_name_and_description_display(&mut self) {
        match self.mode {
            Mode::Add => {
                self.name_line_edit.as_mut().map(|line_edit| line_edit.set_text("".into()));
                self.description_text_edit.as_mut().map(|text_edit| text_edit.set_text("".into()));
            }
            Mode::Edit => {
                self.name_line_edit.as_mut().map(|line_edit|
                    line_edit.set_text(self.list.name.clone().into())
                );
                self.description_text_edit.as_mut().map(|text_edit|
                    text_edit.set_text(self.list.description.clone().into())
                );
            }
        }
    }

    fn refresh_items_in_display(&mut self) {
        // Clear old and create a button for each item
        self.items_in_grid_elements.drain(..).for_each(|mut item| item.bind_mut().queue_free());
        self.items_in_grid_elements.extend(
            self.list.items.iter().map(|item| {
                let instance = self.item_selection_button.instantiate(GenEditState::GEN_EDIT_STATE_DISABLED).unwrap();
                self.items_in_grid.as_mut().map(|grid| grid.add_child(instance.share(), false, InternalMode::INTERNAL_MODE_DISABLED));
                let mut button = instance.cast::<ItemSelectionButton>();
                {
                    let mut button = button.bind_mut();
                    button.set_item(item.clone());
                }
                button
            })
        );
    }

    fn refresh_items_out_display(&mut self) {
        // Clear old and create a button for each item
        self.items_out_grid_elements.drain(..).for_each(|mut item| item.bind_mut().queue_free());
        self.items_out_grid_elements.extend(
            self.items_out.iter().map(|item| {
                let instance = self.item_selection_button.instantiate(GenEditState::GEN_EDIT_STATE_DISABLED).unwrap();
                self.items_out_grid.as_mut().map(|grid| grid.add_child(instance.share(), false, InternalMode::INTERNAL_MODE_DISABLED));
                let mut button = instance.cast::<ItemSelectionButton>();
                {
                    let mut button = button.bind_mut();
                    button.set_item(item.clone());
                }
                button
            })
        );
    }

    #[func]
    fn on_dialog_close_button_up(&mut self) {
        self.hide();
        self.emit_signal("dialog_closed".into(), &[]);
    }

    pub fn set_mode_add(&mut self) {
        self.mode = Mode::Add;
        self.list = List::default();
        self.set_items_out();
        self.refresh_display();
    }

    pub fn set_mode_edit(&mut self, list: List) {
        self.mode = Mode::Edit;
        self.list = list;
        self.set_items_out();
        self.refresh_display();
    }

    fn set_items_out(&mut self) {
        let globals = get_singleton::<Globals>("Globals");
        let connection = &globals.bind().connection;
        self.items_out = self.list.get_items_not_on_list(connection).unwrap();
        godot_print!("{:?}", self.items_out);
    }

}

#[godot_api]
impl GodotExt for ListModifyView {
    fn init(base: Base<Self::Base>) -> Self {
        Self {
            base,

            item_selection_button: load("res://ItemSelectionButton.tscn"),

            name_line_edit: None,
            description_text_edit: None,
            items_in_grid: None,
            items_in_grid_elements: vec![],
            items_out_grid: None,
            items_out_grid_elements: vec![],
            apply_button: None,
            close_button: None,

            list: List::default(),
            items_out: vec![],
            mode: Mode::Add,
        }
    }
    fn ready(&mut self) {
        self.name_line_edit = self.base.try_get_node_as("ListNameLineEdit");
        self.description_text_edit = self.base.try_get_node_as("ListDescriptionTextEdit");
        self.items_in_grid = self.base.try_get_node_as("ListItemsInScrollContainer/ListItemsGridContainer");
        self.items_out_grid = self.base.try_get_node_as("ListItemsOutScrollContainer/ListItemsOutGridContainer");
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