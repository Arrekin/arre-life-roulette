use std::collections::HashMap;
use bus::BusReader;
use godot::engine::{Button, VBoxContainer, VBoxContainerVirtual};
use godot::prelude::*;
use crate::db::DB;
use crate::errors::{ArreResult, BoxedError};
use crate::godot_classes::containers::cards_flow_container::CardsFlowContainer;
use crate::godot_classes::element_card::{Content, ElementCard};
use crate::godot_classes::singletons::logger::log_error;
use crate::godot_classes::utils::{GdHolder};
use crate::godot_classes::views::roll::view_roll::{RollState, RollView};
use crate::item::{Item, ItemId};
use crate::list::{list_items_get, ListId};

#[derive(GodotClass)]
#[class(base=VBoxContainer)]
pub struct RollSelectionSubview {
    #[base]
    base: Base<VBoxContainer>,

    // cached internal UI elements
    pub cards_container: GdHolder<CardsFlowContainer>,
    pub roll_start_button: GdHolder<Button>,

    // cached external UI elements
    pub roll_view: GdHolder<RollView>,

    // observers
    observer_card_left_click: Option<BusReader<InstanceId>>,

    // state
    list_id: ListId,
    items: HashMap<ItemId, Item>,
    items_enabled: HashMap<ItemId, bool>,
}

#[godot_api]
impl RollSelectionSubview {

    pub fn set_state(&mut self, list_id: ListId) {
        self.list_id = list_id;
        self.refresh_state();
    }

    pub fn refresh_state(&mut self) {
        match try {
            let connection = &*DB.ok()?;

            self.items = list_items_get::<Vec<_>>(connection, self.list_id)?
                .into_iter()
                .map(|item| {
                    Ok((item.get_id()?, item))
                }).collect::<ArreResult<_>>()?;
            self.items_enabled = self.items.keys().map(|item_id| (*item_id, true)).collect();
        } {
            Ok(_) => {}
            Err::<_, BoxedError>(e) => log_error(e)
        }
    }

    pub fn refresh_display(&mut self) {
        match try {
            let mut cards_container = self.cards_container.ok_mut()?.bind_mut();
            cards_container.set_cards(self.items.values().cloned().collect());
        } {
            Ok(_) => {}
            Err::<_, BoxedError>(e) => log_error(e)
        }
    }

    #[func]
    fn on_roll_start_button_up(&mut self) {
        match try {
            // make list of enabled items to choose from
            let work_items = self.items
                .iter()
                .filter(|(item_id, _)| self.items_enabled[*item_id])
                .map(|(_, item)| item.clone())
                .collect::<Vec<_>>();
            self.roll_view.ok_mut()?.bind_mut().roll_state_change_request(RollState::Rolling(work_items));
        } {
            Ok(_) => {}
            Err::<_, BoxedError>(e) => log_error(e)
        }
    }

    fn on_item_card_left_click(&mut self, card_id: InstanceId) -> ArreResult<()> {
        let mut card = GdHolder::<ElementCard>::from_instance_id(card_id);
        let mut card = card.ok_mut()?.bind_mut();
        if let Content::Item(item) = &card.content {
            let item_id = item.get_id()?;
            let was_item_enabled = self.items_enabled[&item_id];
            let is_item_enabled = !was_item_enabled;
            self.items_enabled.insert(item_id, is_item_enabled);
            card.set_modulate(
                if is_item_enabled { Color::from_rgba(1.0, 1.0, 1.0, 1.0) } else { Color::from_rgba(1.0, 1.0, 1.0, 0.3) }
            );
        }
        Ok(())
    }
}

#[godot_api]
impl VBoxContainerVirtual for RollSelectionSubview {
    fn init(base: Base<Self::Base>) -> Self {
        Self {
            base,

            // cached internal UI elements
            cards_container: GdHolder::default(),
            roll_start_button: GdHolder::default(),

            // cached external UI elements
            roll_view: GdHolder::default(),

            // observers
            observer_card_left_click: None,

            // state
            list_id: 0.into(),
            items: HashMap::new(),
            items_enabled: HashMap::new(),
        }
    }
    fn ready(&mut self) {
        match try {
            let base = &self.base;

            // cached internal UI elements
            self.cards_container = GdHolder::from_path(base, "TopMarginContainer/PanelContainer/ScrollContainer/CardsFlowContainer");
            self.observer_card_left_click = self.cards_container.ok_mut()?.bind_mut().bus_card_left_click.add_rx();

            self.roll_start_button = GdHolder::from_path(base, "BottomMarginContainer/RollStartButton");
            self.roll_start_button.ok_mut()?.connect(
                "button_up".into(),
                base.callable("on_roll_start_button_up"),
                0,
            );

            // cached external UI elements
            // self.roll_view is set from RollView::ready()
        } {
            Ok(_) => {}
            Err::<_, BoxedError>(e) => log_error(e),
        }
    }
    fn process(&mut self, _delta: f64) {
        match try {
            if let Some(observer) = &mut self.observer_card_left_click {
                if let Ok(card) = observer.try_recv() {
                    self.on_item_card_left_click(card)?;
                }
            }
        } {
            Ok(_) => {}
            Err::<_, BoxedError>(e) => log_error(e),
        }
    }

}
