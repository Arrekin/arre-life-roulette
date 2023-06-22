use bus::BusReader;
use godot::engine::{Control, ControlVirtual, Button, LineEdit};
use godot::prelude::*;
use crate::db::DB;
use crate::errors::{ArreResult, BoxedError};
use crate::godot_classes::containers::cards_flow_container::CardsFlowContainer;
use crate::godot_classes::element_card::{Content, ElementCard};
use crate::godot_classes::resources::TAG_LARGE_PREFAB;
use crate::godot_classes::singletons::logger::log_error;
use crate::godot_classes::singletons::signals::Signals;
use crate::godot_classes::utils::{GdHolder, get_singleton};
use crate::godot_classes::views::view_item_modify::ItemModifyView;
use crate::godot_classes::views::view_item_stats::ItemStatsView;
use crate::item::{Item, item_get_all, item_search};
use crate::item_stats::item_stats_get;

#[derive(GodotClass)]
#[class(base=Control)]
pub struct TagsView {
    #[base]
    base: Base<Control>,

    // cached internal UI elements
    pub tags_container: GdHolder<Control>,

    // cached sub-scenes
    tag_large_prefab: Gd<PackedScene>,
}

#[godot_api]
impl TagsView {
    #[func]
    fn on_tag_add_button_up(&mut self) {
        // match try {
        //     self.item_modify_view.ok_mut().map(|view| {
        //         let mut view = view.bind_mut();
        //         view.set_mode_add();
        //         view.show();
        //     })?;
        // } {
        //     Ok(_) => {},
        //     Err::<_, BoxedError>(e) => log_error(e)
        // }
    }

    #[func]
    fn on_view_selected(&mut self) {
        self.refresh_display();
        self.show();
    }

    #[func]
    fn refresh_display(&mut self) {
        match try {
            //self.cards_container.ok_mut()?.bind_mut().set_cards(self.items.clone());
        } {
            Ok(_) => {},
            Err::<_, BoxedError>(e) => log_error(e)
        }
    }
}

#[godot_api]
impl ControlVirtual for TagsView {
    fn init(base: Base<Self::Base>) -> Self {
        Self {
            base,

            // cached internal UI elements
            tags_container: GdHolder::default(),

            // cached sub-scenes
            tag_large_prefab: load(TAG_LARGE_PREFAB),
        }
    }
    fn ready(&mut self) {
        match try {
            let base = &self.base;
            self.tags_container = GdHolder::from_path(base, "VBoxContainer/MarginContainer/ScrollContainer/HFlowContainer");

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
                    base.callable("hide"),
                    0,
                );
                signals.connect(
                    "tag_view_tab_selected".into(),
                    base.callable("on_view_selected"),
                    0,
                );
            }
        } {
            Ok(_) => {}
            Err::<_, BoxedError>(e) => log_error(e),
        }
    }
}