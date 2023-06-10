use godot::engine::{Panel, PanelVirtual, LineEdit, TextEdit, Button, Label};
use godot::prelude::*;
use crate::db::DB;
use crate::errors::{BoxedError};
use crate::godot_classes::singletons::logger::log_error;
use crate::godot_classes::utils::{GdHolder};
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
    title_label: GdHolder<Label>,
    name_line_edit: GdHolder<LineEdit>,
    description_text_edit: GdHolder<TextEdit>,
    apply_button: GdHolder<Button>,
    close_button: GdHolder<Button>,

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
        match try {
            let new_name = self.name_line_edit.ok()?.get_text().to_string();
            let new_description = self.description_text_edit.ok()?.get_text().to_string();

            let connection = &*DB.ok()?;

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
            }?;

            self.refresh_display();
        } {
            Ok(_) => {}
            Err::<_, BoxedError>(e) => log_error(e)
        }
    }

    #[func]
    fn refresh_display(&mut self) {
        match try {
            self.name_line_edit.ok_mut()?.set_text(self.item.name.clone().into());
            self.description_text_edit.ok_mut()?.set_text(self.item.description.clone().into());

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
        } {
            Ok(_) => {}
            Err::<_, BoxedError>(e) => log_error(e)
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
            title_label: GdHolder::default(),
            name_line_edit: GdHolder::default(),
            description_text_edit: GdHolder::default(),
            apply_button: GdHolder::default(),
            close_button: GdHolder::default(),

            item: Item::default(),
            mode: Mode::Add,
        }
    }
    fn ready(&mut self) {
        match try {
            let base = &self.base;
            self.title_label = GdHolder::from_path(base, "VBoxContainer/TopMarginContainer/TitleLabel");
            self.name_line_edit = GdHolder::from_path(base,"VBoxContainer/CentralMarginContainer/VBoxContainer/ItemNameLineEdit");
            self.description_text_edit = GdHolder::from_path(base,"VBoxContainer/CentralMarginContainer/VBoxContainer/ItemDescriptionTextEdit");
            self.apply_button = GdHolder::from_path(base,"VBoxContainer/BottomMarginContainer/ItemApplyButton");
            self.apply_button.ok_mut()?.connect(
                "button_up".into(),
                base.callable("on_apply_item_button_up"),
                0,
            );
            self.close_button = GdHolder::from_path(base,"DialogCloseButton");
            self.close_button.ok_mut()?.connect(
                "button_up".into(),
                base.callable("on_dialog_close_button_up"),
                0,
            );
        } {
            Ok(_) => {}
            Err::<_, BoxedError>(e) => log_error(e)
        }
    }
}