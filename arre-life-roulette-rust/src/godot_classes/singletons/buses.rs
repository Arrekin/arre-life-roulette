use std::cell::RefCell;
use std::rc::Rc;
use bus::{Bus, BusReader};
use godot::engine::{Node, NodeVirtual};
use godot::prelude::*;
use crate::errors::{ArreError, ArreResult};

pub const BUS_SIZE: usize = 1024;

pub enum BusType<T> {
    Owned(Bus<T>),
    Shared(Rc<RefCell<Bus<T>>>),
    None,
}

impl <T>BusType<T> {
    pub fn new_owned() -> Self {
        BusType::Owned(Bus::new(BUS_SIZE))
    }
    pub fn new_shared() -> Self {
        BusType::Shared(Rc::new(RefCell::new(Bus::new(BUS_SIZE))))
    }
    pub fn broadcast(&mut self, message: T) {
        match self {
            BusType::Owned(bus) => {
                bus.broadcast(message);
            },
            BusType::Shared(rc) => {
                rc.borrow_mut().broadcast(message);
            },
            BusType::None => {
                // do nothing
            }
        }
    }
    pub fn add_rx(&mut self) -> Option<BusReader<T>> {
        match self {
            BusType::Owned(bus) => {
                Some(bus.add_rx())
            },
            BusType::Shared(rc) => {
                Some(rc.borrow_mut().add_rx())
            },
            BusType::None => {
                None
            }
        }
    }
    pub fn cloned(&self) -> ArreResult<BusType<T>> {
        match self {
            BusType::Owned(_) => Err(ArreError::OwnedBusCannotBeCloned().into()),
            BusType::Shared(rc) => Ok(BusType::Shared(Rc::clone(rc))),
            BusType::None => Ok(BusType::None),
        }
    }
}

#[derive(GodotClass)]
#[class(base=Node)]
pub struct Buses {
    #[base]
    base: Base<Node>,
}

#[godot_api]
impl Buses {}

#[godot_api]
impl NodeVirtual for Buses {
    fn init(base: Base<Self::Base>) -> Self {
        Self {
            base,

        }
    }
}