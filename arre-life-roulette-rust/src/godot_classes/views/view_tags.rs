use godot::engine::{Control, ControlVirtual, Button};
use godot::engine::node::InternalMode;
use godot::prelude::*;
use crate::db::DB;
use crate::errors::{ArreError, ArreResult, BoxedError};
use crate::godot_classes::resources::TAG_LARGE_PREFAB;
use crate::godot_classes::singletons::logger::log_error;
use crate::godot_classes::singletons::signals::Signals;
use crate::godot_classes::tag_card::TagLargeCard;
use crate::godot_classes::utils::{GdHolder, get_singleton};
use crate::tag::{Tag, tag_get_all};

#[derive(GodotClass)]
#[class(base=Control)]
pub struct TagsView {
    #[base]
    base: Base<Control>,

    // cached internal UI elements
    pub tags_container: GdHolder<Control>,
    pub tag_add_button: GdHolder<Button>,

    // cached sub-scenes
    tag_large_prefab: Gd<PackedScene>,
}

#[godot_api]
impl TagsView {
    #[func]
    fn on_tag_add_button_up(&mut self) {
        match try {
            let mut new_tag = Tag::default();
            new_tag.name = "New Tag".to_string();

            let mut card = self.add_card(new_tag)?;
            let mut card = card.bind_mut();

            // Card is called back when it's line_edit focus changes, so we have to deffer it to avoid re-borrow
            let line_edit = card.name_line_edit.ok_mut()?;
            line_edit.call_deferred("grab_focus".into(), &[]);
            line_edit.call_deferred("select_all".into(), &[]);

        } {
            Ok(_) => {},
            Err::<_, BoxedError>(e) => log_error(e)
        }
    }

    #[func]
    fn on_view_selected(&mut self) {
        self.refresh_display();
        self.show();
    }

    #[func]
    fn refresh_display(&mut self) {
        match try {
            // Remove existing tag cards
            self.tags_container.ok_mut()?
                .get_children(false)
                .iter_shared()
                .for_each(|mut child_card| child_card.queue_free());

            let connection = &*DB.ok()?;
            for tag in tag_get_all::<Vec<_>>(connection)? {
                self.add_card(tag)?;
            }
        } {
            Ok(_) => {},
            Err::<_, BoxedError>(e) => log_error(e)
        }
    }

    pub fn add_card(&mut self, tag: Tag) -> ArreResult<Gd<TagLargeCard>>{
        let mut card = self.tag_large_prefab
            .try_instantiate_as::<TagLargeCard>()
            .ok_or(ArreError::InstantiateFailed(
                TAG_LARGE_PREFAB.into(),
                "TagsView::refresh_display".into()
            ))?;
        self.tags_container.ok_mut()?.add_child(card.share().upcast(), false, InternalMode::INTERNAL_MODE_DISABLED);
        {
            let mut card = card.bind_mut();
            card.set_tag(tag);
        }
        Ok(card)
    }
}

#[godot_api]
impl ControlVirtual for TagsView {
    fn init(base: Base<Self::Base>) -> Self {
        Self {
            base,

            // cached internal UI elements
            tags_container: GdHolder::default(),
            tag_add_button: GdHolder::default(),

            // cached sub-scenes
            tag_large_prefab: load(TAG_LARGE_PREFAB),
        }
    }
    fn ready(&mut self) {
        match try {
            let base = &self.base;
            self.tags_container = GdHolder::from_path(base, "VBoxContainer/CentralMarginContainer/ScrollContainer/HFlowContainer");
            self.tag_add_button = GdHolder::from_path(base, "VBoxContainer/BottomMarginContainer/TagAddButton");
            self.tag_add_button.ok_mut()?.connect(
                "button_up".into(),
                base.callable("on_tag_add_button_up"),
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
                    base.callable("hide"),
                    0,
                );
                signals.connect(
                    "tag_view_tab_selected".into(),
                    base.callable("on_view_selected"),
                    0,
                );
            }
            if self.is_visible() {
                self.refresh_display();
            }
        } {
            Ok(_) => {}
            Err::<_, BoxedError>(e) => log_error(e),
        }
    }
}