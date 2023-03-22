use godot::builtin::{Callable, ToVariant};
use godot::engine::{Control, ControlVirtual, Button, GridContainer};
use godot::engine::node::InternalMode;
use godot::engine::packed_scene::GenEditState;
use godot::obj::EngineClass;
use godot::prelude::*;
use crate::godot_classes::globals::{Globals};
use crate::godot_classes::utils::get_singleton;
use crate::godot_classes::view_item_modify::ItemModifyView;
use crate::godot_classes::view_lists_modify::ListModifyView;
use crate::item::Item;
use crate::list::List;

#[derive(GodotClass)]
#[class(base=Control)]
pub struct ListsView {
    #[base]
    base: Base<Control>,

    // cached sub-scenes
    list_selection_button: Gd<PackedScene>,

    // cached UI elements
    list_add_button: Option<Gd<Button>>,
    list_modify_view: Option<Gd<ListModifyView>>,
    lists_grid: Option<Gd<GridContainer>>,
    lists_grid_elements: Vec<Gd<Button>>,

    // state
    lists: Vec<List>,
}

#[godot_api]
impl ListsView {
    #[func]
    fn on_list_add_button_up(&mut self) {
        self.list_modify_view.as_mut().map(|mut view| {
            let mut view = view.bind_mut();
            view.set_mode_add();
            view.set_visible(true);
        });
    }
    #[func]
    fn refresh_lists_list(&mut self) {
        // Get current list of all lists from the DB
        let globals = get_singleton::<Globals>("Globals");
        let connection = &globals.bind().connection;
        let mut stmt = connection.prepare("SELECT * FROM lists").unwrap();
        let mut rows = stmt.query_map([], |row| {
            Ok(List::from_row(row).unwrap())
        }).unwrap();
        self.lists = rows.map(|row| row.unwrap()).collect();

        // Clear old and create a button for each item
        self.lists_grid_elements.drain(..).for_each(|mut list| list.queue_free());
        self.lists_grid_elements.extend(
            self.lists.iter().map(|list| {
                    let instance = self.list_selection_button.instantiate(GenEditState::GEN_EDIT_STATE_DISABLED).unwrap();
                    self.lists_grid.as_mut().map(|grid| grid.add_child(instance.share(), false, InternalMode::INTERNAL_MODE_DISABLED));
                    let mut button = instance.cast::<Button>();
                    button.set_text(list.name.clone().into());
                    button.set_tooltip_text(list.description.clone().into());
                    button
                })
            );

    }
}

#[godot_api]
impl ControlVirtual for ListsView {
    fn init(base: Base<Self::Base>) -> Self {
        Self {
            base,
            list_selection_button: load("res://ItemSelectionButton.tscn"),

            list_add_button: None,
            list_modify_view: None,
            lists_grid: None,
            lists_grid_elements: vec![],

            lists: vec![],
        }
    }
    fn ready(&mut self) {
        self.list_add_button = self.base.try_get_node_as("ListAddDialogButton");
        self.list_add_button.as_mut().map(|mut button| {
            button.connect(
                "button_up".into(),
                Callable::from_object_method(self.base.share(), "on_list_add_button_up"),
                0,
            );
        });
        self.list_modify_view = self.base.try_get_node_as("../ListModifyView");
        self.list_modify_view.as_mut().map(|mut view| {
            view.bind_mut().connect(
                "dialog_closed".into(),
                Callable::from_object_method(self.base.share(), "refresh_lists_list"),
                0,
            );
        });
        self.lists_grid = self.base.try_get_node_as("ListsListScrollContainer/ListsListGridContainer");

        if self.is_visible() {
            self.refresh_lists_list();
        }

        // Get singleton and connect to global signals(show / hide)
        let mut globals = get_singleton::<Globals>("Globals");
        globals.bind_mut().connect(
            "item_view_tab_selected".into(),
            Callable::from_object_method(self.base.share(), "hide"),
            0,
        );
        globals.bind_mut().connect(
            "list_view_tab_selected".into(),
            Callable::from_object_method(self.base.share(), "show"),
            0,
        );
        globals.bind_mut().connect(
            "tag_view_tab_selected".into(),
            Callable::from_object_method(self.base.share(), "hide"),
            0,
        );
    }
}