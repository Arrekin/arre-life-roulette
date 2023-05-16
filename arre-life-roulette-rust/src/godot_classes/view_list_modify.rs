use godot::engine::{Panel, PanelVirtual, LineEdit, TextEdit, Button, Label};
use godot::prelude::*;
use crate::errors::{ArreResult};
use crate::godot_classes::containers::cards_flow_container::CardsFlowContainer;
use crate::godot_classes::singletons::globals::{Globals};
use crate::godot_classes::element_card::{OnClickBehavior, Content};
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

    // cached elements
    title_label: GdHolder<Label>,
    name_line_edit: GdHolder<LineEdit>,
    description_text_edit: GdHolder<TextEdit>,
    cards_in_container: GdHolder<CardsFlowContainer>,
    cards_out_container: GdHolder<CardsFlowContainer>,
    apply_button: GdHolder<Button>,
    close_button: GdHolder<Button>,

    // state
    list: List,
    items_in: Vec<Item>,
    items_out: Vec<Item>,
    mode: Mode,

    // internal
    needs_deferred_display_update: bool,
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
            self.refresh_state();
            self.refresh_display();
        }: ArreResult<()> {
            Ok(_) => {}
            Err(err) => { log_error(err);}
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

            let self_reference = self.base.share().cast::<Self>();
            self.cards_in_container.ok_mut()?.bind_mut().set_cards(
                self.items_in.clone(),
                |mut card| {
                    card.on_left_click_behavior = Some(Box::new(OnClickBehaviorSwitchItemsInOut {
                        parent: self_reference.share(),
                        in_or_out: InOrOut::In
                    }));
                }
            );
            self.cards_out_container.ok_mut()?.bind_mut().set_cards(
                self.items_out.clone(),
                |mut card| {
                    card.on_left_click_behavior = Some(Box::new(OnClickBehaviorSwitchItemsInOut {
                        parent: self_reference.share(),
                        in_or_out: InOrOut::Out
                    }))
                }
            )
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

            title_label: GdHolder::default(),
            name_line_edit: GdHolder::default(),
            description_text_edit: GdHolder::default(),
            cards_in_container: GdHolder::default(),
            cards_out_container: GdHolder::default(),
            apply_button: GdHolder::default(),
            close_button: GdHolder::default(),

            list: List::default(),
            items_in: vec![],
            items_out: vec![],
            mode: Mode::Add,

            needs_deferred_display_update: false
        }
    }
    fn ready(&mut self) {
        match try {
            let base = &self.base;
            self.title_label = GdHolder::from_path(base, "VBoxContainer/TopMarginContainer/TitleLabel");
            self.name_line_edit = GdHolder::from_path(base, "VBoxContainer/ListNameLineEdit");
            self.description_text_edit = GdHolder::from_path(base, "VBoxContainer/ListDescriptionTextEdit");
            self.cards_in_container = GdHolder::from_path(base, "VBoxContainer/VBoxContainer/ScrollContainerIn/CardsInContainer");
            self.cards_out_container = GdHolder::from_path(base, "VBoxContainer/VBoxContainer/ScrollContainerOut/CardsOutContainer");
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
        }: ArreResult<()> {
            Ok(_) => {},
            Err(e) => { log_error(e); }
        }

    }

    fn process(&mut self, _delta: f64) {
        if self.needs_deferred_display_update {
            self.needs_deferred_display_update = false;
            self.refresh_display();
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
            parent.needs_deferred_display_update = true;
        }
    }
}