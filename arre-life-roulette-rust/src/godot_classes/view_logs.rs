use godot::engine::{Control, ControlVirtual, VBoxContainer, RichTextLabel, InputEvent};
use godot::engine::node::InternalMode;
use godot::engine::packed_scene::GenEditState;
use godot::prelude::*;
use crate::errors::ArreError;
use crate::godot_classes::resources::{LOG_ENTRY_PREFAB, LOGS_VIEW_TOGGLE, SELECTION_BUTTON_PREFAB};
use crate::godot_classes::singletons::logger::Logger;
use crate::godot_classes::utils::get_singleton;

#[derive(GodotClass)]
#[class(base=Control)]
pub struct LogsView {
    #[base]
    base: Base<Control>,

    // cached sub-scenes
    log_entry_prefab: Gd<PackedScene>,

    // cached UI elements
    pub logs_vboxcontainer: Option<Gd<VBoxContainer>>
}

#[godot_api]
impl LogsView {

    #[func]
    fn refresh_display(&mut self) {
        let mut logger = get_singleton::<Logger>("Logger");
        let mut logger = logger.bind_mut();
        self.logs_vboxcontainer.as_mut().map(|vbox| {
            // First delete all existing UI log entries
            for mut child in vbox.get_children(false).iter_shared() {
                child.queue_free();
            }
            // Then add new ones
            for log in logger.logs.clone() {
                let log_entry = self.log_entry_prefab.instantiate(GenEditState::GEN_EDIT_STATE_DISABLED);
                if let Some(log_node) = log_entry {
                    let rich_text_label = log_node.try_get_node_as::<RichTextLabel>("PanelContainer/MarginContainer/RichTextLabel");
                    if let Some(mut log_label) = rich_text_label {
                        log_label.set_text(log.clone());
                        vbox.add_child(log_node, false, InternalMode::INTERNAL_MODE_DISABLED);
                    } else { logger.error(ArreError::NullGd("LogsView::refresh_display::<RichTextLabel>".into())); continue; }
                } else { logger.error(ArreError::InstantiateFailed(SELECTION_BUTTON_PREFAB.into(), "LogsView::refresh_display".into())); continue; }
            }
        });
    }
}

#[godot_api]
impl ControlVirtual for LogsView {
    fn init(base: Base<Self::Base>) -> Self {
        Self {
            base,
            log_entry_prefab: load(LOG_ENTRY_PREFAB),

            logs_vboxcontainer: None
        }
    }
    fn ready(&mut self) {
        self.logs_vboxcontainer = self.base.try_get_node_as("ScrollContainer/VBoxContainer");
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
