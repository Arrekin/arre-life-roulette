use chrono::{Duration};
use godot::engine::{Panel, PanelVirtual, Button, Label};
use godot::prelude::*;
use crate::errors::{ArreResult, BoxedError};
use crate::godot_classes::singletons::logger::log_error;
use crate::godot_classes::utils::{GdHolder};
use crate::godot_classes::views::roll::subview_rolling::RollRollingSubview;
use crate::godot_classes::views::roll::subview_selection::RollSelectionSubview;
use crate::godot_classes::views::roll::subview_work_assigned::RollWorkAssignedSubview;
use crate::godot_classes::views::roll::subview_work_finished::RollWorkFinishedSubview;
use crate::item::{Item};
use crate::list::{List};

pub enum RollState {
    ItemsSelection,
    Rolling(Vec<Item>),
    WorkAssigned{item: Item},
    WorkFinished(Duration),
}

#[derive(GodotClass)]
#[class(base=Panel)]
pub struct RollView {
    #[base]
    base: Base<Panel>,

    // cached UI elements
    work_cancel_button: GdHolder<Button>,
    list_name_label: GdHolder<Label>,
    // subviews
    selection_subview: GdHolder<RollSelectionSubview>,
    rolling_subview: GdHolder<RollRollingSubview>,
    work_assigned_subview: GdHolder<RollWorkAssignedSubview>,
    work_finished_subview: GdHolder<RollWorkFinishedSubview>,

    // state
    list: List,
    pub roll_state: RollState,
    roll_state_requested: Option<RollState>,
}

#[godot_api]
impl RollView {
    #[signal]
    fn dialog_closed();

    pub fn set_list(&mut self, list: List) {
        self.roll_state_requested = Some(RollState::ItemsSelection);
        self.list = list;
    }

    #[func]
    pub fn refresh_view(&mut self){
        match try {
            self.list_name_label.ok_mut()?.set_text(self.list.name.clone().into());
            self.hide_all_subviews()?;
            match &self.roll_state {
                RollState::ItemsSelection => self.selection_subview.ok_mut()?.bind_mut().set_visible(true),
                RollState::Rolling(_items_list) => self.rolling_subview.ok_mut()?.bind_mut().set_visible(true),
                RollState::WorkAssigned{..} => self.work_assigned_subview.ok_mut()?.bind_mut().set_visible(true),
                RollState::WorkFinished(_duration) => self.work_finished_subview.ok_mut()?.bind_mut().set_visible(true),
            }
        } {
            Ok(_) => {}
            Err::<_, BoxedError>(e) => log_error(e)
        }
    }

    fn hide_all_subviews(&mut self) -> ArreResult<()> {
        self.selection_subview.ok_mut()?.bind_mut().set_visible(false);
        self.rolling_subview.ok_mut()?.bind_mut().set_visible(false);
        self.work_assigned_subview.ok_mut()?.bind_mut().set_visible(false);
        self.work_finished_subview.ok_mut()?.bind_mut().set_visible(false);
        Ok(())
    }

    #[func]
    fn on_work_finish_button_up(&mut self) {
        self.roll_state_requested = Some(RollState::WorkFinished(Duration::zero()));
    }

    #[func]
    pub fn close_dialog(&mut self) {
        self.base.hide();
        self.emit_signal("dialog_closed".into(), &[]);
    }

    #[func]
    fn on_roll_again_button_up(&mut self) {
        self.roll_state = RollState::ItemsSelection;
        self.refresh_view();
    }

    pub fn roll_state_change_request(&mut self, new_state: RollState) {
        self.roll_state_requested = Some(new_state);
    }
}

#[godot_api]
impl PanelVirtual for RollView {
    fn init(base: Base<Self::Base>) -> Self {
        Self {
            base,

            // cached UI elements
            work_cancel_button: GdHolder::default(),
            list_name_label: GdHolder::default(),
            // subviews
            selection_subview: GdHolder::default(),
            rolling_subview: GdHolder::default(),
            work_assigned_subview: GdHolder::default(),
            work_finished_subview: GdHolder::default(),

            list: List::default(),
            roll_state: RollState::ItemsSelection,
            roll_state_requested: None,
        }
    }
    fn ready(&mut self) {
        match try {
            let base = &self.base;
            // main UI elements
            self.work_cancel_button = GdHolder::from_path(base, "WorkCancelButton");
            self.work_cancel_button.ok_mut()?.connect(
                "button_up".into(),
                base.callable("close_dialog"),
                0,
            );
            self.list_name_label = GdHolder::from_path(base, "VBoxContainer/TopMarginContainer/ListNameLabel");
            // subviews
            self.selection_subview = GdHolder::from_path(base, "VBoxContainer/SelectionSubview");
            self.selection_subview.ok_mut()?.bind_mut().roll_view = GdHolder::from_gd(base.share());
            self.rolling_subview = GdHolder::from_path(base, "VBoxContainer/RollingSubview");
            self.rolling_subview.ok_mut()?.bind_mut().roll_view = GdHolder::from_gd(base.share());
            self.work_assigned_subview = GdHolder::from_path(base, "VBoxContainer/WorkAssignedSubview");
            self.work_assigned_subview.ok_mut()?.bind_mut().roll_view = GdHolder::from_gd(base.share());
            self.work_finished_subview = GdHolder::from_path(base, "VBoxContainer/WorkFinishedSubview");
            self.work_finished_subview.ok_mut()?.bind_mut().roll_view = GdHolder::from_gd(base.share());
        } {
            Ok(_) => {}
            Err::<_, BoxedError>(e) => log_error(e),
        }
    }
    fn process(&mut self, _delta: f64) {
        match try {
            if let Some(new_requested_state) = self.roll_state_requested.take() {
                self.roll_state = new_requested_state;
                match &self.roll_state {
                    RollState::ItemsSelection => {
                        let mut selection_subview = self.selection_subview.ok_mut()?.bind_mut();
                        selection_subview.set_state(self.list.get_id()?);
                        selection_subview.refresh_display();
                    },
                    RollState::Rolling(eligible_items) => {
                        self.rolling_subview.ok_mut()?.bind_mut().animate(eligible_items.clone());
                    },
                    RollState::WorkAssigned{item} => {
                        let mut work_subview = self.work_assigned_subview.ok_mut()?.bind_mut();
                        work_subview.set_state(item.clone());
                        work_subview.refresh_display();
                    }
                    RollState::WorkFinished(..) => {}
                };
                self.refresh_view();
            };
        } {
            Ok(_) => {}
            Err::<_, BoxedError>(e) => log_error(e),
        }
    }
}