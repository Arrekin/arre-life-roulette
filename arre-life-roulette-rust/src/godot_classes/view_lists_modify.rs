use godot::builtin::{Callable};
use godot::engine::{Panel, PanelVirtual, LineEdit, TextEdit, Button, NodeExt, GridContainer};
use godot::engine::node::InternalMode;
use godot::engine::packed_scene::GenEditState;
use godot::prelude::*;
use crate::godot_classes::globals::{Globals};
use crate::godot_classes::selection_button::{SelectionButton, OnClickBehavior, Content};
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
    items_in_grid_elements: Vec<Gd<SelectionButton>>,
    items_out_grid: Option<Gd<GridContainer>>,
    items_out_grid_elements: Vec<Gd<SelectionButton>>,
    apply_button: Option<Gd<Button>>,
    close_button: Option<Gd<Button>>,

    // state
    list: List,
    items_out: Vec<Item>,
    mode: Mode,

    // internals
    needs_full_refresh: bool,
    needs_items_refresh: bool,
}

#[godot_api]
impl ListModifyView {
    #[signal]
    fn dialog_closed();

    #[func]
    fn on_apply_list_button_up(&mut self) {
        let new_name = self.name_line_edit.as_ref().map(|line_edit| line_edit.get_text()).unwrap().to_string();
        let new_description = self.description_text_edit.as_ref().map(|text_edit| text_edit.get_text()).unwrap().to_string();

        let globals = get_singleton::<Globals>("Globals");
        let connection = &globals.bind().connection;

        match self.mode {
            Mode::Add => {
                let mut new_list = List::create_new(connection, new_name, new_description).unwrap();
                new_list.items = std::mem::replace(&mut self.list.items, vec![]);
                new_list.save(connection).unwrap();
                self.set_mode_edit(new_list);
            }
            Mode::Edit => {
                self.list.name = new_name;
                self.list.description = new_description;
                self.list.save(connection).unwrap();
            }
        }
        self.needs_full_refresh = true;
    }

    fn refresh_name_and_description_display(&mut self) {
        self.name_line_edit.as_mut().map(|line_edit|
            line_edit.set_text(self.list.name.clone().into())
        );
        self.description_text_edit.as_mut().map(|text_edit|
            text_edit.set_text(self.list.description.clone().into())
        );
    }

    fn refresh_items_in_display(&mut self) {
        // Clear old and create a button for each item
        self.items_in_grid_elements.drain(..).for_each(|mut item| item.bind_mut().queue_free());
        self.items_in_grid_elements.extend(
            self.list.items.iter().map(|item| {
                let instance = self.item_selection_button.instantiate(GenEditState::GEN_EDIT_STATE_DISABLED).unwrap();
                self.items_in_grid.as_mut().map(|grid| grid.add_child(instance.share(), false, InternalMode::INTERNAL_MODE_DISABLED));
                let mut button = instance.cast::<SelectionButton>();
                {
                    let mut button = button.bind_mut();
                    button.set_item(item.clone());
                    button.on_click_behavior = Some(Box::new(OnClickBehaviorSwitchItemsInOut{
                        parent: self.base.share().cast::<Self>(),
                        in_or_out: InOrOut::In
                    }));
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
                let mut button = instance.cast::<SelectionButton>();
                {
                    let mut button = button.bind_mut();
                    button.set_item(item.clone());
                    button.on_click_behavior = Some(Box::new(OnClickBehaviorSwitchItemsInOut{
                        parent: self.base.share().cast::<Self>(),
                        in_or_out: InOrOut::Out
                    }));
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
        self.refresh_items_in_out();
        self.needs_full_refresh = true;
    }

    pub fn set_mode_edit(&mut self, list: List) {
        self.mode = Mode::Edit;
        self.list = list;
        self.refresh_items_in_out();
        self.needs_full_refresh = true;
    }

    fn refresh_items_in_out(&mut self) {
        let globals = get_singleton::<Globals>("Globals");
        let connection = &globals.bind().connection;
        self.items_out = self.list.get_items_not_on_list(connection).unwrap();
        self.list.load_items(connection).unwrap();
    }

}

#[godot_api]
impl PanelVirtual for ListModifyView {
    fn init(base: Base<Self::Base>) -> Self {
        Self {
            base,

            item_selection_button: load("res://SelectionButton.tscn"),

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

            needs_full_refresh: false,
            needs_items_refresh: false,
        }
    }
    fn ready(&mut self) {
        self.name_line_edit = self.base.try_get_node_as("ListNameLineEdit");
        self.description_text_edit = self.base.try_get_node_as("ListDescriptionTextEdit");
        self.items_in_grid = self.base.try_get_node_as("ListItemsInScrollContainer/ListItemsInGridContainer");
        self.items_out_grid = self.base.try_get_node_as("ListItemsOutScrollContainer/ListItemsOutGridContainer");
        self.apply_button = self.base.try_get_node_as("ListApplyButton");
        self.apply_button.as_mut().map(|button| {
            button.connect(
                "button_up".into(),
                Callable::from_object_method(self.base.share(), "on_apply_list_button_up"),
                0,
            )
        });
        self.close_button = self.base.try_get_node_as("DialogCloseButton");
        self.close_button.as_mut().map(|button| {
            button.connect(
                "button_up".into(),
                Callable::from_object_method(self.base.share(), "on_dialog_close_button_up"),
                0,
            )
        });
    }

    fn process(&mut self, _delta: f64) {
        if self.needs_full_refresh {
            self.refresh_name_and_description_display();
            self.refresh_items_in_display();
            self.refresh_items_out_display();
            self.needs_full_refresh = false;
        } else if self.needs_items_refresh {
            self.refresh_items_in_display();
            self.refresh_items_out_display();
            self.needs_items_refresh = false;
        }
    }
}

enum InOrOut {
    In,
    Out,
}

struct OnClickBehaviorSwitchItemsInOut {
    pub parent: Gd<ListModifyView>,
    pub in_or_out: InOrOut,
}

impl OnClickBehavior for OnClickBehaviorSwitchItemsInOut {
    fn on_click(&mut self, content: &Content) {
        if let Content::Item(item) = content {
            let mut parent = self.parent.bind_mut();
            // // Depending whether the item is in or out, move it from one list to the other
            match self.in_or_out {
                InOrOut::In => {
                    parent.list.items.retain(|elem| elem != item);
                    parent.items_out.push(item.clone());
                },
                InOrOut::Out => {
                    parent.items_out.retain(|elem| elem != item);
                    parent.list.items.push(item.clone());
                }
            }
            parent.needs_items_refresh = true;
        }
    }
}