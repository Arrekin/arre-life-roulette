use godot::engine::{MarginContainer, InputEvent, InputEventMouseButton, MarginContainerVirtual, Label, Button, LineEdit};
use godot::engine::global::MouseButton;
use godot::prelude::*;
use crate::errors::{BoxedError};
use crate::godot_classes::singletons::buses::{BusType};
use crate::godot_classes::singletons::logger::log_error;
use crate::godot_classes::utils::{GdHolder};
use crate::item::Item;
use crate::list::List;
use crate::tag::Tag;

#[derive(GodotClass)]
#[class(base=MarginContainer)]
pub struct TagLargeCard {
    #[base]
    base: Base<MarginContainer>,

    // cached UI elements
    pub button: GdHolder<Button>,
    pub name_line_edit: GdHolder<LineEdit>,

    // state
    pub tag: Tag,
    pub is_being_edited: bool,
}

#[godot_api]
impl TagLargeCard {
    #[func]
    fn refresh_display(&mut self) {
        match try {
            self.name_line_edit.ok_mut()?.set_text(self.tag.name.clone().into());
        } {
            Ok(_) => {}
            Err::<_, BoxedError>(e) => log_error(e),
        }
    }

    pub fn set_tag(&mut self, tag: Tag) {
        self.tag = tag;
        self.refresh_display();
    }
}

#[godot_api]
impl MarginContainerVirtual for TagLargeCard {
    fn init(base: Base<Self::Base>) -> Self {
        Self {
            base,

            // cached UI elements
            button: GdHolder::default(),
            name_line_edit: GdHolder::default(),

            // state
            tag: Tag::default(),
            is_being_edited: false,
        }
    }

    fn ready(&mut self) {
        match try {
            self.add_theme_constant_override("margin_left".into(), 5);
            self.add_theme_constant_override("margin_top".into(), 5);
            self.add_theme_constant_override("margin_right".into(), 5);
            self.add_theme_constant_override("margin_bottom".into(), 5);

            let base = &self.base;
            self.button = GdHolder::from_path(base, "PanelContainer/Button");
            self.button.ok_mut()?.connect(
                "gui_input".into(),
                base.callable("on_gui_input"),
                0,
            );
            self.name_line_edit = GdHolder::from_path(base, "PanelContainer/LineEdit");
        } {
            Ok(_) => {}
            Err::<_, BoxedError>(e) => log_error(e),
        }
    }

    fn input(&mut self, event: Gd<InputEvent>) {
        match try {
            let event_local = {
                let event_local = self.base.make_input_local(event);
                match event_local {
                    Some(event_local) => event_local,
                    None => return,
                }
            };
            if let Some(mouse_event) = event_local.try_cast::<InputEventMouseButton>() {
                if mouse_event.is_pressed() {
                    match mouse_event.get_button_index() {
                        MouseButton::MOUSE_BUTTON_LEFT => {
                            let self_rect = Rect2::new(Vector2::new(0.0, 0.0), self.base.get_size());
                            if !self_rect.has_point(mouse_event.get_position()) {
                                self.name_line_edit.ok_mut()?.release_focus();
                            }
                        },
                        _ => {}
                    }
                }
            }
        } {
            Ok(_) => {},
            Err::<_, BoxedError>(e) => log_error(e),
        }
    }
}
