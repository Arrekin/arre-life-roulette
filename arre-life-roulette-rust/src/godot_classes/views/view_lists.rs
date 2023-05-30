use bus::BusReader;
use godot::engine::{Control, ControlVirtual, Button, LineEdit};
use godot::prelude::*;
use crate::errors::{ArreResult};
use crate::godot_classes::containers::cards_flow_container::CardsFlowContainer;
use crate::godot_classes::singletons::globals::{Globals};
use crate::godot_classes::element_card::{Content, ElementCard};
use crate::godot_classes::singletons::logger::log_error;
use crate::godot_classes::singletons::signals::Signals;
use crate::godot_classes::utils::{GdHolder, get_singleton};
use crate::godot_classes::views::view_list_modify::ListModifyView;
use crate::godot_classes::views::roll::view_roll::RollView;
use crate::list::{List, list_get_all, list_search};

#[derive(GodotClass)]
#[class(base=Control)]
pub struct ListsView {
    #[base]
    base: Base<Control>,

    // cached internal UI elements
    pub list_add_button: GdHolder<Button>,
    pub cards_container: GdHolder<CardsFlowContainer>,
    pub searchbar: GdHolder<LineEdit>,

    // cached external UI elements
    pub list_roll_view: GdHolder<RollView>,
    pub list_modify_view: GdHolder<ListModifyView>,

    // observers
    observer_card_left_click: Option<BusReader<InstanceId>>,
    observer_card_right_click: Option<BusReader<InstanceId>>,

    // state
    lists: Vec<List>,
    search_term: Option<String>,
}

#[godot_api]
impl ListsView {
    #[func]
    fn on_list_add_button_up(&mut self) {
        match try {
            let mut view = self.list_modify_view.ok_mut()?.bind_mut();
            view.set_mode_add();
            view.set_visible(true);
        }: ArreResult<()> {
            Ok(_) => {},
            Err(e) => { log_error(e); }
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
            let globals = get_singleton::<Globals>("Globals");
            let connection = &globals.bind().connection;
            match &self.search_term {
                Some(search_term) => {
                    self.lists = list_search(connection, search_term)?;
                },
                None => {
                    self.lists = list_get_all(connection)?;
                }
            }
        }: ArreResult<()> {
            Ok(_) => {},
            Err(e) => { log_error(e); }
        }
    }

    #[func]
    fn refresh_display(&mut self) {
        match try {
            self.cards_container.ok_mut()?.bind_mut().set_cards(self.lists.clone());
        }: ArreResult<()> {
            Ok(_) => {},
            Err(e) => { log_error(e); }
        }
    }

    fn on_list_card_left_click(&mut self, card_id: InstanceId) -> ArreResult<()> {
        let mut card = GdHolder::<ElementCard>::from_instance_id(card_id);
        {
            let card = card.ok_mut()?.bind();
            if let Content::List(list) = &card.content {
                let mut view = self.list_roll_view.ok_mut()?.bind_mut();
                view.set_list(list.clone())?;
                view.refresh_view();
                view.show();
            }
        }
        Ok(())
    }

    fn on_list_card_right_click(&mut self, card_id: InstanceId) -> ArreResult<()> {
        let mut card = GdHolder::<ElementCard>::from_instance_id(card_id);
        {
            let card = card.ok_mut()?.bind();
            if let Content::List(list) = &card.content {
                let mut view = self.list_modify_view.ok_mut()?.bind_mut();
                view.set_mode_edit(list.clone());
                view.show();
            }
        }
        Ok(())
    }
}

#[godot_api]
impl ControlVirtual for ListsView {
    fn init(base: Base<Self::Base>) -> Self {
        Self {
            base,

            // cached internal UI elements
            list_add_button: GdHolder::default(),
            cards_container: GdHolder::default(),
            searchbar: GdHolder::default(),

            // cached external UI elements
            list_roll_view: GdHolder::default(),
            list_modify_view: GdHolder::default(),

            // observers
            observer_card_left_click: None,
            observer_card_right_click: None,

            lists: vec![],
            search_term: None,
        }
    }
    fn ready(&mut self) {
        match try {
            let base = &self.base;
            self.list_add_button = GdHolder::from_path(base, "VBoxContainer/MarginContainer/ListAddDialogButton");
            self.list_add_button.ok_mut()?.connect(
                "button_up".into(),
                base.callable("on_list_add_button_up"),
                0,
            );
            self.cards_container = GdHolder::from_path(base, "VBoxContainer/ListsListScrollContainer/CardsFlowContainer");
            self.cards_container.ok_mut().map(|cc| {
                let mut cc = cc.bind_mut();
                self.observer_card_left_click = cc.bus_card_left_click.add_rx();
                self.observer_card_right_click = cc.bus_card_right_click.add_rx();
            })?;
            self.searchbar = GdHolder::from_path(base, "VBoxContainer/SearchBarLineEdit");
            self.searchbar.ok_mut()?.connect(
                "text_submitted".into(),
                base.callable("on_search_request"),
                0,
            );

            self.list_roll_view = GdHolder::from_path(base, "../../RollView");
            self.list_roll_view.ok_mut()?.bind_mut().connect(
                "dialog_closed".into(),
                base.callable("refresh_full"),
                0,
            );
            self.list_modify_view = GdHolder::from_path(base, "../../ListModifyView");
            self.list_modify_view.ok_mut()?.bind_mut().connect(
                "dialog_closed".into(),
                base.callable("refresh_full"),
                0,
            );

            // Get singleton and connect to global signals(show / hide)
            let mut signals = get_singleton::<Signals>("Signals");
            {
                let mut signals = signals.bind_mut();
                signals.connect(
                    "item_view_tab_selected".into(),
                    base.callable("hide"),
                    0,
                );
                signals.connect(
                    "list_view_tab_selected".into(),
                    base.callable("on_view_selected"),
                    0,
                );
                signals.connect(
                    "tag_view_tab_selected".into(),
                    base.callable("hide"),
                    0,
                );

                if self.is_visible() {
                    self.refresh_full();
                }
            }
        }: ArreResult<()> {
            Ok(_) => {},
            Err(e) => { log_error(e); }
        }
    }
    fn process(&mut self, _delta: f64) {
        match try {
            // Item cards LEFT click listener
            if let Some(observer) = &mut self.observer_card_left_click {
                if let Ok(card_id) = observer.try_recv() {
                    self.on_list_card_left_click(card_id)?;
                }
            }
            // Item cards RIGHT click listener
            if let Some(observer) = &mut self.observer_card_right_click {
                if let Ok(card_id) = observer.try_recv() {
                    self.on_list_card_right_click(card_id)?;
                }
            }
        }: ArreResult<()> {
            Ok(_) => {},
            Err(e) => { log_error(e); }
        }
    }
}