use godot::builtin::{Callable};
use godot::engine::{Control, ControlVirtual, Button, HFlowContainer, LineEdit};
use godot::engine::node::InternalMode;
use godot::engine::packed_scene::GenEditState;
use godot::prelude::*;
use crate::errors::{ArreError, ArreResult};
use crate::godot_classes::element_card::{ElementCard, OnClickBehavior, Content};
use crate::godot_classes::singletons::globals::{Globals};
use crate::godot_classes::resources::{ELEMENT_CARD_PREFAB};
use crate::godot_classes::singletons::logger::log_error;
use crate::godot_classes::singletons::signals::Signals;
use crate::godot_classes::utils::{GdHolder, get_singleton};
use crate::godot_classes::view_item_modify::ItemModifyView;
use crate::godot_classes::view_item_stats::ItemStatsView;
use crate::item::{Item, item_get_all, item_search};
use crate::item_stats::item_stats_get;

#[derive(GodotClass)]
#[class(base=Control)]
pub struct ItemsView {
    #[base]
    base: Base<Control>,

    // cached sub-scenes
    element_card_prefab: Gd<PackedScene>,

    // cached UI elements
    pub item_add_button: GdHolder<Button>,
    pub item_modify_view: GdHolder<ItemModifyView>,
    pub item_stats_view: GdHolder<ItemStatsView>,
    pub items_grid: GdHolder<HFlowContainer>,
    pub items_grid_elements: Vec<Gd<ElementCard>>,
    pub searchbar: GdHolder<LineEdit>,

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
        self.refresh_display()
    }

    #[func]
    fn refresh_state(&mut self) {
        match try {
            let globals = get_singleton::<Globals>("Globals");
            let connection = &globals.bind().connection;
            match &self.search_term {
                Some(search_term) => {
                    self.items = item_search(connection, search_term)?;
                },
                None => { // Get list of all items from the DB
                    self.items = item_get_all(connection)?;
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
            // Clear old cards and then create a new card for each item
            self.items_grid_elements.drain(..).for_each(|mut item_btn| item_btn.bind_mut().queue_free());
            let new_items = self.items.iter().map(
                |item| {
                    let instance = self.element_card_prefab
                        .instantiate(GenEditState::GEN_EDIT_STATE_DISABLED)
                        .ok_or(ArreError::InstantiateFailed(
                            ELEMENT_CARD_PREFAB.into(),
                            "ItemsView::refresh_items_list".into()
                        ))?;
                    self.items_grid.ok_mut()?.add_child(instance.share(), false, InternalMode::INTERNAL_MODE_DISABLED);
                    let mut button = instance.try_cast::<ElementCard>()
                        .ok_or(ArreError::CastFailed("ElementCard".into(), "ItemsView::refresh_items_list".into()))?;
                    {
                        let mut button = button.bind_mut();
                        button.set_item(item.clone());
                        button.on_left_click_behavior = Some(Box::new(OnClickBehaviorShowItemStatsView {
                            parent: self_reference.share(),
                        }));
                        button.on_right_click_behavior = Some(Box::new(OnClickBehaviorShowItemModifyView {
                            parent: self_reference.share(),
                        }));
                    }
                    Ok(button)
                }
            ).collect::<ArreResult<Vec<_>>>()?;
            self.items_grid_elements.extend(new_items);
        }: ArreResult<()> {
            Ok(_) => {},
            Err(e) => { log_error(e); }
        }
    }
}

#[godot_api]
impl ControlVirtual for ItemsView {
    fn init(base: Base<Self::Base>) -> Self {
        Self {
            base,
            element_card_prefab: load(ELEMENT_CARD_PREFAB),

            item_add_button: GdHolder::default(),
            item_modify_view: GdHolder::default(),
            item_stats_view: GdHolder::default(),
            items_grid: GdHolder::default(),
            items_grid_elements: vec![],
            searchbar: GdHolder::default(),

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
                Callable::from_object_method(base.share(), "on_item_add_button_up"),
                0,
            );
            self.item_modify_view = GdHolder::from_path(base, "../../ItemModifyView");
            self.item_modify_view.ok_mut()?.bind_mut().connect(
                "dialog_closed".into(),
                Callable::from_object_method(base.share(), "refresh_full"),
                0,
            );
            self.item_stats_view = GdHolder::from_path(base, "../../ItemStatsView");
            self.items_grid = GdHolder::from_path(base,"VBoxContainer/ItemsListScrollContainer/ItemsListHFlowContainer");
            self.searchbar = GdHolder::from_path(base,"VBoxContainer/SearchBarLineEdit");
            self.searchbar.ok_mut()?.connect(
                "text_submitted".into(),
                Callable::from_object_method(base.share(), "on_search_request"),
                0,
            );


            // Get singleton and connect to global signals(show / hide)
            let mut signals = get_singleton::<Signals>("Signals");
            {
                let mut signals = signals.bind_mut();
                signals.connect(
                    "item_view_tab_selected".into(),
                    Callable::from_object_method(base.share(), "on_view_selected"),
                    0,
                );
                signals.connect(
                    "list_view_tab_selected".into(),
                    Callable::from_object_method(base.share(), "hide"),
                    0,
                );
                signals.connect(
                    "tag_view_tab_selected".into(),
                    Callable::from_object_method(base.share(), "hide"),
                    0,
                );
            }

            if self.is_visible() {
                self.refresh_full();
            }
        }: ArreResult<()> {
            Ok(_) => {}
            Err(e) => log_error(e),
        }
    }
}

struct OnClickBehaviorShowItemModifyView {
    pub parent: Gd<ItemsView>,
}

impl OnClickBehavior for OnClickBehaviorShowItemModifyView {
    fn on_click(&mut self, content: &Content) {
        if let Content::Item(item) = content {
            let mut parent = self.parent.bind_mut();
            parent.item_modify_view.ok_mut().map(|view| {
                let mut view = view.bind_mut();
                view.set_mode_edit(item.clone());
                view.show();
            }).unwrap_or_else(|e| log_error(e));
        }
    }
}

struct OnClickBehaviorShowItemStatsView {
    pub parent: Gd<ItemsView>,
}

impl OnClickBehavior for OnClickBehaviorShowItemStatsView {
    fn on_click(&mut self, content: &Content) {
        match try {
            if let Content::Item(item) = content {
                let globals = get_singleton::<Globals>("Globals");
                let connection = &globals.bind().connection;

                let mut parent = self.parent.bind_mut();
                let mut view = parent.item_stats_view.ok_mut()?.bind_mut();
                view.item_stats = item_stats_get(connection, item.get_id()?)?;
                view.refresh_display();
                view.show();
            }
        }: ArreResult<()> {
            Ok(_) => {}
            Err(e) => log_error(e),
        }
    }
}