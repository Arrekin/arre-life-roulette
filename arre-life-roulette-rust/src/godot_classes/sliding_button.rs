use godot::engine::{Button, Control, ControlVirtual, Tween};
use godot::prelude::*;
use crate::errors::{ArreError, ArreResult, BoxedError};
use crate::godot_classes::singletons::logger::log_error;
use crate::godot_classes::utils::GdHolder;

pub enum SlidingInDirection {
    Right,
    Bottom,
}

#[derive(GodotClass)]
#[class(base=Control)]
pub struct SlidingButton {
    #[base]
    base: Base<Control>,

    // cached UI elements
    pub button: GdHolder<Button>,

    // state
    pub direction: SlidingInDirection,
    sliding_tween: GdHolder<Tween>,
}

#[godot_api]
impl SlidingButton {

    pub fn set_size(&mut self, size: Vector2) -> ArreResult<()> {
        self.base.set_size(size);
        self.button.ok_mut()?.set_size(size);
        Ok(())
    }

    pub fn set_sliding_direction(&mut self, direction: SlidingInDirection) -> ArreResult<()> {
        self.direction = direction;
        let hidden_position = self.get_hiding_offset();
        self.button.ok_mut()?.set_position(hidden_position);
        Ok(())
    }

    pub fn slide_in(&mut self) -> ArreResult<()> {
        // In case of parent leaving scene we return early to avoid tween errors
        if !self.base.is_inside_tree() { return Ok(()); }
        self.sliding_tween.ok_mut()?.kill();
        let mut new_sliding_tween = self.base.create_tween()
            .ok_or(ArreError::UnexpectedNone("SlidingButton::slide_in".into()))?;

        match self.direction {
            SlidingInDirection::Right => {
                new_sliding_tween.tween_property(
                    self.button.ok_shared()?.upcast(),
                    "position:x".into(),
                    0.to_variant(),
                    0.2
                );
            },
            SlidingInDirection::Bottom => {
                new_sliding_tween.tween_property(
                    self.button.ok_shared()?.upcast(),
                    "position:y".into(),
                    0.to_variant(),
                    0.2,
                );
            }
        }

        new_sliding_tween.play();
        self.sliding_tween = new_sliding_tween.into();
        Ok(())
    }

    pub fn slide_out(&mut self) -> ArreResult<()> {
        // In case of parent leaving scene we return early to avoid tween errors
        if !self.base.is_inside_tree() { return Ok(()); }
        self.sliding_tween.ok_mut()?.kill();
        let mut new_sliding_tween = self.base.create_tween()
            .ok_or(ArreError::UnexpectedNone("SlidingButton::slide_out".into()))?;

        match self.direction {
            SlidingInDirection::Right => {
                new_sliding_tween.tween_property(
                    self.button.ok_shared()?.upcast(),
                    "position:x".into(),
                    self.get_hiding_offset().x.to_variant(),
                    0.2
                );
            },
            SlidingInDirection::Bottom => {
                new_sliding_tween.tween_property(
                    self.button.ok_shared()?.upcast(),
                    "position:y".into(),
                    self.get_hiding_offset().y.to_variant(),
                    0.2
                );
            }
        }
        new_sliding_tween.play();
        self.sliding_tween = new_sliding_tween.into();
        Ok(())
    }

    fn get_hiding_offset(&self) -> Vector2 {
        match self.direction {
            SlidingInDirection::Right => {
                Vector2::new(-self.base.get_size().x, 0.)
            },
            SlidingInDirection::Bottom => {
                Vector2::new(0., -self.base.get_size().y)
            }
        }
    }
}

#[godot_api]
impl ControlVirtual for SlidingButton {
    fn init(base: Base<Self::Base>) -> Self {
        Self {
            base,

            // cached UI elements
            button: GdHolder::default(),

            // state
            direction: SlidingInDirection::Right,
            sliding_tween: GdHolder::default(),
        }
    }

    fn ready(&mut self) {
        match try {
            let base = &mut self.base;

            let mut sliding_tween = base.create_tween().unwrap();
            sliding_tween.stop();
            self.sliding_tween = sliding_tween.into();
            self.button = GdHolder::from_path(base, "Button");
            self.set_sliding_direction(SlidingInDirection::Right)?;
        } {
            Ok(_) => {}
            Err::<_, BoxedError>(e) => log_error(e),
        }
    }
}
