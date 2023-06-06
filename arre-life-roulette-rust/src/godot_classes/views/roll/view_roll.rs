use chrono::{Duration};
use godot::engine::{Control, Panel, PanelVirtual, Button, Label};
use godot::prelude::*;
use rand::seq::{IteratorRandom};
use crate::errors::{ArreResult, ArreError};
use crate::godot_classes::singletons::globals::{Globals};
use crate::godot_classes::singletons::logger::log_error;
use crate::godot_classes::utils::{GdHolder, get_singleton};
use crate::godot_classes::views::roll::subview_selection::RollSelectionSubview;
use crate::godot_classes::views::roll::subview_work_assigned::RollWorkAssignedSubview;
use crate::godot_classes::views::roll::subview_work_finished::RollWorkFinishedSubview;
use crate::item::{Item, item_get, ItemId};
use crate::list::{List};

pub enum RollState {
    ItemsSelection,
    Rolling(Vec<ItemId>),
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
    work_assigned_subview: GdHolder<RollWorkAssignedSubview>,
    work_finished_subview: GdHolder<RollWorkFinishedSubview>,

    // state
    list: List,
    roll_state: RollState,
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
            match &self.roll_state {
                RollState::ItemsSelection => {
                    self.hide_all_subviews()?;
                    self.selection_subview.ok_mut()?.bind_mut().set_visible(true);
                },
                RollState::Rolling(_items_list) => {
                    // TODO
                },
                RollState::WorkAssigned{..} => {
                    self.hide_all_subviews()?;
                    self.work_assigned_subview.ok_mut()?.bind_mut().set_visible(true);
                },
                RollState::WorkFinished(_duration) => {
                    self.hide_all_subviews()?;
                    self.work_finished_subview.ok_mut()?.bind_mut().set_visible(true);
                }
            }
        }: ArreResult<()> {
            Ok(_) => {}
            Err(e) => { log_error(e); }
        }
    }

    fn hide_all_subviews(&mut self) -> ArreResult<()> {
        self.selection_subview.ok_mut()?.bind_mut().set_visible(false);
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
            self.work_assigned_subview = GdHolder::from_path(base, "VBoxContainer/WorkAssignedSubview");
            self.work_assigned_subview.ok_mut()?.bind_mut().roll_view = GdHolder::from_gd(base.share());
            self.work_finished_subview = GdHolder::from_path(base, "VBoxContainer/WorkFinishedSubview");
            self.work_finished_subview.ok_mut()?.bind_mut().roll_view = GdHolder::from_gd(base.share());
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
                        let mut selection_subview = self.selection_subview.ok_mut()?.bind_mut();
                        selection_subview.set_state(self.list.get_id()?);
                        selection_subview.refresh_display();
                        RollState::ItemsSelection
                    },
                    RollState::Rolling(work_items) => {
                        let mut rng = rand::thread_rng();
                        let work_item = work_items.iter().choose(&mut rng).ok_or(ArreError::ItemsSelectionIsEmpty())?;

                        let globals = get_singleton::<Globals>("Globals");
                        let connection = &globals.bind().connection;

                        self.roll_state_change_request(RollState::WorkAssigned{item: item_get(connection, *work_item)?});
                        RollState::Rolling(work_items)
                    },
                    RollState::WorkAssigned{item} => {
                        let mut work_subview = self.work_assigned_subview.ok_mut()?.bind_mut();
                        work_subview.set_state(item.clone());
                        work_subview.refresh_display();
                        RollState::WorkAssigned{item}
                    }
                    RollState::WorkFinished(duration) => {
                        RollState::WorkFinished(duration)
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