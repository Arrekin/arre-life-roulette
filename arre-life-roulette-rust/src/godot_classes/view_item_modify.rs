use godot::builtin::{Callable};
use godot::engine::{Panel, PanelVirtual, LineEdit, TextEdit, Button, NodeExt, Label};
use godot::prelude::*;
use crate::errors::ArreError;
use crate::godot_classes::singletons::globals::{Globals};
use crate::godot_classes::singletons::logger::log_error;
use crate::godot_classes::utils::get_singleton;
use crate::item::{Item, item_persist, item_update};

const UI_TEXT_CREATE: &str = "Create Item";
const UI_TEXT_MODIFY: &str = "Modify Item";

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
    title_label: Option<Gd<Label>>,
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
        let new_name = {
            if let Some(line_edit) = self.name_line_edit.as_ref() {
                line_edit.get_text().to_string()
            } else {
                log_error(ArreError::NullGd("ItemModifyView::on_apply_item_button_up::name_line_edit".into()));
                return;
            }
        };
        let new_description = {
            if let Some(text_edit) =  self.description_text_edit.as_ref() {
                text_edit.get_text().to_string()
            } else {
                log_error(ArreError::NullGd("ItemModifyView::on_apply_item_button_up::description_text_edit".into()));
                return;
            }
        };

        let globals = get_singleton::<Globals>("Globals");
        let connection = &globals.bind().connection;

        self.item.name = new_name;
        self.item.description = new_description;

        match self.mode {
            Mode::Add => {
                self.mode = Mode::Edit;
                item_persist(connection, &mut self.item)
            },
            Mode::Edit => {
                item_update(connection, &self.item)
            }
        }.unwrap_or_else(|e| log_error(e.as_ref()));

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
        match self.mode {
            Mode::Add => {
                self.title_label.as_mut().map(|label| label.set_text(UI_TEXT_CREATE.into()));
                self.apply_button.as_mut().map(|button| button.set_text(UI_TEXT_CREATE.into()));
            }
            Mode::Edit => {
                self.title_label.as_mut().map(|label| label.set_text(UI_TEXT_MODIFY.into()));
                self.apply_button.as_mut().map(|button| button.set_text(UI_TEXT_MODIFY.into()));
            }
        }
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
impl PanelVirtual for ItemModifyView {
    fn init(base: Base<Self::Base>) -> Self {
        Self {
            base,
            title_label: None,
            name_line_edit: None,
            description_text_edit: None,
            apply_button: None,
            close_button: None,

            item: Item::default(),
            mode: Mode::Add,
        }
    }
    fn ready(&mut self) {
        self.title_label = self.base.try_get_node_as("VBoxContainer/TopMarginContainer/TitleLabel");
        if self.title_label.is_none() { log_error(ArreError::NullGd("ItemModifyView::ready::title_label".into())); }
        self.name_line_edit = self.base.try_get_node_as("VBoxContainer/CentralMarginContainer/VBoxContainer/ItemNameLineEdit");
        if self.name_line_edit.is_none() { log_error(ArreError::NullGd("ItemModifyView::ready::name_line_edit".into())); }
        self.description_text_edit = self.base.try_get_node_as("VBoxContainer/CentralMarginContainer/VBoxContainer/ItemDescriptionTextEdit");
        if self.description_text_edit.is_none() { log_error(ArreError::NullGd("ItemModifyView::ready::description_text_edit".into())); }
        self.apply_button = self.base.try_get_node_as("VBoxContainer/BottomMarginContainer/ItemApplyButton");
        self.apply_button.as_mut().map_or_else(
            || log_error(ArreError::NullGd("ItemModifyView::ready::apply_button".into())),
            |button| {
                button.connect(
                    "button_up".into(),
                    Callable::from_object_method(self.base.share(), "on_apply_item_button_up"),
                    0,
                );
            }
        );
        self.close_button = self.base.try_get_node_as("DialogCloseButton");
        self.close_button.as_mut().map_or_else(
            || log_error(ArreError::NullGd("ItemModifyView::ready::close_button".into())),
            |button| {
                button.connect(
                    "button_up".into(),
                    Callable::from_object_method(self.base.share(), "on_dialog_close_button_up"),
                    0,
                );
            }
        );
    }
}