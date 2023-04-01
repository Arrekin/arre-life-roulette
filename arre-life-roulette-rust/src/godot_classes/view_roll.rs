use godot::builtin::{Callable};
use godot::engine::{Control, Panel, PanelVirtual, Button, Label};
use godot::prelude::*;
use rand::Rng;
use crate::godot_classes::globals::{Globals};
use crate::godot_classes::utils::get_singleton;
use crate::list::List;

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
    work_cancel_button: Option<Gd<Button>>,
    list_name_label: Option<Gd<Label>>,
    // awaiting subview
    awaiting_subview: Option<Gd<Control>>,
    roll_start_button: Option<Gd<Button>>,
    // work assigned subview
    work_assigned_subview: Option<Gd<Control>>,
    item_name_label: Option<Gd<Label>>,
    item_description_label: Option<Gd<Label>>,
    work_finish_button: Option<Gd<Button>>,
    // work finished subview
    work_finished_subview: Option<Gd<Control>>,
    roll_again_button: Option<Gd<Button>>,
    close_button: Option<Gd<Button>>,

    // state
    list: List,
    roll_state: RollState,
    work_item: usize,
}

#[godot_api]
impl RollView {

    fn set_list(&mut self, list: List) {
        self.list = list;
    }

    #[func]
    fn refresh_view(&mut self){
        self.list_name_label.as_mut().unwrap().set_text(self.list.name.clone().into());
        match self.roll_state {
            RollState::AwaitingRoll => {
                self.awaiting_subview.as_mut().unwrap().set_visible(true);
                self.work_assigned_subview.as_mut().unwrap().set_visible(false);
                self.work_finished_subview.as_mut().unwrap().set_visible(false);
            },
            RollState::Rolling => {
                // TODO
            },
            RollState::WorkAssigned => {
                self.awaiting_subview.as_mut().unwrap().set_visible(false);
                self.work_assigned_subview.as_mut().unwrap().set_visible(true);
                self.work_finished_subview.as_mut().unwrap().set_visible(false);
                let work_item = &self.list.items[self.work_item];
                self.item_name_label.as_mut().unwrap().set_text(work_item.name.clone().into());
                self.item_description_label.as_mut().unwrap().set_text(work_item.description.clone().into());
            },
            RollState::WorkFinished => {
                self.awaiting_subview.as_mut().unwrap().set_visible(false);
                self.work_assigned_subview.as_mut().unwrap().set_visible(false);
                self.work_finished_subview.as_mut().unwrap().set_visible(true);
            }
        }
    }

    #[func]
    fn on_roll_start_button_up(&mut self) {
        // Load the list items and randomly choose one element
        let globals = get_singleton::<Globals>("Globals");
        let connection = &globals.bind().connection;
        // TODO: Instead of loading full items, load only their Ids
        self.list.load_items(connection).unwrap();

        let mut rng = rand::thread_rng();
        self.work_item = rng.gen_range(0..self.list.items.len());

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
    }

    #[func]
    fn on_roll_again_button_up(&mut self) {
        self.roll_state = RollState::AwaitingRoll;
        self.refresh_view();
    }

    #[func]
    fn on_close_button_up(&mut self) {
        self.base.hide();
    }
}

#[godot_api]
impl PanelVirtual for RollView {
    fn init(base: Base<Self::Base>) -> Self {
        Self {
            base,

            // cached UI elements
            work_cancel_button: None,
            list_name_label: None,
            // awaiting subview
            awaiting_subview: None,
            roll_start_button: None,
            // work assigned subview
            work_assigned_subview: None,
            item_name_label: None,
            item_description_label: None,
            work_finish_button: None,
            // work finished subview
            work_finished_subview: None,
            roll_again_button: None,
            close_button: None,

            list: List::default(),
            roll_state: RollState::AwaitingRoll,
            work_item: 0,
        }
    }
    fn ready(&mut self) {
        // main UI elements
        self.work_cancel_button = self.base.try_get_node_as("WorkCancelButton");
        self.work_cancel_button.as_mut().map(|button| {
            button.connect(
                "button_up".into(),
                Callable::from_object_method(self.base.share(), "on_work_cancel_button_up"),
                0,
            );
        });
        self.list_name_label = self.base.try_get_node_as("ListNameLabel");
        // awaiting subview
        self.awaiting_subview = self.base.try_get_node_as("AwaitingSubview");
        self.roll_start_button = self.base.try_get_node_as("AwaitingSubview/RollStartButton");
        self.roll_start_button.as_mut().map(|button| {
            button.connect(
                "button_up".into(),
                Callable::from_object_method(self.base.share(), "on_roll_start_button_up"),
                0,
            );
        });
        // work assigned subview
        self.work_assigned_subview = self.base.try_get_node_as("WorkAssignedSubview");
        self.item_name_label = self.base.try_get_node_as("WorkAssignedSubview/ItemNameLabel");
        self.item_description_label = self.base.try_get_node_as("WorkAssignedSubview/ItemDescriptionLabel");
        self.work_finish_button = self.base.try_get_node_as("WorkAssignedSubview/WorkFinishButton");
        self.work_finish_button.as_mut().map(|button| {
            button.connect(
                "button_up".into(),
                Callable::from_object_method(self.base.share(), "on_work_finish_button_up"),
                0,
            );
        });
        // work finished subview
        self.work_finished_subview = self.base.try_get_node_as("WorkFinishedSubview");
        self.roll_again_button = self.base.try_get_node_as("WorkFinishedSubview/RollAgainButton");
        self.roll_again_button.as_mut().map(|button| {
            button.connect(
                "button_up".into(),
                Callable::from_object_method(self.base.share(), "on_roll_again_button_up"),
                0,
            );
        });
        self.close_button = self.base.try_get_node_as("WorkFinishedSubview/CloseButton");
        self.close_button.as_mut().map(|button| {
            button.connect(
                "button_up".into(),
                Callable::from_object_method(self.base.share(), "on_close_button_up"),
                0,
            );
        });
    }
}