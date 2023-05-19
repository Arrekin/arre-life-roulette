use std::collections::VecDeque;
use godot::engine::{Node, NodeVirtual};
use godot::prelude::*;
use crate::errors::BoxedError;
use crate::godot_classes::utils::get_singleton;

#[derive(GodotClass)]
#[class(base=Node)]
pub struct Logger {
    #[base]
    base: Base<Node>,

    max_logs_entries: usize,

    pub logs: VecDeque<GodotString>,
}

#[godot_api]
impl Logger {
    pub fn error(&mut self, error: impl Into<BoxedError>) {
        let error = error.into();
        self.push_log(error.to_string().into())
    }

    pub fn push_log(&mut self, log: GodotString) {
        if self.logs.len() >= self.max_logs_entries {
            self.logs.pop_front();
        }
        self.logs.push_back(log);
    }
}

#[godot_api]
impl NodeVirtual for Logger {
    fn init(base: Base<Self::Base>) -> Self {
        Self {
            base,

            logs: VecDeque::new(),
            max_logs_entries: 10,
        }
    }
}

pub fn log_error(error: impl Into<BoxedError>) {
    let error = error.into();
    utilities::push_error("Rust Error".to_variant(), &[error.to_string().to_variant()]);
    let mut logger = get_singleton::<Logger>("Logger");
    logger.bind_mut().error(error);
}