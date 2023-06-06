use godot::engine::{Button, VBoxContainer, VBoxContainerVirtual};
use godot::prelude::*;
use crate::errors::{BoxedError};
use crate::godot_classes::singletons::logger::log_error;
use crate::godot_classes::utils::{GdHolder};
use crate::godot_classes::views::roll::view_roll::{RollState, RollView};


#[derive(GodotClass)]
#[class(base=VBoxContainer)]
pub struct RollWorkFinishedSubview {
    #[base]
    base: Base<VBoxContainer>,

    // cached internal UI elements
    roll_again_button: GdHolder<Button>,
    close_button: GdHolder<Button>,

    // cached external UI elements
    pub roll_view: GdHolder<RollView>,
}

#[godot_api]
impl RollWorkFinishedSubview {

    #[func]
    fn on_close_button_up(&mut self) {
        match try {
            self.roll_view.ok_mut()?.bind_mut().close_dialog();
        } {
            Ok(_) => {}
            Err::<_, BoxedError>(e) => log_error(e)
        }
    }

    #[func]
    fn on_roll_again_button_up(&mut self) {
        match try {
            self.roll_view.ok_mut()?.bind_mut().roll_state_change_request(RollState::ItemsSelection)
        } {
            Ok(_) => {}
            Err::<_, BoxedError>(e) => log_error(e)
        }
    }
}

#[godot_api]
impl VBoxContainerVirtual for RollWorkFinishedSubview {
    fn init(base: Base<Self::Base>) -> Self {
        Self {
            base,

            // cached internal UI elements
            roll_again_button: GdHolder::default(),
            close_button: GdHolder::default(),

            // cached external UI elements
            roll_view: GdHolder::default(),
        }
    }
    fn ready(&mut self) {
        match try {
            let base = &self.base;

            // cached internal UI elements
            self.roll_again_button = GdHolder::from_path(base, "RollAgainButton");
            self.roll_again_button.ok_mut()?.connect(
                "button_up".into(),
                base.callable("on_roll_again_button_up"),
                0,
            );
            self.close_button = GdHolder::from_path(base, "CloseButton");
            self.close_button.ok_mut()?.connect(
                "button_up".into(),
                base.callable("on_close_button_up"),
                0,
            );

            // cached external UI elements
            // self.roll_view is set from RollView::ready()
        } {
            Ok(_) => {}
            Err::<_, BoxedError>(e) => log_error(e),
        }
    }
}
