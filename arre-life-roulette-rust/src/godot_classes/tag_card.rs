use godot::engine::{MarginContainer, InputEvent, InputEventMouseButton, MarginContainerVirtual, Button, LineEdit, InputEventKey, StyleBoxFlat};
use godot::engine::global::{Key, MouseButton};
use godot::prelude::*;
use crate::db::DB;
use crate::errors::{ArreResult, BoxedError};
use crate::godot_classes::resources::TAG_LARGE_STYLE_BOX_FLAT;
use crate::godot_classes::singletons::logger::log_error;
use crate::godot_classes::utils::{GdHolder};
use crate::tag::{Tag, tag_delete, tag_persist, tag_update};

#[derive(GodotClass)]
#[class(base=MarginContainer)]
pub struct TagLargeCard {
    #[base]
    base: Base<MarginContainer>,

    // cached UI elements
    pub name_line_edit: GdHolder<LineEdit>,
    pub delete_button: GdHolder<Button>,

    // cached themes
    pub tag_large_style_box_flat: Gd<StyleBoxFlat>,

    // state
    pub tag: Tag,
}

#[godot_api]
impl TagLargeCard {
    #[func]
    fn refresh_display(&mut self) {
        match try {
            let line_edit = self.name_line_edit.ok_mut()?;
            line_edit.set_text(self.tag.name.clone().into());
            self.tag_large_style_box_flat.set_bg_color(Color::from_html(self.tag.color.clone()).unwrap());
        } {
            Ok(_) => {}
            Err::<_, BoxedError>(e) => log_error(e),
        }
    }

    pub fn set_tag(&mut self, tag: Tag) {
        self.tag = tag;
        self.refresh_display();
    }

    #[func]
    fn on_text_submitted(&mut self) {
        match try {
            self.name_line_edit.ok_mut()?.call_deferred("release_focus".into(), &[]);
        } {
            Ok(_) => {},
            Err::<_, BoxedError>(e) => log_error(e),
        }
    }

    #[func]
    fn on_focus_entered(&mut self) {
        match try {
            if self.tag.id.is_some() {
                self.delete_button.ok_mut()?.set_visible(true);
            }
        } {
            Ok(_) => {},
            Err::<_, BoxedError>(e) => log_error(e),
        }
    }

    #[func]
    fn on_focus_exited(&mut self) {
        match try {
            let new_name = self.name_line_edit.ok_mut()?.get_text().to_string();
            let connection = &*DB.ok()?;
            match self.tag.id {
                Some(_) => {
                    if new_name.is_empty() {
                        self.refresh_display();
                    } else {
                        self.tag.name = new_name;
                        tag_update(connection, &self.tag)?;
                    }
                },
                None => {
                    if new_name.is_empty() {
                        self.queue_free();
                    } else {
                        self.tag.name = new_name;
                        tag_persist(connection, &mut self.tag)?;
                    }
                }
            }
            self.delete_button.ok_mut()?.set_visible(false);
        } {
            Ok(_) => {},
            Err::<_, BoxedError>(e) => log_error(e),
        }
    }

    #[func]
    fn on_delete_button_up(&mut self) {
        match try {
            let connection = &*DB.ok()?;
            tag_delete(connection, self.tag.get_id()?)?;
            self.queue_free();
        } {
            Ok(_) => {},
            Err::<_, BoxedError>(e) => log_error(e),
        }
    }


    fn position_delete_button(&mut self) -> ArreResult<()> {
        // Tag Card references
        let size = self.get_size();
        let le_size = self.name_line_edit.ok()?.get_size();
        let global_pos =  self.get_global_position();
        let mut delete_button = self.delete_button.ok_mut()?;

        // Size
        delete_button.set_size(Vector2::new(le_size.y, le_size.y), false);

        // Position
        let shift_x = size.x;
        let shift_y = size.y / 2. - delete_button.get_size().y / 2.;
        let new_pos = global_pos + Vector2::new(shift_x, shift_y);
        delete_button.set_position(new_pos, false);
        Ok(())
    }
}

#[godot_api]
impl MarginContainerVirtual for TagLargeCard {
    fn init(base: Base<Self::Base>) -> Self {
        Self {
            base,

            // cached UI elements
            name_line_edit: GdHolder::default(),
            delete_button: GdHolder::default(),

            // cached themes
            tag_large_style_box_flat: load::<StyleBoxFlat>(TAG_LARGE_STYLE_BOX_FLAT)
                .duplicate(true).unwrap()
                .cast::<StyleBoxFlat>(),

            // state
            tag: Tag::default(),
        }
    }

    fn ready(&mut self) {
        match try {
            self.add_theme_constant_override("margin_left".into(), 5);
            self.add_theme_constant_override("margin_top".into(), 5);
            self.add_theme_constant_override("margin_right".into(), 5);
            self.add_theme_constant_override("margin_bottom".into(), 5);

            let base = &self.base;
            self.name_line_edit = GdHolder::from_path(base, "PanelContainer/LineEdit");
            {
                let line_edit = self.name_line_edit.ok_mut()?;
                line_edit.connect(
                    "text_submitted".into(),
                    base.callable("on_text_submitted"),
                    0,
                );
                line_edit.connect(
                    "focus_entered".into(),
                    base.callable("on_focus_entered"),
                    0,
                );
                line_edit.connect(
                    "focus_exited".into(),
                    base.callable("on_focus_exited"),
                    0,
                );
                line_edit.add_theme_stylebox_override("normal".into(), self.tag_large_style_box_flat.share().upcast());
            }
            self.delete_button = GdHolder::from_path(base, "TopLevel/DeleteButton");
            {
                let delete_button = self.delete_button.ok_mut()?;
                delete_button.set_visible(false);
                delete_button.connect(
                    "button_up".into(),
                    base.callable("on_delete_button_up"),
                    0,
                );
            }
        } {
            Ok(_) => {}
            Err::<_, BoxedError>(e) => log_error(e),
        }
    }

    fn process(&mut self, _delta: f64) {
        match try {
            if self.delete_button.ok_mut()?.is_visible() {
                self.position_delete_button()?;
            }
        } {
            Ok(_) => {}
            Err::<_, BoxedError>(e) => log_error(e),
        }
    }

    fn input(&mut self, event: Gd<InputEvent>) {
        match try {
            if !self.name_line_edit.ok()?.has_focus() { return; }
            let event_local = {
                let event_local = self.base.make_input_local(event);
                match event_local {
                    Some(event_local) => event_local,
                    None => return,
                }
            };
            if let Some(mouse_event) = event_local.share().try_cast::<InputEventMouseButton>() {
                if mouse_event.is_pressed() {
                    match mouse_event.get_button_index() {
                        MouseButton::MOUSE_BUTTON_RIGHT => {
                            let self_rect = Rect2::new(Vector2::new(0.0, 0.0), self.base.get_size());
                            if !self_rect.has_point(mouse_event.get_position()) {
                                self.name_line_edit.ok_mut()?.call_deferred("release_focus".into(), &[]);
                            }
                        },
                        _ => {}
                    }
                }
            } else if let Some(key_event) = event_local.try_cast::<InputEventKey>() {
                if !key_event.is_pressed() && key_event.get_keycode() == Key::KEY_ESCAPE {
                    self.refresh_display(); // Cancel any changes and reload data from the state
                    self.name_line_edit.ok_mut()?.call_deferred("release_focus".into(), &[]);
                }
            }
        } {
            Ok(_) => {},
            Err::<_, BoxedError>(e) => log_error(e),
        }
    }
}
