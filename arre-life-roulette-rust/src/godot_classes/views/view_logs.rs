use godot::engine::{Control, ControlVirtual, VBoxContainer, RichTextLabel, InputEvent};
use godot::engine::node::InternalMode;
use godot::engine::packed_scene::GenEditState;
use godot::prelude::*;
use crate::errors::{ArreError, BoxedError};
use crate::godot_classes::resources::{LOG_ENTRY_PREFAB, LOGS_VIEW_TOGGLE, SELECTION_BUTTON_PREFAB};
use crate::godot_classes::singletons::logger::{log_error, Logger};
use crate::godot_classes::utils::{GdHolder, get_singleton};

#[derive(GodotClass)]
#[class(base=Control)]
pub struct LogsView {
    #[base]
    base: Base<Control>,

    // cached sub-scenes
    log_entry_prefab: Gd<PackedScene>,

    // cached UI elements
    pub logs_vboxcontainer: GdHolder<VBoxContainer>,
}

#[godot_api]
impl LogsView {

    #[func]
    fn refresh_display(&mut self) {
        match try {
            let logger = get_singleton::<Logger>("Logger");
            let logger = logger.bind();
            let vbox = self.logs_vboxcontainer.ok_mut()?;

            // First delete all existing UI log entries
            for mut child in vbox.get_children(false).iter_shared() {
                child.queue_free();
            }
            // Then add new ones
            for log in logger.logs.clone() {
                let log_node = self.log_entry_prefab
                    .instantiate(GenEditState::GEN_EDIT_STATE_DISABLED)
                    .ok_or(ArreError::InstantiateFailed(SELECTION_BUTTON_PREFAB.into(), "LogsView::refresh_display".into()))?;

                    let rich_text_label = log_node.try_get_node_as::<RichTextLabel>("PanelContainer/MarginContainer/RichTextLabel");
                    rich_text_label.map(|mut log_label| {
                        log_label.set_text(log.clone());
                        vbox.add_child(log_node, false, InternalMode::INTERNAL_MODE_DISABLED);
                    }).ok_or(ArreError::NullGd("LogsView::refresh_display::<RichTextLabel>".into()))?;
            }
        } {
            Ok(_) => {},
            Err::<_, BoxedError>(e) => log_error(e)
        }
    }
}

#[godot_api]
impl ControlVirtual for LogsView {
    fn init(base: Base<Self::Base>) -> Self {
        Self {
            base,
            log_entry_prefab: load(LOG_ENTRY_PREFAB),

            logs_vboxcontainer: GdHolder::default(),
        }
    }
    fn ready(&mut self) {
        match try {
            let base = &self.base;
            self.logs_vboxcontainer = GdHolder::from_path(base, "ScrollContainer/VBoxContainer");
        } {
            Ok(_) => {},
            Err::<_, BoxedError>(e) => log_error(e)
        }
    }

    fn unhandled_key_input(&mut self, event: Gd<InputEvent>) {
        if event.is_action_released(LOGS_VIEW_TOGGLE.into(), true) {
            let current_visible = self.is_visible();
            if !current_visible {
                self.refresh_display();
            }
            self.set_visible(!current_visible);
        }
    }
}
