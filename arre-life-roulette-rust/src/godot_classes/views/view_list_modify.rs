use std::collections::HashSet;
use bus::BusReader;
use godot::engine::{Panel, PanelVirtual, LineEdit, TextEdit, Button, Label};
use godot::prelude::*;
use crate::db::DB;
use crate::errors::{ArreResult, BoxedError};
use crate::godot_classes::containers::cards_flow_container::CardsFlowContainer;
use crate::godot_classes::element_card::{ElementCard, Content};
use crate::godot_classes::singletons::logger::log_error;
use crate::godot_classes::utils::{GdHolder};
use crate::item::{Item, item_get_all, item_search, items_to_ids};
use crate::list::{List, list_create, list_items_get, list_items_get_complement, list_items_update, list_update};

const UI_TEXT_CREATE: &str = "Create List";
const UI_TEXT_MODIFY: &str = "Modify List";

enum Mode {
    Add,
    Edit,
}

struct DeferredActions {
    refresh_display: bool,
    save_name: bool,
    save_description: bool,
}
impl Default for DeferredActions {
    fn default() -> Self {
        Self {
            refresh_display: false,
            save_name: false,
            save_description: false,
        }
    }
}

/// View allowing List modifications
/// items_in: Items in the list
/// items_out: Items not on the list
#[derive(GodotClass)]
#[class(base=Panel)]
pub struct ListModifyView {
    #[base]
    base: Base<Panel>,

    // cached internal UI elements
    title_label: GdHolder<Label>,
    name_line_edit: GdHolder<LineEdit>,
    description_text_edit: GdHolder<TextEdit>,
    searchbar: GdHolder<LineEdit>,
    cards_in_container: GdHolder<CardsFlowContainer>,
    cards_out_container: GdHolder<CardsFlowContainer>,
    apply_button: GdHolder<Button>,
    close_button: GdHolder<Button>,

    // observers
    observer_card_in_left_click: Option<BusReader<InstanceId>>,
    observer_card_out_left_click: Option<BusReader<InstanceId>>,

    // state
    list: List,
    items_in: HashSet<Item>,
    items_out: HashSet<Item>,
    mode: Mode,
    search_term: Option<String>,

    // internal
    deferred_actions: DeferredActions,
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

            {
                let connection = &*DB.ok()?;
                match self.mode {
                    Mode::Add => {
                        let new_list = list_create(connection, new_name, new_description)?;
                        let items = items_to_ids::<_, Vec<_>>(self.items_in.iter())?;
                        list_items_update(connection, new_list.get_id()?, items)?;
                        self.set_mode_edit(new_list);
                    }
                    Mode::Edit => {
                        self.list.name = new_name;
                        self.list.description = new_description;
                        list_update(connection, &self.list)?;
                        let items = items_to_ids::<_, Vec<_>>(self.items_in.iter())?;
                        list_items_update(connection, self.list.get_id()?, items)?;
                    }
                }
            }

            self.refresh_state();
            self.refresh_display();
        } {
            Ok(_) => {}
            Err::<_, BoxedError>(e) => log_error(e)
        }
    }

    fn refresh_display(&mut self) {
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

            let display_items_in = self.get_display_items_in()?;
            self.cards_in_container.ok_mut()?.bind_mut().set_cards(display_items_in);
            let display_items_out = self.get_display_items_out()?;
            self.cards_out_container.ok_mut()?.bind_mut().set_cards(display_items_out)
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
    fn on_name_line_edit_text_set(&mut self) {
        self.deferred_actions.save_name = true;
    }

    #[func]
    fn on_description_text_edit_text_set(&mut self) {
        self.deferred_actions.save_description = true;
    }

    #[func]
    fn on_search_request(&mut self) {
        match try {
            let search_term = self.searchbar.ok()?.get_text().to_string();
            self.search_term = if search_term.is_empty() { None } else { Some(search_term) };
            self.deferred_actions.refresh_display = true;
        } {
            Ok(_) => {}
            Err::<_, BoxedError>(e) => log_error(e)
        }
    }

    pub fn set_mode_add(&mut self) {
        self.mode = Mode::Add;
        self.list = List::default();
        self.refresh_state();
        self.refresh_display();
    }

    pub fn set_mode_edit(&mut self, list: List) {
        self.mode = Mode::Edit;
        self.list = list;
        self.refresh_state();
        self.refresh_display();
    }

    fn refresh_state(&mut self) {
        match try {
            let connection = &*DB.ok()?;
            match self.mode {
                Mode::Add => {
                    self.items_out = item_get_all(connection)?;
                    self.items_in = HashSet::new();
                },
                Mode::Edit => {
                    let list_id = self.list.get_id()?;
                    self.items_out = list_items_get_complement(connection, list_id)?;
                    self.items_in = list_items_get(connection, list_id)?;
                }
            }
        } {
            Ok(_) => {}
            Err::<_, BoxedError>(e) => log_error(e)
        }
    }

    fn get_display_items_in(&self) -> ArreResult<Vec<Item>> {
        let connection = &*DB.ok()?;

        match &self.search_term {
            None => {
                Ok(self.items_in.iter().cloned().collect())
            }
            Some(search_term) => {
                let search_fitting_items = item_search(connection, search_term)?;
                Ok(self.items_in.intersection(&search_fitting_items).cloned().collect())
            }
        }
    }

    fn get_display_items_out(&self) -> ArreResult<Vec<Item>> {
        let connection = &*DB.ok()?;

        match &self.search_term {
            None => {
                Ok(self.items_out.iter().cloned().collect())
            }
            Some(search_term) => {
                let search_fitting_items = item_search(connection, search_term)?;
                Ok(self.items_out.intersection(&search_fitting_items).cloned().collect())
            }
        }
    }

    fn on_item_card_in_left_click(&mut self, card_id: InstanceId) -> ArreResult<()> {
        let mut card = GdHolder::<ElementCard>::from_instance_id(card_id);
        {
            let card = card.ok_mut()?.bind();
            if let Content::Item(item) = &card.content {
                self.items_in.remove(&item);
                self.items_out.insert(item.clone());
                self.deferred_actions.refresh_display = true;
            }
        }
        Ok(())
    }

    fn on_item_card_out_left_click(&mut self, card_id: InstanceId) -> ArreResult<()> {
        let mut card = GdHolder::<ElementCard>::from_instance_id(card_id);
        {
            let card = card.ok_mut()?.bind();
            if let Content::Item(item) = &card.content {
                self.items_out.remove(&item);
                self.items_in.insert(item.clone());
                self.deferred_actions.refresh_display = true;
            }
        }
        Ok(())
    }

}

#[godot_api]
impl PanelVirtual for ListModifyView {
    fn init(base: Base<Self::Base>) -> Self {
        Self {
            base,

            // cached internal UI elements
            title_label: GdHolder::default(),
            name_line_edit: GdHolder::default(),
            description_text_edit: GdHolder::default(),
            searchbar: GdHolder::default(),
            cards_in_container: GdHolder::default(),
            cards_out_container: GdHolder::default(),
            apply_button: GdHolder::default(),
            close_button: GdHolder::default(),

            // observers
            observer_card_in_left_click: None,
            observer_card_out_left_click: None,

            // state
            list: List::default(),
            items_in: HashSet::new(),
            items_out: HashSet::new(),
            mode: Mode::Add,
            search_term: None,

            // internal
            deferred_actions: DeferredActions::default(),
        }
    }
    fn ready(&mut self) {
        match try {
            let base = &self.base;
            self.title_label = GdHolder::from_path(base, "VBoxContainer/TopMarginContainer/TitleLabel");
            self.name_line_edit = GdHolder::from_path(base, "VBoxContainer/TextMarginContainer/VBoxContainer/ListNameLineEdit");
            self.name_line_edit.ok_mut()?.connect(
                "text_changed".into(),
                base.callable("on_name_line_edit_text_set"),
                0,
            );
            self.description_text_edit = GdHolder::from_path(base, "VBoxContainer/TextMarginContainer/VBoxContainer/ListDescriptionTextEdit");
            self.description_text_edit.ok_mut()?.connect(
                "text_changed".into(),
                base.callable("on_description_text_edit_text_set"),
                0,
            );
            self.searchbar = GdHolder::from_path(base, "VBoxContainer/SearchBarLineEdit");
            self.searchbar.ok_mut()?.connect(
                "text_submitted".into(),
                base.callable("on_search_request"),
                0,
            );
            self.cards_in_container = GdHolder::from_path(base, "VBoxContainer/HSplitContainer/PanelContainerIn/ScrollContainer/CardsInContainer");
            self.observer_card_in_left_click = self.cards_in_container.ok_mut()?.bind_mut().bus_card_left_click.add_rx();
            self.cards_out_container = GdHolder::from_path(base, "VBoxContainer/HSplitContainer/PanelContainerOut/ScrollContainer/CardsOutContainer");
            self.observer_card_out_left_click = self.cards_out_container.ok_mut()?.bind_mut().bus_card_left_click.add_rx();
            self.apply_button = GdHolder::from_path(base, "VBoxContainer/BottomMarginContainer/ListApplyButton");
            self.apply_button.ok_mut()?.connect(
                "button_up".into(),
                base.callable("on_apply_list_button_up"),
                0,
            );
            self.close_button = GdHolder::from_path(base, "DialogCloseButton");
            self.close_button.ok_mut()?.connect(
                "button_up".into(),
                base.callable("on_dialog_close_button_up"),
                0,
            );
        } {
            Ok(_) => {},
            Err::<_, BoxedError>(e) => log_error(e)
        }

    }

    fn process(&mut self, _delta: f64) {
        match try {
            // Item cards IN listener
            if let Some(observer) = &mut self.observer_card_in_left_click {
                if let Ok(card_id) = observer.try_recv() {
                    self.on_item_card_in_left_click(card_id)?;
                }
            }
            // Item cards OUT listener
            if let Some(observer) = &mut self.observer_card_out_left_click {
                if let Ok(card_id) = observer.try_recv() {
                    self.on_item_card_out_left_click(card_id)?;
                }
            }

            if self.deferred_actions.save_name {
                self.list.name = self.name_line_edit.ok()?.get_text().to_string();
            }
            if self.deferred_actions.save_description {
                self.list.description = self.description_text_edit.ok()?.get_text().to_string();
            }
            if self.deferred_actions.refresh_display {
                self.refresh_display();
            }
            self.deferred_actions = DeferredActions::default();
        } {
            Ok(_) => {},
            Err::<_, BoxedError>(e) => log_error(e)
        }
    }
}