use godot::builtin::{Callable};
use godot::engine::{Control, ControlVirtual, Button};
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
    WorkCanceled,
}

#[derive(GodotClass)]
#[class(base=Control)]
pub struct RollView {
    #[base]
    base: Base<Control>,

    // cached UI elements
    roll_start_button: Option<Gd<Button>>,
    work_finish_button: Option<Gd<Button>>,
    work_cancel_button: Option<Gd<Button>>,

    // state
    list: List,
    roll_state: RollState,
    work_item: usize,
}

#[godot_api]
impl RollView {

    #[func]
    fn on_roll_start_button_up(&mut self) {
        // Load the list items and randomly choose one element
        let globals = get_singleton::<Globals>("Globals");
        let connection = &globals.bind().connection;
        // TODO: Instead of loading full items, load only their Ids
        self.list.load_items(connection).unwrap();

        let mut rng = rand::thread_rng();
        self.work_item = rng.gen_range(0..self.list.items.len());
        godot_print!("Selected work item: {}", self.work_item);
    }

    #[func]
    fn on_work_finish_button_up(&mut self) {
        godot_print!("Work finished");
    }

    #[func]
    fn on_work_cancel_button_up(&mut self) {
        self.base.hide();
    }
}

#[godot_api]
impl ControlVirtual for RollView {
    fn init(base: Base<Self::Base>) -> Self {
        Self {
            base,

            roll_start_button: None,
            work_finish_button: None,
            work_cancel_button: None,

            list: List::default(),
            roll_state: RollState::AwaitingRoll,
            work_item: 0,
        }
    }
    fn ready(&mut self) {
        self.roll_start_button = self.base.try_get_node_as("RollStartButton");
        self.roll_start_button.as_mut().map(|button| {
            button.connect(
                "button_up".into(),
                Callable::from_object_method(self.base.share(), "on_roll_start_button_up"),
                0,
            );
        });
        self.work_finish_button = self.base.try_get_node_as("WorkFinishButton");
        self.work_finish_button.as_mut().map(|button| {
            button.connect(
                "button_up".into(),
                Callable::from_object_method(self.base.share(), "on_work_finish_button_up"),
                0,
            );
        });
        self.work_cancel_button = self.base.try_get_node_as("WorkCancelButton");
        self.work_cancel_button.as_mut().map(|button| {
            button.connect(
                "button_up".into(),
                Callable::from_object_method(self.base.share(), "on_work_cancel_button_up"),
                0,
            );
        });
    }
}