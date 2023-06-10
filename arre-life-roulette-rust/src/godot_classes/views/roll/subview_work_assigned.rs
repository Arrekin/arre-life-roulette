use chrono::{DateTime, Duration, Utc};
use godot::engine::{Button, Label, VBoxContainer, VBoxContainerVirtual};
use godot::prelude::*;
use crate::db::DB;
use crate::errors::{ArreResult, BoxedError};
use crate::godot_classes::singletons::logger::log_error;
use crate::godot_classes::utils::{GdHolder};
use crate::godot_classes::views::roll::view_roll::{RollState, RollView};
use crate::item::{Item};
use crate::item_details::{item_details_get, ItemDetails};
use crate::item_stats::{item_stats_get, item_stats_update};
use crate::utils::format_duration;


#[derive(GodotClass)]
#[class(base=VBoxContainer)]
pub struct RollWorkAssignedSubview {
    #[base]
    base: Base<VBoxContainer>,

    // cached internal UI elements
    pub item_name_label: GdHolder<Label>,
    pub item_description_label: GdHolder<Label>,
    pub session_time_label: GdHolder<Label>,
    pub elapsed_time_label: GdHolder<Label>,
    pub work_finish_button: GdHolder<Button>,

    // cached external UI elements
    pub roll_view: GdHolder<RollView>,

    // state
    pub work_item: Item,
    pub work_item_details: ItemDetails,
    pub work_started_timestamp: DateTime<Utc>
}

#[godot_api]
impl RollWorkAssignedSubview {

    pub fn set_state(&mut self, work_item: Item) -> ArreResult<()> {
        self.work_item = work_item;
        self.work_started_timestamp = Utc::now();

        let connection = &*DB.ok()?;
        self.work_item_details = item_details_get(connection, self.work_item.get_id()?)?;
        self.refresh_display()?;
        Ok(())
    }

    pub fn refresh_display(&mut self) -> ArreResult<()> {
        self.item_name_label.ok_mut()?.set_text(self.work_item.name.clone().into());
        self.item_description_label.ok_mut()?.set_text(self.work_item.description.clone().into());
        self.refresh_time_display()?;
        Ok(())
    }

    pub fn refresh_time_display(&mut self) -> ArreResult<()> {
        let now = Utc::now();
        let elapsed_time = now - self.work_started_timestamp;
        self.elapsed_time_label.ok_mut()?.set_text(format_duration(elapsed_time).into());
        if let Some(session_time) = self.work_item_details.session_duration {
            let session_time = session_time - elapsed_time;
            let session_time = if session_time < Duration::zero() { Duration::zero() } else { session_time };
            self.session_time_label.ok_mut()?.set_text(format_duration(session_time).into());
            self.session_time_label.ok_mut()?.set_visible(true);
        } else {
            self.session_time_label.ok_mut()?.set_visible(false);
        }
        Ok(())
    }

    #[func]
    fn on_work_finish_button_up(&mut self) {
        match try {
            let time_worked = Utc::now() - self.work_started_timestamp;
            // Update stats in db
            let connection = &*DB.ok()?;
            let mut item_stats = item_stats_get(connection, self.work_item.get_id()?)?;
            item_stats.times_worked += 1;
            item_stats.time_spent = item_stats.time_spent + time_worked;
            item_stats_update(connection, &item_stats)?;
            self.roll_view.ok_mut()?.bind_mut().roll_state_change_request(RollState::WorkFinished(time_worked));
        } {
            Ok(_) => {}
            Err::<_, BoxedError>(e) => log_error(e)
        }
    }
}

#[godot_api]
impl VBoxContainerVirtual for RollWorkAssignedSubview {
    fn init(base: Base<Self::Base>) -> Self {
        Self {
            base,

            // cached internal UI elements
            item_name_label: GdHolder::default(),
            item_description_label: GdHolder::default(),
            session_time_label: GdHolder::default(),
            elapsed_time_label: GdHolder::default(),
            work_finish_button: GdHolder::default(),

            // cached external UI elements
            roll_view: GdHolder::default(),

            // state
            work_item: Item::default(),
            work_item_details: ItemDetails::default(),
            work_started_timestamp: Utc::now(),
        }
    }
    fn ready(&mut self) {
        match try {
            let base = &self.base;

            // cached internal UI elements
            self.item_name_label = GdHolder::from_path(base, "VBoxContainer/ItemNameLabel");
            self.item_description_label = GdHolder::from_path(base, "VBoxContainer/ItemDescriptionLabel");
            self.session_time_label = GdHolder::from_path(base, "VBoxContainer/SessionTimeLabel");
            self.elapsed_time_label = GdHolder::from_path(base, "VBoxContainer/ElapsedTimeLabel");
            self.work_finish_button = GdHolder::from_path(base, "VBoxContainer/BottomMarginContainer/WorkFinishButton");
            self.work_finish_button.ok_mut()?.connect(
                "button_up".into(),
                base.callable("on_work_finish_button_up"),
                0,
            );

            // cached external UI elements
            // self.roll_view is set from RollView::ready()
        } {
            Ok(_) => {}
            Err::<_, BoxedError>(e) => log_error(e),
        }
    }

    fn process(&mut self, _delta: f64) {
        match try {
            if godot::engine::Engine::singleton().is_editor_hint() { return; }
            if self.base.is_visible() {
                self.refresh_time_display()?;
            }
        } {
            Ok(_) => {}
            Err::<_, BoxedError>(e) => log_error(e),
        }
    }
}
