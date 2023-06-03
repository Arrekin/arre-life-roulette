use chrono::{DateTime, Duration, Utc};
use godot::engine::{Control, Panel, PanelVirtual, Button, Label};
use godot::prelude::*;
use rand::seq::{IteratorRandom};
use crate::errors::{ArreResult, ArreError};
use crate::godot_classes::singletons::globals::{Globals};
use crate::godot_classes::singletons::logger::log_error;
use crate::godot_classes::utils::{GdHolder, get_singleton};
use crate::godot_classes::views::roll::subview_selection::RollSelectionSubview;
use crate::item::{Item, item_get, ItemId};
use crate::item_stats::{item_stats_get, item_stats_update};
use crate::list::{List};

pub enum RollState {
    ItemsSelection,
    Rolling(Vec<ItemId>),
    WorkAssigned(Item),
    WorkFinished,
}

#[derive(GodotClass)]
#[class(base=Panel)]
pub struct RollView {
    #[base]
    base: Base<Panel>,

    // cached UI elements
    work_cancel_button: GdHolder<Button>,
    list_name_label: GdHolder<Label>,
    // awaiting subview
    selection_subview: GdHolder<RollSelectionSubview>,
    // work assigned subview
    work_assigned_subview: GdHolder<Control>,
    item_name_label: GdHolder<Label>,
    item_description_label: GdHolder<Label>,
    work_finish_button: GdHolder<Button>,
    // work finished subview
    work_finished_subview: GdHolder<Control>,
    roll_again_button: GdHolder<Button>,
    close_button: GdHolder<Button>,

    // state
    list: List,
    roll_state: RollState,
    roll_state_requested: Option<RollState>,
    work_start_timestamp: DateTime<Utc>,
    time_worked: Duration, // To display after work is done
}

#[godot_api]
impl RollView {
    #[signal]
    fn dialog_closed();

    pub fn set_list(&mut self, list: List) -> ArreResult<()> {
        // Reset view
        self.roll_state = RollState::ItemsSelection;

        self.list = list;
        self.selection_subview.ok_mut()?.bind_mut().set_state(self.list.get_id()?);
        self.selection_subview.ok_mut()?.bind_mut().refresh_display();
        Ok(())
    }

    #[func]
    pub fn refresh_view(&mut self){
        match try {
            self.list_name_label.ok_mut()?.set_text(self.list.name.clone().into());
            match &self.roll_state {
                RollState::ItemsSelection => {
                    self.hide_all_subviews()?;
                    self.selection_subview.ok_mut()?.bind_mut().set_visible(true);
                },
                RollState::Rolling(_items_list) => {
                    // TODO
                },
                RollState::WorkAssigned(work_item) => {
                    self.item_name_label.ok_mut()?.set_text(work_item.name.clone().into());
                    self.item_description_label.ok_mut()?.set_text(work_item.description.clone().into());
                    self.hide_all_subviews()?;
                    self.work_assigned_subview.ok_mut()?.set_visible(true);
                },
                RollState::WorkFinished => {
                    self.hide_all_subviews()?;
                    self.work_finished_subview.ok_mut()?.set_visible(true);
                }
            }
        }: ArreResult<()> {
            Ok(_) => {}
            Err(e) => { log_error(e); }
        }
    }

    fn hide_all_subviews(&mut self) -> ArreResult<()> {
        self.selection_subview.ok_mut()?.bind_mut().set_visible(false);
        self.work_assigned_subview.ok_mut()?.set_visible(false);
        self.work_finished_subview.ok_mut()?.set_visible(false);
        Ok(())
    }

    #[func]
    fn on_work_finish_button_up(&mut self) {
        self.roll_state_requested = Some(RollState::WorkFinished);
    }

    #[func]
    fn on_work_cancel_button_up(&mut self) {
        self.base.hide();
        self.emit_signal("dialog_closed".into(), &[]);
    }

    #[func]
    fn on_roll_again_button_up(&mut self) {
        self.roll_state = RollState::ItemsSelection;
        self.refresh_view();
    }

    #[func]
    fn on_close_button_up(&mut self) {
        self.base.hide();
        self.emit_signal("dialog_closed".into(), &[]);
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
            // awaiting subview
            selection_subview: GdHolder::default(),
            // work assigned subview
            work_assigned_subview: GdHolder::default(),
            item_name_label: GdHolder::default(),
            item_description_label: GdHolder::default(),
            work_finish_button: GdHolder::default(),
            // work finished subview
            work_finished_subview: GdHolder::default(),
            roll_again_button: GdHolder::default(),
            close_button: GdHolder::default(),

            list: List::default(),
            roll_state: RollState::ItemsSelection,
            roll_state_requested: None,
            work_start_timestamp: Utc::now(),
            time_worked: Duration::zero(),
        }
    }
    fn ready(&mut self) {
        match try {
            let base = &self.base;
            // main UI elements
            self.work_cancel_button = GdHolder::from_path(base, "WorkCancelButton");
            self.work_cancel_button.ok_mut()?.connect(
                "button_up".into(),
                base.callable("on_work_cancel_button_up"),
                0,
            );
            self.list_name_label = GdHolder::from_path(base, "VBoxContainer/TopMarginContainer/ListNameLabel");
            // selection subview
            self.selection_subview = GdHolder::from_path(base, "VBoxContainer/SelectionSubview");
            self.selection_subview.ok_mut()?.bind_mut().roll_view = GdHolder::from_gd(base.share());

            // work assigned subview
            self.work_assigned_subview = GdHolder::from_path(base, "VBoxContainer/WorkAssignedSubview");
            self.item_name_label = GdHolder::from_path(base, "VBoxContainer/WorkAssignedSubview/ItemNameLabel");
            self.item_description_label = GdHolder::from_path(base, "VBoxContainer/WorkAssignedSubview/ItemDescriptionLabel");
            self.work_finish_button = GdHolder::from_path(base, "VBoxContainer/WorkAssignedSubview/WorkFinishButton");
            self.work_finish_button.ok_mut()?.connect(
                "button_up".into(),
                base.callable("on_work_finish_button_up"),
                0,
            );

            // work finished subview
            self.work_finished_subview = GdHolder::from_path(base, "VBoxContainer/WorkFinishedSubview");
            self.roll_again_button = GdHolder::from_path(base, "VBoxContainer/WorkFinishedSubview/RollAgainButton");
            self.roll_again_button.ok_mut()?.connect(
                "button_up".into(),
                base.callable("on_roll_again_button_up"),
                0,
            );
            self.close_button = GdHolder::from_path(base, "VBoxContainer/WorkFinishedSubview/CloseButton");
            self.close_button.ok_mut()?.connect(
                "button_up".into(),
                base.callable("on_close_button_up"),
                0,
            );
        }: ArreResult<()> {
            Ok(_) => {}
            Err(e) => log_error(e),
        }
    }
    fn process(&mut self, _delta: f64) {
        match try {
            if let Some(new_requested_state) = self.roll_state_requested.take() {
                self.roll_state  = match new_requested_state {
                    RollState::ItemsSelection => {
                        RollState::ItemsSelection
                    },
                    RollState::Rolling(work_items) => {
                        let mut rng = rand::thread_rng();
                        let work_item = work_items.into_iter().choose(&mut rng).ok_or(ArreError::ItemsSelectionIsEmpty())?;

                        let globals = get_singleton::<Globals>("Globals");
                        let connection = &globals.bind().connection;

                        self.work_start_timestamp = Utc::now();
                        RollState::WorkAssigned(item_get(connection, work_item)?)
                    },
                    RollState::WorkAssigned(item_id) => {
                        RollState::WorkAssigned(item_id)
                    }
                    RollState::WorkFinished => {
                        if let RollState::WorkAssigned(item) = &self.roll_state {
                            self.time_worked = Utc::now() - self.work_start_timestamp;

                            // Update stats in db
                            let globals = get_singleton::<Globals>("Globals");
                            let connection = &globals.bind().connection;
                            let mut item_stats = item_stats_get(connection, item.get_id()?)?;
                            item_stats.times_worked += 1;
                            item_stats.time_spent = item_stats.time_spent + self.time_worked;
                            item_stats_update(connection, &item_stats)?;
                        }
                        RollState::WorkFinished
                    }
                };
                self.refresh_view();
            };
        }: ArreResult<()> {
            Ok(_) => {}
            Err(e) => log_error(e),
        }
    }
}