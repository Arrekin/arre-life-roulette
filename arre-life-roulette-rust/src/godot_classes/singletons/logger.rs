use std::collections::VecDeque;
use godot::engine::{Node, NodeVirtual, RegEx};
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
    pub regex_bbcode_strip: Gd<RegEx>,
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
        let mut regex_bbcode_strip = RegEx::new();
        regex_bbcode_strip.compile("\\[.*?\\]".into());
        Self {
            base,

            max_logs_entries: 10,

            logs: VecDeque::new(),
            regex_bbcode_strip,
        }
    }
}

pub fn log_error(error: impl Into<BoxedError>) {
    let error = error.into();
    let mut logger = get_singleton::<Logger>("Logger");
    {
        let mut logger = logger.bind_mut();
        let stripped_error = logger.regex_bbcode_strip.sub(error.to_string().into(), "".into());
        utilities::push_error("Rust Error".to_variant(), &[stripped_error.to_variant()]);
        logger.error(error);
    }
}