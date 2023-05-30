use std::collections::HashMap;
use bus::BusReader;
use godot::engine::{Button, VBoxContainer, VBoxContainerVirtual};
use godot::prelude::*;
use crate::errors::{ArreResult};
use crate::godot_classes::containers::cards_flow_container::CardsFlowContainer;
use crate::godot_classes::element_card::{Content, ElementCard};
use crate::godot_classes::singletons::globals::{Globals};
use crate::godot_classes::singletons::logger::log_error;
use crate::godot_classes::utils::{GdHolder, get_singleton};
use crate::item::{Item, ItemId};
use crate::list::{list_items_get, ListId};

#[derive(Clone)]
struct SelectionItem {
    item: Item,
    selected: bool,
}

impl Into<Content> for SelectionItem {
    fn into(self) -> Content {
        Content::Item(self.item)
    }
}

#[derive(GodotClass)]
#[class(base=VBoxContainer)]
pub struct RollSelectionSubview {
    #[base]
    base: Base<VBoxContainer>,

    // cached internal UI elements
    cards_container: GdHolder<CardsFlowContainer>,
    roll_start_button: GdHolder<Button>,

    // observers
    observer_card_left_click: Option<BusReader<InstanceId>>,

    // state
    list_id: ListId,
    items: HashMap<ItemId, SelectionItem>,
}

#[godot_api]
impl RollSelectionSubview {

    pub fn set_state(&mut self, list_id: ListId) {
        self.list_id = list_id;
        self.refresh_state();
    }

    pub fn refresh_state(&mut self) {
        match try {
            let globals = get_singleton::<Globals>("Globals");
            let connection = &globals.bind().connection;

            self.items = list_items_get::<Vec<_>>(connection, self.list_id)?
                .into_iter()
                .map(|item| {
                    Ok((item.get_id()?, SelectionItem { item, selected: true, }))
                }).collect::<ArreResult<_>>()?;
        }: ArreResult<()> {
            Ok(_) => {}
            Err(e) => log_error(e)
        }
    }

    pub fn refresh_display(&mut self) {
        match try {
            let mut cards_container = self.cards_container.ok_mut()?.bind_mut();
            cards_container.set_cards(self.items.values().cloned().collect());
        }: ArreResult<()> {
            Ok(_) => {}
            Err(e) => log_error(e)
        }
    }

    #[func]
    fn on_roll_start_button_up(&mut self) {
        // if self.items.len() == 0 {
        //     log_error(ArreError::ListHasNoItems(self.list.name.clone()));
        //     return;
        // }
        // let mut rng = rand::thread_rng();
        // self.work_item = rng.gen_range(0..self.items.len());
        // self.work_start_timestamp = Utc::now();
        //
        // self.roll_state = RollState::WorkAssigned;
        // godot_print!("Selected work item: {}", self.work_item);
        // self.refresh_view();
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

            // observers
            observer_card_left_click: None,

            // state
            list_id: 0.into(),
            items: HashMap::new(),
        }
    }
    fn ready(&mut self) {
        match try {
            let base = &self.base;

            self.cards_container = GdHolder::from_path(base, "TopMarginContainer/PanelContainer/ScrollContainer/CardsFlowContainer");
            self.observer_card_left_click = self.cards_container.ok_mut()?.bind_mut().bus_card_left_click.add_rx();

            self.roll_start_button = GdHolder::from_path(base, "BottomMarginContainer/RollStartButton");
            self.roll_start_button.ok_mut()?.connect(
                "button_up".into(),
                base.callable("on_roll_start_button_up"),
                0,
            );
        }: ArreResult<()> {
            Ok(_) => {}
            Err(e) => log_error(e),
        }
    }
}
