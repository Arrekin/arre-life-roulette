use godot::builtin::{Callable};
use godot::engine::{Panel, PanelVirtual, LineEdit, TextEdit, Button, GridContainer, Label};
use godot::engine::node::InternalMode;
use godot::engine::packed_scene::GenEditState;
use godot::prelude::*;
use crate::errors::{ArreError, ArreResult};
use crate::godot_classes::singletons::globals::{Globals};
use crate::godot_classes::resources::SELECTION_BUTTON_PREFAB;
use crate::godot_classes::selection_button::{SelectionButton, OnClickBehavior, Content};
use crate::godot_classes::singletons::logger::log_error;
use crate::godot_classes::utils::{GdHolder, get_singleton};
use crate::item::{Item, item_get_all, items_to_ids};
use crate::list::{List, list_create, list_items_get, list_items_get_complement, list_items_update, list_update};

const UI_TEXT_CREATE: &str = "Create List";
const UI_TEXT_MODIFY: &str = "Modify List";

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
    title_label: GdHolder<Label>,
    name_line_edit: GdHolder<LineEdit>,
    description_text_edit: GdHolder<TextEdit>,
    items_in_grid: GdHolder<GridContainer>,
    items_in_grid_elements: Vec<Gd<SelectionButton>>,
    items_out_grid: GdHolder<GridContainer>,
    items_out_grid_elements: Vec<Gd<SelectionButton>>,
    apply_button: GdHolder<Button>,
    close_button: GdHolder<Button>,

    // state
    list: List,
    items_in: Vec<Item>,
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
        match try {
            let new_name = self.name_line_edit.ok()?.get_text().to_string();
            let new_description = self.description_text_edit.ok()?.get_text().to_string();

            let globals = get_singleton::<Globals>("Globals");
            let connection = &globals.bind().connection;

            match self.mode {
                Mode::Add => {
                    let new_list = list_create(connection, new_name, new_description)?;
                    let items = items_to_ids::<Vec<_>>(&self.items_in)?;
                    list_items_update(connection, new_list.get_id()?, items)?;
                    self.set_mode_edit(new_list);
                }
                Mode::Edit => {
                    self.list.name = new_name;
                    self.list.description = new_description;
                    list_update(connection, &self.list)?;
                    let items = items_to_ids::<Vec<_>>(&self.items_in)?;
                    list_items_update(connection, self.list.get_id()?, items)?;
                }
            }
            self.needs_full_refresh = true;
        }: ArreResult<()> {
            Ok(_) => {}
            Err(err) => { log_error(err);}
        }
    }

    fn refresh_ui_display(&mut self) {
        match try {
            self.name_line_edit.ok_mut()?.set_text(self.list.name.clone().into());
            self.description_text_edit.ok_mut()?.set_text(self.list.description.clone().into());
            match self.mode {
                Mode::Add => {
                    self.title_label.ok_mut()?.set_text(UI_TEXT_CREATE.into());
                    self.apply_button.ok_mut()?.set_text(UI_TEXT_CREATE.into());
                }
                Mode::Edit => {
                    self.title_label.ok_mut()?.set_text(UI_TEXT_MODIFY.into());
                    self.apply_button.ok_mut()?.set_text(UI_TEXT_MODIFY.into());
                }
            }
        }: ArreResult<()> {
            Ok(_) => {}
            Err(err) => { log_error(err); }
        }
    }

    fn refresh_items_in_display(&mut self) {
        match try {
            // Clear old and create a button for each item
            self.items_in_grid_elements.drain(..).for_each(|mut item| item.bind_mut().queue_free());
            let new_items = self.items_in.iter().map(
                |item| {
                    let instance = self.item_selection_button
                        .instantiate(GenEditState::GEN_EDIT_STATE_DISABLED)
                        .ok_or(ArreError::InstantiateFailed(
                            SELECTION_BUTTON_PREFAB.into(),
                            "ListViewModify::refresh_items_in_display".into())
                        )?;
                    self.items_in_grid.ok_mut()?.add_child(instance.share(), false, InternalMode::INTERNAL_MODE_DISABLED);
                    let mut button = instance.cast::<SelectionButton>();
                    {
                        let mut button = button.bind_mut();
                        button.set_item(item.clone());
                        button.on_left_click_behavior = Some(Box::new(OnClickBehaviorSwitchItemsInOut {
                            parent: self.base.share().cast::<Self>(),
                            in_or_out: InOrOut::In
                        }));
                    }
                    Ok(button)
                }
            ).collect::<ArreResult<Vec<_>>>()?;
            self.items_in_grid_elements.extend(new_items);
        }: ArreResult<()> {
            Ok(_) => {}
            Err(err) => { log_error(err);}
        }
    }

    fn refresh_items_out_display(&mut self) {
        match try {
            // Clear old and create a button for each item
            self.items_out_grid_elements.drain(..).for_each(|mut item| item.bind_mut().queue_free());
            let new_items = self.items_out.iter().map(
                |item| {
                    let instance = self.item_selection_button
                        .instantiate(GenEditState::GEN_EDIT_STATE_DISABLED)
                        .ok_or(ArreError::InstantiateFailed(
                            SELECTION_BUTTON_PREFAB.into(),
                            "ListViewModify::refresh_items_out_display".into())
                        )?;
                    self.items_out_grid.ok_mut()?.add_child(instance.share(), false, InternalMode::INTERNAL_MODE_DISABLED);
                    let mut button = instance.cast::<SelectionButton>();
                    {
                        let mut button = button.bind_mut();
                        button.set_item(item.clone());
                        button.on_left_click_behavior = Some(Box::new(OnClickBehaviorSwitchItemsInOut {
                            parent: self.base.share().cast::<Self>(),
                            in_or_out: InOrOut::Out
                        }));
                    }
                    Ok(button)
                }
            ).collect::<ArreResult<Vec<_>>>()?;
            self.items_out_grid_elements.extend(new_items);
        }: ArreResult<()> {
            Ok(_) => {}
            Err(err) => { log_error(err);}
        }
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
        match self.mode {
            Mode::Add => {
                self.items_out = item_get_all(connection).unwrap();
                self.items_in = vec![];
            },
            Mode::Edit => {
                let list_id = self.list.get_id().unwrap();
                self.items_out = list_items_get_complement(connection, list_id).unwrap();
                self.items_in = list_items_get(connection, list_id).unwrap();
            }
        }
    }

}

#[godot_api]
impl PanelVirtual for ListModifyView {
    fn init(base: Base<Self::Base>) -> Self {
        Self {
            base,

            item_selection_button: load(SELECTION_BUTTON_PREFAB),

            title_label: GdHolder::default(),
            name_line_edit: GdHolder::default(),
            description_text_edit: GdHolder::default(),
            items_in_grid: GdHolder::default(),
            items_in_grid_elements: vec![],
            items_out_grid: GdHolder::default(),
            items_out_grid_elements: vec![],
            apply_button: GdHolder::default(),
            close_button: GdHolder::default(),

            list: List::default(),
            items_in: vec![],
            items_out: vec![],
            mode: Mode::Add,

            needs_full_refresh: false,
            needs_items_refresh: false,
        }
    }
    fn ready(&mut self) {
        match try {
            let base = &self.base;
            self.title_label = GdHolder::from_path(base, "VBoxContainer/TopMarginContainer/TitleLabel");
            self.name_line_edit = GdHolder::from_path(base, "VBoxContainer/ListNameLineEdit");
            self.description_text_edit = GdHolder::from_path(base, "VBoxContainer/ListDescriptionTextEdit");
            self.items_in_grid = GdHolder::from_path(base, "VBoxContainer/VBoxContainer/ListItemsInScrollContainer/ListItemsInGridContainer");
            self.items_out_grid = GdHolder::from_path(base, "VBoxContainer/VBoxContainer/ListItemsOutScrollContainer/ListItemsOutGridContainer");
            self.apply_button = GdHolder::from_path(base, "VBoxContainer/BottomMarginContainer/ListApplyButton");
            self.apply_button.ok_mut()?.connect(
                "button_up".into(),
                Callable::from_object_method(self.base.share(), "on_apply_list_button_up"),
                0,
            );
            self.close_button = GdHolder::from_path(base, "DialogCloseButton");
            self.close_button.ok_mut()?.connect(
                "button_up".into(),
                Callable::from_object_method(self.base.share(), "on_dialog_close_button_up"),
                0,
            );
        }: ArreResult<()> {
            Ok(_) => {},
            Err(e) => { log_error(e); }
        }

    }

    fn process(&mut self, _delta: f64) {
        if self.needs_full_refresh {
            self.refresh_ui_display();
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
                    parent.items_in.retain(|elem| elem != item);
                    parent.items_out.push(item.clone());
                },
                InOrOut::Out => {
                    parent.items_out.retain(|elem| elem != item);
                    parent.items_in.push(item.clone());
                }
            }
            parent.needs_items_refresh = true;
        }
    }
}