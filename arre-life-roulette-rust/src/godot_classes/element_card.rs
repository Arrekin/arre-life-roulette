use godot::engine::{MarginContainer, InputEvent, InputEventMouseButton, MarginContainerVirtual, Label, Button};
use godot::engine::global::MouseButton;
use godot::prelude::*;
use crate::errors::{BoxedError};
use crate::godot_classes::singletons::buses::{BusType};
use crate::godot_classes::singletons::logger::log_error;
use crate::godot_classes::utils::{GdHolder};
use crate::item::Item;
use crate::list::List;

#[derive(Clone)]
pub enum Content {
    Empty,
    Item(Item),
    List(List),
}

impl From<Item> for Content {
    fn from(value: Item) -> Self {
        Content::Item(value)
    }
}
impl From<List> for Content {
    fn from(value: List) -> Self {
        Content::List(value)
    }
}

#[derive(GodotClass)]
#[class(base=MarginContainer)]
pub struct ElementCard {
    #[base]
    base: Base<MarginContainer>,

    // cached UI elements
    pub button: GdHolder<Button>,
    pub name_label: GdHolder<Label>,
    pub description_label: GdHolder<Label>,

    // buses
    pub bus_left_click: BusType<InstanceId>,
    pub bus_right_click: BusType<InstanceId>,

    // state
    pub content: Content,
}

#[godot_api]
impl ElementCard {
    #[func]
    fn refresh_display(&mut self) {
        match try {
            let (name, description) = match &self.content {
                Content::Empty => ("".into(), "".into()),
                Content::Item(item) => (item.name.clone(), item.description.clone()),
                Content::List(list) => (list.name.clone(), list.description.clone()),
            };
            self.name_label.ok_mut()?.set_text(name.into());
            self.description_label.ok_mut()?.set_text(description.into());
        } {
            Ok(_) => {}
            Err::<_, BoxedError>(e) => log_error(e),
        }
    }

    #[func]
    fn on_gui_input(&mut self, event: Gd<InputEvent>) {
        if let Some(event) = event.try_cast::<InputEventMouseButton>() {
            if event.is_pressed() {
                let instance_id = self.base.instance_id();
                match event.get_button_index() {
                    MouseButton::MOUSE_BUTTON_LEFT => self.bus_left_click.broadcast(instance_id),
                    MouseButton::MOUSE_BUTTON_RIGHT => self.bus_right_click.broadcast(instance_id),
                    _ => {}
                }
            }
        }
    }

    pub fn set_content(&mut self, content: impl Into<Content>) {
        self.content = content.into();
        self.refresh_display();
    }
}

#[godot_api]
impl MarginContainerVirtual for ElementCard {
    fn init(base: Base<Self::Base>) -> Self {
        Self {
            base,

            // cached UI elements
            button: GdHolder::default(),
            name_label: GdHolder::default(),
            description_label: GdHolder::default(),

            // buses
            bus_left_click: BusType::None,
            bus_right_click: BusType::None,

            // state
            content: Content::Empty,
        }
    }

    fn ready(&mut self) {
        match try {
            self.add_theme_constant_override("margin_left".into(), 16);
            self.add_theme_constant_override("margin_top".into(), 16);
            self.add_theme_constant_override("margin_right".into(), 16);
            self.add_theme_constant_override("margin_bottom".into(), 16);

            let base = &self.base;
            self.button = GdHolder::from_path(base, "Button");
            self.button.ok_mut()?.connect(
                "gui_input".into(),
                base.callable("on_gui_input"),
                0,
            );
            self.name_label = GdHolder::from_path(base, "MarginContainer/VBoxContainer/NameLabel");
            self.description_label = GdHolder::from_path(base, "MarginContainer/VBoxContainer/DescriptionLabel");
        } {
            Ok(_) => {}
            Err::<_, BoxedError>(e) => log_error(e),
        }
    }
}
