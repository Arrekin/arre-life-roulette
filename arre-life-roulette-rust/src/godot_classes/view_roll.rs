use godot::builtin::{Callable};
use godot::engine::{Control, Panel, PanelVirtual, Button, Label};
use godot::prelude::*;
use rand::Rng;
use crate::errors::{ArreError, ArreResult};
use crate::godot_classes::singletons::globals::{Globals};
use crate::godot_classes::singletons::logger::log_error;
use crate::godot_classes::utils::{GdHolder, get_singleton};
use crate::item::Item;
use crate::list::{List, list_items_get};

pub enum RollState {
    AwaitingRoll,
    Rolling, // To be used in future when Rolling animation plays
    WorkAssigned,
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
    awaiting_subview: GdHolder<Control>,
    roll_start_button: GdHolder<Button>,
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
    items: Vec<Item>,
    roll_state: RollState,
    work_item: usize,
}

#[godot_api]
impl RollView {
    #[signal]
    fn dialog_closed();

    pub fn set_list(&mut self, list: List) -> ArreResult<()> {
        self.list = list;
        let globals = get_singleton::<Globals>("Globals");
        let connection = &globals.bind().connection;
        self.items = list_items_get(connection, self.list.get_id()?)?;
        Ok(())
    }

    #[func]
    pub fn refresh_view(&mut self){
        match try {
            self.list_name_label.ok_mut()?.set_text(self.list.name.clone().into());
            match self.roll_state {
                RollState::AwaitingRoll => {
                    self.awaiting_subview.ok_mut()?.set_visible(true);
                    self.work_assigned_subview.ok_mut()?.set_visible(false);
                    self.work_finished_subview.ok_mut()?.set_visible(false);
                },
                RollState::Rolling => {
                    // TODO
                },
                RollState::WorkAssigned => {
                    self.awaiting_subview.ok_mut()?.set_visible(false);
                    self.work_assigned_subview.ok_mut()?.set_visible(true);
                    self.work_finished_subview.ok_mut()?.set_visible(false);
                    let work_item = &self.items[self.work_item];
                    self.item_name_label.ok_mut()?.set_text(work_item.name.clone().into());
                    self.item_description_label.ok_mut()?.set_text(work_item.description.clone().into());
                },
                RollState::WorkFinished => {
                    self.awaiting_subview.ok_mut()?.set_visible(false);
                    self.work_assigned_subview.ok_mut()?.set_visible(false);
                    self.work_finished_subview.ok_mut()?.set_visible(true);
                }
            }
        }: ArreResult<()> {
            Ok(_) => {}
            Err(e) => { log_error(e); }
        }
    }

    #[func]
    fn on_roll_start_button_up(&mut self) {
        if self.items.len() == 0 {
            log_error(ArreError::ListHasNoItems(self.list.name.clone()));
            return;
        }
        let mut rng = rand::thread_rng();
        self.work_item = rng.gen_range(0..self.items.len());

        self.roll_state = RollState::WorkAssigned;
        godot_print!("Selected work item: {}", self.work_item);
        self.refresh_view();
    }

    #[func]
    fn on_work_finish_button_up(&mut self) {
        self.roll_state = RollState::WorkFinished;
        self.refresh_view();
    }

    #[func]
    fn on_work_cancel_button_up(&mut self) {
        self.base.hide();
        self.emit_signal("dialog_closed".into(), &[]);
    }

    #[func]
    fn on_roll_again_button_up(&mut self) {
        self.roll_state = RollState::AwaitingRoll;
        self.refresh_view();
    }

    #[func]
    fn on_close_button_up(&mut self) {
        self.base.hide();
        self.emit_signal("dialog_closed".into(), &[]);
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
            awaiting_subview: GdHolder::default(),
            roll_start_button: GdHolder::default(),
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
            items: Vec::new(),
            roll_state: RollState::AwaitingRoll,
            work_item: 0,
        }
    }
    fn ready(&mut self) {
        match try {
            let base = &self.base;
            // main UI elements
            self.work_cancel_button = GdHolder::from_path(base, "WorkCancelButton");
            self.work_cancel_button.ok_mut()?.connect(
                "button_up".into(),
                Callable::from_object_method(self.base.share(), "on_work_cancel_button_up"),
                0,
            );
            self.list_name_label = GdHolder::from_path(base, "ListNameLabel");
            // awaiting subview
            self.awaiting_subview = GdHolder::from_path(base, "AwaitingSubview");
            self.roll_start_button = GdHolder::from_path(base, "AwaitingSubview/RollStartButton");
            self.roll_start_button.ok_mut()?.connect(
                "button_up".into(),
                Callable::from_object_method(self.base.share(), "on_roll_start_button_up"),
                0,
            );

            // work assigned subview
            self.work_assigned_subview = GdHolder::from_path(base, "WorkAssignedSubview");
            self.item_name_label = GdHolder::from_path(base, "WorkAssignedSubview/ItemNameLabel");
            self.item_description_label = GdHolder::from_path(base, "WorkAssignedSubview/ItemDescriptionLabel");
            self.work_finish_button = GdHolder::from_path(base, "WorkAssignedSubview/WorkFinishButton");
            self.work_finish_button.ok_mut()?.connect(
                "button_up".into(),
                Callable::from_object_method(self.base.share(), "on_work_finish_button_up"),
                0,
            );

            // work finished subview
            self.work_finished_subview = GdHolder::from_path(base, "WorkFinishedSubview");
            self.roll_again_button = GdHolder::from_path(base, "WorkFinishedSubview/RollAgainButton");
            self.roll_again_button.ok_mut()?.connect(
                "button_up".into(),
                Callable::from_object_method(self.base.share(), "on_roll_again_button_up"),
                0,
            );
            self.close_button = GdHolder::from_path(base, "WorkFinishedSubview/CloseButton");
            self.close_button.ok_mut()?.connect(
                "button_up".into(),
                Callable::from_object_method(self.base.share(), "on_close_button_up"),
                0,
            );
        }: ArreResult<()> {
            Ok(_) => {}
            Err(e) => log_error(e),
        }
    }
}