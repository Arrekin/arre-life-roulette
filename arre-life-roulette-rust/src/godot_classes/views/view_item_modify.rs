use chrono::Duration;
use godot::engine::{Panel, PanelVirtual, LineEdit, TextEdit, Button, Label, CheckButton, SpinBox};
use godot::prelude::*;
use crate::db::DB;
use crate::errors::{ArreResult, BoxedError};
use crate::godot_classes::singletons::logger::log_error;
use crate::godot_classes::utils::{GdHolder};
use crate::item::{Item, item_persist, item_update};
use crate::item_details::{item_details_get, item_details_update, ItemDetails};

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

    // cached internal UI elements
    title_label: GdHolder<Label>,
    name_line_edit: GdHolder<LineEdit>,
    description_text_edit: GdHolder<TextEdit>,
    session_time_check_button: GdHolder<CheckButton>,
    session_time_spin_box: GdHolder<SpinBox>,
    apply_button: GdHolder<Button>,
    close_button: GdHolder<Button>,

    // state
    item: Item,
    item_details: ItemDetails,
    mode: Mode,
}

#[godot_api]
impl ItemModifyView {
    #[signal]
    fn dialog_closed();

    #[func]
    fn on_apply_item_button_up(&mut self) {
        match try {
            self.item.name = self.name_line_edit.ok()?.get_text().to_string();
            self.item.description = self.description_text_edit.ok()?.get_text().to_string();
            self.item_details.session_duration =
                if self.session_time_check_button.ok()?.is_pressed() {
                    Some(Duration::minutes(self.session_time_spin_box.ok()?.get_value() as i64))
                } else {
                    None
                };

            let connection = &*DB.ok()?;
            match self.mode {
                Mode::Add => {
                    self.mode = Mode::Edit;
                    item_persist(connection, &mut self.item)?;
                    self.item_details.id = self.item.id;
                    item_details_update(connection, &self.item_details)?;
                },
                Mode::Edit => {
                    item_update(connection, &self.item)?;
                    item_details_update(connection, &self.item_details)?;
                }
            };

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
            if let Some(session_duration) = self.item_details.session_duration {
                self.session_time_spin_box.ok_mut()?.set_value(session_duration.num_minutes() as f64);
                self.session_time_spin_box.ok_mut()?.set_editable(true);
                self.session_time_check_button.ok_mut()?.call_deferred(
                    // Deferred call, as it triggers `toggle` signal, which this class in handling
                    // resulting in mutable re-borrow of `self` otherwise
                    "set_pressed".into() , &[true.to_variant()]
                );
            } else {
                self.session_time_spin_box.ok_mut()?.set_value(5.);
                self.session_time_spin_box.ok_mut()?.set_editable(false);
                self.session_time_check_button.ok_mut()?.call_deferred(
                    "set_pressed".into() , &[false.to_variant()]
                );
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

    #[func]
    fn on_session_time_check_button_toggled(&mut self, checked: bool) {
        match try {
        self.session_time_spin_box.ok_mut()?.set_editable(checked);
        } {
            Ok(_) => {}
            Err::<_, BoxedError>(e) => log_error(e)
        }
    }

    pub fn set_mode_add(&mut self) {
        self.mode = Mode::Add;
        self.item = Item::default();
        self.item_details = ItemDetails::default();
        self.refresh_display();
    }

    pub fn set_mode_edit(&mut self, item: Item) -> ArreResult<()> {
        self.mode = Mode::Edit;
        self.item = item;

        let connection = &*DB.ok()?;
        self.item_details = item_details_get(connection, self.item.get_id()?)?;

        self.refresh_display();
        Ok(())
    }

}

#[godot_api]
impl PanelVirtual for ItemModifyView {
    fn init(base: Base<Self::Base>) -> Self {
        Self {
            base,

            // cached internal UI elements
            title_label: GdHolder::default(),
            name_line_edit: GdHolder::default(),
            description_text_edit: GdHolder::default(),
            session_time_check_button: GdHolder::default(),
            session_time_spin_box: GdHolder::default(),
            apply_button: GdHolder::default(),
            close_button: GdHolder::default(),

            // state
            item: Item::default(),
            item_details: ItemDetails::default(),
            mode: Mode::Add,
        }
    }
    fn ready(&mut self) {
        match try {
            let base = &self.base;
            self.title_label = GdHolder::from_path(base, "VBoxContainer/TopMarginContainer/TitleLabel");
            self.name_line_edit = GdHolder::from_path(base,"VBoxContainer/CentralMarginContainer/VBoxContainer/ItemNameLineEdit");
            self.description_text_edit = GdHolder::from_path(base,"VBoxContainer/CentralMarginContainer/VBoxContainer/ItemDescriptionTextEdit");
            self.session_time_check_button = GdHolder::from_path(base,"VBoxContainer/CentralMarginContainer/VBoxContainer/SessionTimeHBoxContainer/CheckButton");
            self.session_time_check_button.ok_mut()?.connect(
                "toggled".into(),
                base.callable("on_session_time_check_button_toggled"),
            );
            self.session_time_spin_box = GdHolder::from_path(base,"VBoxContainer/CentralMarginContainer/VBoxContainer/SessionTimeHBoxContainer/SpinBox");
            self.apply_button = GdHolder::from_path(base,"VBoxContainer/BottomMarginContainer/ItemApplyButton");
            self.apply_button.ok_mut()?.connect(
                "button_up".into(),
                base.callable("on_apply_item_button_up"),
            );
            self.close_button = GdHolder::from_path(base,"DialogCloseButton");
            self.close_button.ok_mut()?.connect(
                "button_up".into(),
                base.callable("on_dialog_close_button_up"),
            );
        } {
            Ok(_) => {}
            Err::<_, BoxedError>(e) => log_error(e)
        }
    }
}