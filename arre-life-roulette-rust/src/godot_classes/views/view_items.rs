use bus::BusReader;
use godot::engine::{Control, ControlVirtual, Button, LineEdit};
use godot::prelude::*;
use crate::db::DB;
use crate::errors::{ArreResult, BoxedError};
use crate::godot_classes::containers::cards_flow_container::CardsFlowContainer;
use crate::godot_classes::element_card::{Content, ElementCard};
use crate::godot_classes::singletons::logger::log_error;
use crate::godot_classes::singletons::signals::Signals;
use crate::godot_classes::utils::{GdHolder, get_singleton};
use crate::godot_classes::views::view_item_modify::ItemModifyView;
use crate::godot_classes::views::view_item_stats::ItemStatsView;
use crate::item::{Item, item_get_all, item_search};
use crate::item_stats::item_stats_get;

#[derive(GodotClass)]
#[class(base=Control)]
pub struct ItemsView {
    #[base]
    base: Base<Control>,

    // cached internal UI elements
    pub item_add_button: GdHolder<Button>,
    pub cards_container: GdHolder<CardsFlowContainer>,
    pub searchbar: GdHolder<LineEdit>,

    // cached external UI elements
    pub item_modify_view: GdHolder<ItemModifyView>,
    pub item_stats_view: GdHolder<ItemStatsView>,

    // observers
    observer_card_left_click: Option<BusReader<InstanceId>>,
    observer_card_right_click: Option<BusReader<InstanceId>>,

    // state
    items: Vec<Item>,
    search_term: Option<String>,
}

#[godot_api]
impl ItemsView {
    #[func]
    fn on_item_add_button_up(&mut self) {
        match try {
            self.item_modify_view.ok_mut().map(|view| {
                let mut view = view.bind_mut();
                view.set_mode_add();
                view.show();
            })?;
        } {
            Ok(_) => {},
            Err::<_, BoxedError>(e) => log_error(e)
        }
    }

    #[func]
    fn on_view_selected(&mut self) {
        self.refresh_full();
        self.show();
    }

    #[func]
    fn on_search_request(&mut self, search_term: GodotString) {
        let search_term = search_term.to_string();
        self.search_term = if search_term.is_empty() { None } else { Some(search_term) };
        self.refresh_full();
    }

    #[func]
    fn refresh_full(&mut self) {
        self.refresh_state();
        self.refresh_display();
    }

    #[func]
    fn refresh_state(&mut self) {
        match try {
            let connection = &*DB.ok()?;
            match &self.search_term {
                Some(search_term) => {
                    self.items = item_search(connection, search_term)?;
                },
                None => {
                    self.items = item_get_all(connection)?;
                }
            }
        } {
            Ok(_) => {},
            Err::<_, BoxedError>(e) => log_error(e)
        }
    }

    #[func]
    fn refresh_display(&mut self) {
        match try {
            self.cards_container.ok_mut()?.bind_mut().set_cards(self.items.clone());
        } {
            Ok(_) => {},
            Err::<_, BoxedError>(e) => log_error(e)
        }
    }

    fn on_item_card_left_click(&mut self, card_id: InstanceId) -> ArreResult<()> {
        let mut card = GdHolder::<ElementCard>::from_instance_id(card_id);
        {
            let card = card.ok_mut()?.bind();
            if let Content::Item(item) = &card.content {
                let connection = &*DB.ok()?;

                let mut view = self.item_stats_view.ok_mut()?.bind_mut();
                view.item_stats = item_stats_get(connection, item.get_id()?)?;
                view.refresh_display();
                view.show();
            }
        }
        Ok(())
    }

    fn on_item_card_right_click(&mut self, card_id: InstanceId) -> ArreResult<()> {
        let mut card = GdHolder::<ElementCard>::from_instance_id(card_id);
        {
            let card = card.ok_mut()?.bind();
            if let Content::Item(item) = &card.content {
                let mut view = self.item_modify_view.ok_mut()?.bind_mut();
                view.set_mode_edit(item.clone())?;
                view.show();
            }
        }
        Ok(())
    }
}

#[godot_api]
impl ControlVirtual for ItemsView {
    fn init(base: Base<Self::Base>) -> Self {
        Self {
            base,

            // cached internal UI elements
            item_add_button: GdHolder::default(),
            cards_container: GdHolder::default(),
            searchbar: GdHolder::default(),

            // cached external UI elements
            item_modify_view: GdHolder::default(),
            item_stats_view: GdHolder::default(),

            // observers
            observer_card_left_click: None,
            observer_card_right_click: None,

            // state
            items: vec![],
            search_term: None,
        }
    }
    fn ready(&mut self) {
        match try {
            let base = &self.base;
            self.item_add_button = GdHolder::from_path(base, "VBoxContainer/MarginContainer/ItemAddDialogButton");
            self.item_add_button.ok_mut()?.connect(
                "button_up".into(),
                base.callable("on_item_add_button_up"),
                0,
            );
            self.cards_container = GdHolder::from_path(base,"VBoxContainer/ItemsListScrollContainer/CardsFlowContainer");
            self.cards_container.ok_mut().map(|cc| {
                let mut cc = cc.bind_mut();
                self.observer_card_left_click = cc.bus_card_left_click.add_rx();
                self.observer_card_right_click = cc.bus_card_right_click.add_rx();
            })?;
            self.searchbar = GdHolder::from_path(base,"VBoxContainer/SearchBarLineEdit");
            self.searchbar.ok_mut()?.connect(
                "text_submitted".into(),
                base.callable("on_search_request"),
                0,
            );

            self.item_modify_view = GdHolder::from_path(base, "../../ItemModifyView");
            self.item_modify_view.ok_mut()?.bind_mut().connect(
                "dialog_closed".into(),
                base.callable("refresh_full"),
                0,
            );
            self.item_stats_view = GdHolder::from_path(base, "../../ItemStatsView");


            // Get singleton and connect to global signals(show / hide)
            let mut signals = get_singleton::<Signals>("Signals");
            {
                let mut signals = signals.bind_mut();
                signals.connect(
                    "item_view_tab_selected".into(),
                    base.callable("on_view_selected"),
                    0,
                );
                signals.connect(
                    "list_view_tab_selected".into(),
                    base.callable("hide"),
                    0,
                );
                signals.connect(
                    "tag_view_tab_selected".into(),
                    base.callable("hide"),
                    0,
                );
            }

            if self.is_visible() {
                self.refresh_full();
            }
        } {
            Ok(_) => {}
            Err::<_, BoxedError>(e) => log_error(e),
        }
    }
    fn process(&mut self, _delta: f64) {
        match try {
            // Item cards LEFT click listener
            if let Some(observer) = &mut self.observer_card_left_click {
                if let Ok(card) = observer.try_recv() {
                    self.on_item_card_left_click(card)?;
                }
            }
            // Item cards RIGHT click listener
            if let Some(observer) = &mut self.observer_card_right_click {
                if let Ok(card) = observer.try_recv() {
                    self.on_item_card_right_click(card)?;
                }
            }
        } {
            Ok(_) => {}
            Err::<_, BoxedError>(e) => log_error(e)
        }
    }
}