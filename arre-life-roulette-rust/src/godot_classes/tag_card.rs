use godot::engine::{MarginContainer, InputEvent, InputEventMouseButton, MarginContainerVirtual, LineEdit, InputEventKey, StyleBoxFlat, ColorPicker, DisplayServer};
use godot::engine::global::{Key, MouseButton};
use godot::prelude::*;
use crate::db::DB;
use crate::errors::{ArreError, ArreResult, BoxedError};
use crate::godot_classes::resources::{TAG_ACCEPT_CHANGES_ICON, TAG_BG_COLOR_ICON, TAG_DELETE_ICON, TAG_LARGE_STYLE_BOX_FLAT, TAG_REJECT_CHANGES_ICON};
use crate::godot_classes::singletons::logger::log_error;
use crate::godot_classes::sliding_button::{SlidingButton, SlidingInDirection};
use crate::godot_classes::utils::{GdHolder};
use crate::tag::{Tag, tag_delete, tag_persist, tag_update};

#[derive(GodotClass)]
#[class(base=MarginContainer)]
pub struct TagLargeCard {
    #[base]
    base: Base<MarginContainer>,

    // cached UI elements
    pub name_line_edit: GdHolder<LineEdit>,
    pub delete_sliding_button: GdHolder<SlidingButton>,
    pub bg_color_sliding_button: GdHolder<SlidingButton>,
    pub reject_sliding_button: GdHolder<SlidingButton>,
    pub accept_sliding_button: GdHolder<SlidingButton>,
    pub color_picker: GdHolder<ColorPicker>,

    // cached themes
    pub tag_large_style_box_flat: Gd<StyleBoxFlat>,

    // state
    pub tag: Tag,
}

#[godot_api]
impl TagLargeCard {

    pub fn set_tag(&mut self, tag: Tag) {
        self.tag = tag;
        self.refresh_display();
    }

    pub fn grab_focus(&mut self) -> ArreResult<()> {
        self.name_line_edit.ok_mut()?.call_deferred("grab_focus".into(), &[]); Ok(())
    }

    pub fn release_focus(&mut self) -> ArreResult<()> {
        self.name_line_edit.ok_mut()?.call_deferred("release_focus".into(), &[]); Ok(())
    }

    pub fn select_all(&mut self) -> ArreResult<()> {
        self.name_line_edit.ok_mut()?.call_deferred("select_all".into(), &[]); Ok(())
    }

    #[func]
    fn refresh_display(&mut self) {
        match try {
            let line_edit = self.name_line_edit.ok_mut()?;
            line_edit.set_text(self.tag.name.clone().into());
            self.tag_large_style_box_flat.set_bg_color(
                Color::from_html(self.tag.color.clone())
                    .ok_or(ArreError::UnexpectedNone("TagLargeCard::refresh_display[Color::from_html]".into()))?
            );
        } {
            Ok(_) => {}
            Err::<_, BoxedError>(e) => log_error(e),
        }
    }

    #[func]
    fn on_text_submitted(&mut self) {
        match try {
            self.release_focus()?;
        } {
            Ok(_) => {},
            Err::<_, BoxedError>(e) => log_error(e),
        }
    }

    #[func]
    fn on_focus_entered(&mut self) {
        match try {
            if self.tag.id.is_some() {
                self.delete_sliding_button.ok_mut()?.bind_mut().slide_in()?;
                self.reject_sliding_button.ok_mut()?.bind_mut().slide_in()?;
            }
            self.accept_sliding_button.ok_mut()?.bind_mut().slide_in()?;
            self.bg_color_sliding_button.ok_mut()?.bind_mut().slide_in()?;
        } {
            Ok(_) => {},
            Err::<_, BoxedError>(e) => log_error(e),
        }
    }

    #[func]
    fn on_focus_exited(&mut self) {
        match try {
            let new_name = self.name_line_edit.ok_mut()?.get_text().to_string();
            let new_bg_color = self.tag_large_style_box_flat.get_bg_color().to_html();
            let connection = &*DB.ok()?;
            match self.tag.id {
                Some(_) => {
                    if new_name.is_empty() {
                        self.refresh_display();
                    } else {
                        self.tag.name = new_name;
                        self.tag.color = new_bg_color.into();
                        tag_update(connection, &self.tag)?;
                    }
                },
                None => {
                    if new_name.is_empty() {
                        self.queue_free();
                    } else {
                        self.tag.name = new_name;
                        self.tag.color = new_bg_color.into();
                        tag_persist(connection, &mut self.tag)?;
                    }
                }
            }
            self.delete_sliding_button.ok_mut()?.bind_mut().slide_out()?;
            self.bg_color_sliding_button.ok_mut()?.bind_mut().slide_out()?;
            self.reject_sliding_button.ok_mut()?.bind_mut().slide_out()?;
            self.accept_sliding_button.ok_mut()?.bind_mut().slide_out()?;
            self.color_picker.ok_mut()?.hide();
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

    #[func]
    fn on_bg_color_button_up(&mut self) {
        match try {
            let is_visible = self.color_picker.ok_mut()?.is_visible();
            self.color_picker.ok_mut()?.set_visible(!is_visible);

        } {
            Ok(_) => {},
            Err::<_, BoxedError>(e) => log_error(e),
        }
    }

    #[func]
    fn reject_changes(&mut self) {
        match try {
            self.refresh_display();
            self.release_focus()?;
        } {
            Ok(_) => {},
            Err::<_, BoxedError>(e) => log_error(e),
        }
    }

    #[func]
    fn on_accept_button_up(&mut self) {
        match try {
            self.release_focus()?;
        } {
            Ok(_) => {},
            Err::<_, BoxedError>(e) => log_error(e),
        }
    }

    #[func]
    fn on_color_changed(&mut self, color: Color) {
        match try {
            self.tag_large_style_box_flat.set_bg_color(color);
        } {
            Ok(_) => {},
            Err::<_, BoxedError>(e) => log_error(e),
        }
    }

    fn position_buttons(&mut self) -> ArreResult<()> {
        self.position_delete_button()?;
        self.position_bg_color_button()?;
        self.position_reject_button()?;
        self.position_accept_button()?;
        self.position_color_picker()?;
        Ok(())
    }

    fn position_delete_button(&mut self) -> ArreResult<()> {
        // Tag Card references
        let size = self.get_size();
        let btn_size = self.name_line_edit.ok()?.get_size().y;
        let global_pos =  self.get_global_position();
        let mut delete_button = self.delete_sliding_button.ok_mut()?.bind_mut();

        // Size
        delete_button.set_size(Vector2::new(btn_size, btn_size))?;

        // Position - to the right
        let shift_x = size.x;
        let shift_y = size.y / 2. - delete_button.get_size().y / 2.;
        let new_pos = global_pos + Vector2::new(shift_x, shift_y);
        delete_button.set_position(new_pos);
        Ok(())
    }

    fn position_bg_color_button(&mut self) -> ArreResult<()> {
        // Tag Card references
        let size = self.get_size();
        let btn_size = self.name_line_edit.ok()?.get_size().y;
        let global_pos =  self.get_global_position();
        let mut bg_color_button = self.bg_color_sliding_button.ok_mut()?.bind_mut();

        // Size
        bg_color_button.set_size(Vector2::new(btn_size, btn_size))?;

        // Position - to the right, below
        let shift_x = size.x - btn_size;
        let shift_y = size.y;
        let new_pos = global_pos + Vector2::new(shift_x, shift_y);
        bg_color_button.set_position(new_pos);
        Ok(())
    }

    fn position_reject_button(&mut self) -> ArreResult<()> {
        // Tag Card references
        let btn_size = self.name_line_edit.ok()?.get_size().y;
        let global_pos =  self.get_global_position();
        let mut reject_button = self.reject_sliding_button.ok_mut()?.bind_mut();

        // Size
        reject_button.set_size(Vector2::new(btn_size, btn_size))?;

        // Position - to the left, above
        let shift_x = 0.;
        let shift_y = -btn_size;
        let new_pos = global_pos + Vector2::new(shift_x, shift_y);
        reject_button.set_position(new_pos);
        Ok(())
    }

    fn position_accept_button(&mut self) -> ArreResult<()> {
        // Tag Card references
        let size = self.get_size();
        let btn_size = self.name_line_edit.ok()?.get_size().y;
        let global_pos =  self.get_global_position();
        let mut accept_button = self.accept_sliding_button.ok_mut()?.bind_mut();

        // Size
        accept_button.set_size(Vector2::new(btn_size, btn_size))?;

        // Position - to the left, below
        let shift_x = 0.;
        let shift_y = size.y;
        let new_pos = global_pos + Vector2::new(shift_x, shift_y);
        accept_button.set_position(new_pos);
        Ok(())
    }

    fn position_color_picker(&mut self) -> ArreResult<()> {
        // Tag Card references
        let size = self.get_size();
        let global_pos =  self.get_global_position();
        // Background color button reference
        let bg_button = self.bg_color_sliding_button.ok()?.bind();
        let bg_button_size = bg_button.get_size();
        let bg_button_pos = bg_button.get_position();

        // Color picker reference
        let cp = self.color_picker.ok_mut()?;
        let cp_size = cp.get_size();

        // Window reference
        let window_size = DisplayServer::singleton().window_get_size();

        // Position depending on free space
        let right_margin = window_size.x as f32 - (bg_button_pos.x + bg_button_size.x);
        let bottom_margin = window_size.y as f32 - (bg_button_pos.y + bg_button_size.y);
        if right_margin > cp_size.x && bottom_margin > cp_size.y {
            // If bottom right has enough space, that is the preferred location
            cp.set_position(Vector2::new(bg_button_pos.x + bg_button_size.x, bg_button_pos.y + bg_button_size.y));
        } else if bottom_margin > cp_size.y {
            // Otherwise, if at least bottom has enough space, place it bottom left
            cp.set_position(Vector2::new(bg_button_pos.x - cp_size.x, bg_button_pos.y + bg_button_size.y));
        } else if right_margin > cp_size.x {
            // Otherwise, If at least right has enough space, place it top right
            cp.set_position(Vector2::new(global_pos.x + size.x, global_pos.y - cp_size.y));
        } else {
            // Otherwise, top left should have enough space
            cp.set_position(Vector2::new(global_pos.x - cp_size.x, global_pos.y - cp_size.y));
        }
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
            delete_sliding_button: GdHolder::default(),
            bg_color_sliding_button: GdHolder::default(),
            reject_sliding_button: GdHolder::default(),
            accept_sliding_button: GdHolder::default(),
            color_picker: GdHolder::default(),

            // cached themes
            tag_large_style_box_flat: load::<StyleBoxFlat>(TAG_LARGE_STYLE_BOX_FLAT)
                .duplicate().unwrap()
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
                );
                line_edit.connect(
                    "focus_entered".into(),
                    base.callable("on_focus_entered"),
                );
                line_edit.connect(
                    "focus_exited".into(),
                    base.callable("on_focus_exited"),
                );
                line_edit.add_theme_stylebox_override("normal".into(), self.tag_large_style_box_flat.share().upcast());
            }
            self.delete_sliding_button = GdHolder::from_path(base, "TopLevel/DeleteSlidingButton");
            {
                let mut delete_button = self.delete_sliding_button.ok_mut()?.bind_mut();
                delete_button.set_sliding_direction(SlidingInDirection::Right)?;
                let texture = try_load(TAG_DELETE_ICON).ok_or(ArreError::UnexpectedNone("TagLargeCard::ready[DeleteSlidingButton]".into()))?;
                delete_button.set_texture(texture)?;
                let actual_button = delete_button.button.ok_mut()?;
                actual_button.set_tooltip_text("Delete Tag".into());
                actual_button.connect(
                    "button_up".into(),
                    base.callable("on_delete_button_up"),
                );
            }
            self.bg_color_sliding_button = GdHolder::from_path(base, "TopLevel/BackgroundColorSlidingButton");
            {
                let mut bg_color_button = self.bg_color_sliding_button.ok_mut()?.bind_mut();
                bg_color_button.set_sliding_direction(SlidingInDirection::Bottom)?;
                let texture = try_load(TAG_BG_COLOR_ICON).ok_or(ArreError::UnexpectedNone("TagLargeCard::ready[DeleteSlidingButton]".into()))?;
                bg_color_button.set_texture(texture)?;
                let actual_button = bg_color_button.button.ok_mut()?;
                actual_button.set_tooltip_text("Pick Background Color".into());
                actual_button.connect(
                    "button_up".into(),
                    base.callable("on_bg_color_button_up"),
                );
            }
            self.reject_sliding_button = GdHolder::from_path(base, "TopLevel/RejectSlidingButton");
            {
                let mut reject_button = self.reject_sliding_button.ok_mut()?.bind_mut();
                reject_button.set_sliding_direction(SlidingInDirection::Up)?;
                let texture = try_load(TAG_REJECT_CHANGES_ICON).ok_or(ArreError::UnexpectedNone("TagLargeCard::ready[DeleteSlidingButton]".into()))?;
                reject_button.set_texture(texture)?;
                let actual_button = reject_button.button.ok_mut()?;
                actual_button.set_tooltip_text("Reject Changes (ESC)".into());
                actual_button.connect(
                    "button_up".into(),
                    base.callable("reject_changes"),
                );
            }
            self.accept_sliding_button = GdHolder::from_path(base, "TopLevel/AcceptSlidingButton");
            {
                let mut accept_button = self.accept_sliding_button.ok_mut()?.bind_mut();
                accept_button.set_sliding_direction(SlidingInDirection::Bottom)?;
                let texture = try_load(TAG_ACCEPT_CHANGES_ICON).ok_or(ArreError::UnexpectedNone("TagLargeCard::ready[DeleteSlidingButton]".into()))?;
                accept_button.set_texture(texture)?;
                let actual_button = accept_button.button.ok_mut()?;
                actual_button.set_tooltip_text("Accept Changes (ENTER)".into());
                actual_button.connect(
                    "button_up".into(),
                    base.callable("on_accept_button_up"),
                );
            }
            self.color_picker = GdHolder::from_path(base, "TopLevel/ColorPicker");
            {
                let color_picker = self.color_picker.ok_mut()?;
                color_picker.connect(
                    "color_changed".into(),
                    base.callable("on_color_changed"),
                )
            }
        } {
            Ok(_) => {}
            Err::<_, BoxedError>(e) => log_error(e),
        }
    }

    fn process(&mut self, _delta: f64) {
        match try {
            self.position_buttons()?;
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
                                self.release_focus()?;
                            }
                        },
                        _ => {}
                    }
                }
            } else if let Some(key_event) = event_local.try_cast::<InputEventKey>() {
                if !key_event.is_pressed() && key_event.get_keycode() == Key::KEY_ESCAPE {
                    self.refresh_display(); // Cancel any changes and reload data from the state
                    self.release_focus()?;
                }
            }
        } {
            Ok(_) => {},
            Err::<_, BoxedError>(e) => log_error(e),
        }
    }
}
