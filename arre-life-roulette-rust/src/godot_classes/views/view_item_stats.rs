use godot::engine::{Panel, PanelVirtual, Button, Label};
use godot::prelude::*;
use crate::errors::{BoxedError};
use crate::godot_classes::singletons::logger::log_error;
use crate::godot_classes::utils::{GdHolder};
use crate::item_stats::ItemStats;
use crate::utils::format_duration;

const UI_TEXT_TIMES_WORKED: &str = "Times Worked: ";
const UI_TEXT_TIME_SPENT: &str = "Time Spent: ";

#[derive(GodotClass)]
#[class(base=Panel)]
pub struct ItemStatsView {
    #[base]
    base: Base<Panel>,

    // cached elements
    pub times_worked_label: GdHolder<Label>,
    pub time_spent_label: GdHolder<Label>,
    pub close_button: GdHolder<Button>,

    // state
    pub item_stats: ItemStats,
}

#[godot_api]
impl ItemStatsView {
    #[signal]
    fn dialog_closed();

    #[func]
    pub fn refresh_display(&mut self) {
        match try {
            let time_spent = format_duration(self.item_stats.time_spent);
            self.times_worked_label.ok_mut()?.set_text(format!("{}{}", UI_TEXT_TIMES_WORKED, self.item_stats.times_worked).into());
            self.time_spent_label.ok_mut()?.set_text(format!("{}{}", UI_TEXT_TIME_SPENT, time_spent).into());
        } {
            Ok(_) => {}
            Err::<_, BoxedError>(e) => log_error(e)
        }
    }

    #[func]
    fn on_dialog_close_button_up(&mut self) {
        self.hide();
        self.emit_signal("dialog_closed".into(), &[]);
    }
}

#[godot_api]
impl PanelVirtual for ItemStatsView {
    fn init(base: Base<Self::Base>) -> Self {
        Self {
            base,
            times_worked_label: GdHolder::default(),
            time_spent_label: GdHolder::default(),
            close_button: GdHolder::default(),

            item_stats: ItemStats::default(),
        }
    }
    fn ready(&mut self) {
        match try {
            let base = &self.base;
            self.times_worked_label = GdHolder::from_path(base, "VBoxContainer/VBoxContainer/TimesWorkedLabel");
            self.time_spent_label = GdHolder::from_path(base, "VBoxContainer/VBoxContainer/TimeSpentLabel");
            self.close_button = GdHolder::from_path(base,"DialogCloseButton");
            self.close_button.ok_mut()?.connect(
                "button_up".into(),
                base.callable("on_dialog_close_button_up"),
                0,
            );
        } {
            Ok(_) => {}
            Err::<_, BoxedError>(e) => log_error(e)
        }
    }
}