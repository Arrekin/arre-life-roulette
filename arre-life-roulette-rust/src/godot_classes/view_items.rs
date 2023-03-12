use godot::builtin::{Callable, ToVariant};
use godot::engine::{Control,  Button, GridContainer};
use godot::engine::node::InternalMode;
use godot::engine::packed_scene::GenEditState;
use godot::prelude::*;
use crate::godot_classes::globals::{Globals};
use crate::godot_classes::item_button::ItemSelectionButton;
use crate::godot_classes::utils::get_singleton;
use crate::godot_classes::view_item_modify::ItemModifyView;
use crate::item::Item;

#[derive(GodotClass)]
#[class(base=Control)]
pub struct ItemsView {
    #[base]
    base: Base<Control>,

    // cached sub-scenes
    item_selection_button: Gd<PackedScene>,

    // cached UI elements
    pub item_add_button: Option<Gd<Button>>,
    pub item_modify_view: Option<Gd<ItemModifyView>>,
    pub items_grid: Option<Gd<GridContainer>>,
    pub items_grid_elements: Vec<Gd<ItemSelectionButton>>,

    // state
    items: Vec<Item>,
}

#[godot_api]
impl ItemsView {
    #[func]
    fn on_item_add_button_up(&mut self) {
        self.item_modify_view.as_mut().map(|mut view| {
            let mut view = view.bind_mut();
            view.set_mode_add();
            view.show();
        });
    }
    #[func]
    fn on_item_selection_button_up(&mut self, item_selection_button: Gd<ItemSelectionButton>) {
        self.item_modify_view.as_mut().map(|mut view| {
            let mut view = view.bind_mut();
            view.set_mode_edit(item_selection_button.bind().item.clone());
            view.show();
        });
    }
    #[func]
    fn refresh_items_list(&mut self) {
        // Get current list of all items from the DB
        let globals = get_singleton::<Globals>("Globals");
        let connection = &globals.bind().connection;
        let mut stmt = connection.prepare("SELECT * FROM items").unwrap();
        let mut rows = stmt.query_map([], |row| {
            Ok(Item::from_row(row).unwrap())
        }).unwrap();
        self.items = rows.map(|row| row.unwrap()).collect();

        // Clear old and create a button for each item
        self.items_grid_elements.drain(..).for_each(|mut item| item.bind_mut().queue_free());
        self.items_grid_elements.extend(
        self.items.iter().map(
            |item| {
                let instance = self.item_selection_button.instantiate(GenEditState::GEN_EDIT_STATE_DISABLED).unwrap();
                self.items_grid.as_mut().map(|grid| grid.add_child(instance.share(), false, InternalMode::INTERNAL_MODE_DISABLED));
                let mut button = instance.cast::<ItemSelectionButton>();
                {
                    let mut button = button.bind_mut();
                    button.set_item(item.clone());
                }
                button
            })
        );
    }
}

#[godot_api]
impl GodotExt for ItemsView {
    fn init(base: Base<Self::Base>) -> Self {
        Self {
            base,
            item_selection_button: load("res://ItemSelectionButton.tscn"),

            item_add_button: None,
            item_modify_view: None,
            items_grid: None,
            items_grid_elements: vec![],

            items: vec![],
        }
    }
    fn ready(&mut self) {
        self.item_add_button = self.base.try_get_node_as("ItemAddDialogButton");
        self.item_add_button.as_mut().map(|mut button| {
            button.connect(
                "button_up".into(),
                Callable::from_object_method(self.base.share(), "on_item_add_button_up"),
                0,
            );
        });
        self.item_modify_view = self.base.try_get_node_as("../ItemModifyView");
        self.item_modify_view.as_mut().map(|mut view| {
            view.bind_mut().connect(
                "dialog_closed".into(),
                Callable::from_object_method(self.base.share(), "refresh_items_list"),
                0,
            );
        });
        self.items_grid = self.base.try_get_node_as("ItemsListScrollContainer/ItemsListGridContainer");

        if self.is_visible() {
            self.refresh_items_list();
        }

        // Get singleton and connect to global signals(show / hide)
        let mut globals = get_singleton::<Globals>("Globals");
        globals.bind_mut().connect(
            "item_view_tab_selected".into(),
            Callable::from_object_method(self.base.share(), "show"),
            0,
        );
        globals.bind_mut().connect(
            "list_view_tab_selected".into(),
            Callable::from_object_method(self.base.share(), "hide"),
            0,
        );
        globals.bind_mut().connect(
            "tag_view_tab_selected".into(),
            Callable::from_object_method(self.base.share(), "hide"),
            0,
        );
    }
}