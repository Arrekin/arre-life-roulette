use godot::engine::{VBoxContainer, VBoxContainerVirtual};
use godot::prelude::*;
use rand::prelude::IteratorRandom;
use crate::errors::{ArreResult, ArreError, BoxedError};
use crate::godot_classes::singletons::globals::Globals;
use crate::godot_classes::utils::get_singleton;
use crate::godot_classes::singletons::logger::log_error;
use crate::godot_classes::utils::{GdHolder};
use crate::godot_classes::views::roll::view_roll::{RollState, RollView};
use crate::item::{item_get};


#[derive(GodotClass)]
#[class(base=VBoxContainer)]
pub struct RollRollingSubview {
    #[base]
    base: Base<VBoxContainer>,

    // cached external UI elements
    pub roll_view: GdHolder<RollView>,

    // state
    pub do_roll_deferred: bool,
}

#[godot_api]
impl RollRollingSubview {

    pub fn roll(&mut self) -> ArreResult<()>{
        let mut roll_view = self.roll_view.ok_mut()?.bind_mut();
        let change_state = if let RollState::Rolling(eligible_items) = &roll_view.roll_state {
            let mut rng = rand::thread_rng();
            let work_item = eligible_items.iter().choose(&mut rng).ok_or(ArreError::ItemsSelectionIsEmpty())?;
            Some(*work_item)
        } else {
            None
        };
        if let Some(work_item) = change_state {
            let globals = get_singleton::<Globals>("Globals");
            let connection = &globals.bind().connection;
            roll_view.roll_state_change_request(RollState::WorkAssigned{item: item_get(connection, work_item)?});
        }
        Ok(())
    }
}

#[godot_api]
impl VBoxContainerVirtual for RollRollingSubview {
    fn init(base: Base<Self::Base>) -> Self {
        Self {
            base,

            // cached external UI elements
            roll_view: GdHolder::default(),

            // state
            do_roll_deferred: false,
        }
    }
    fn ready(&mut self) {
        match try {
            // cached external UI elements
            // self.roll_view is set from RollView::ready()
        } {
            Ok(_) => {}
            Err::<_, BoxedError>(e) => log_error(e),
        }
    }

    fn process(&mut self, _delta: f64) {
        match try {
            if self.do_roll_deferred {
                self.roll()?;
                self.do_roll_deferred = false;
            }

        } {
            Ok(_) => {}
            Err::<_, BoxedError>(e) => log_error(e),
        }
    }
}
