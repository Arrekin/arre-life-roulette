use godot::engine::{Control, ControlVirtual, Button, LineEdit};
use godot::prelude::*;
use crate::errors::{ArreResult};
use crate::godot_classes::containers::cards_flow_container::CardsFlowContainer;
use crate::godot_classes::singletons::globals::{Globals};
use crate::godot_classes::element_card::{Content, OnClickBehavior};
use crate::godot_classes::singletons::logger::log_error;
use crate::godot_classes::singletons::signals::Signals;
use crate::godot_classes::utils::{GdHolder, get_singleton};
use crate::godot_classes::views::view_list_modify::ListModifyView;
use crate::godot_classes::views::view_roll::RollView;
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
            let self_reference = self.base.share().cast::<Self>();
            self.cards_container.ok_mut()?.bind_mut().set_cards(
                self.lists.clone(),
                |mut card| {
                    card.on_left_click_behavior = Some(Box::new(OnClickBehaviorShowListRollView {
                        parent: self_reference.share(),
                    }));
                    card.on_right_click_behavior = Some(Box::new(OnClickBehaviorShowListModifyView {
                        parent: self_reference.share(),
                    }));
                }
            );
        }: ArreResult<()> {
            Ok(_) => {},
            Err(e) => { log_error(e); }
        }
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
}

struct OnClickBehaviorShowListModifyView {
    pub parent: Gd<ListsView>,
}

impl OnClickBehavior for OnClickBehaviorShowListModifyView {
    fn on_click(&mut self, content: &Content) {
        if let Content::List(list) = content {
            let mut parent = self.parent.bind_mut();
            parent.list_modify_view.ok_mut().map(|view| {
                let mut view = view.bind_mut();
                view.set_mode_edit(list.clone());
                view.show();
            }).unwrap_or_else(|e| log_error(e));
        }
    }
}

struct OnClickBehaviorShowListRollView {
    pub parent: Gd<ListsView>,
}

impl OnClickBehavior for OnClickBehaviorShowListRollView {
    fn on_click(&mut self, content: &Content) {
        if let Content::List(list) = content {
            let mut parent = self.parent.bind_mut();
            parent.list_roll_view.ok_mut().map(|view| {
                let mut view = view.bind_mut();
                view.set_list(list.clone()).unwrap_or_else(|e| log_error(e));
                view.refresh_view();
                view.show();
            }).unwrap_or_else(|e| log_error(e));
        }
    }
}