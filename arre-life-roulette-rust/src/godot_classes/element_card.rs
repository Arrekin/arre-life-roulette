use godot::engine::{MarginContainer, InputEvent, InputEventMouseButton, MarginContainerVirtual, Label, Button};
use godot::engine::global::MouseButton;
use godot::prelude::*;
use crate::errors::ArreResult;
use crate::godot_classes::singletons::logger::log_error;
use crate::godot_classes::utils::GdHolder;
use crate::item::Item;
use crate::list::List;

#[derive(Clone)]
pub enum Content {
    Empty,
    Item(Item),
    List(List),
}

pub trait OnClickBehavior {
    fn on_click(&mut self, content: &Content);
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

    // components
    pub on_left_click_behavior: Option<Box<dyn OnClickBehavior>>,
    pub on_right_click_behavior: Option<Box<dyn OnClickBehavior>>,

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
        }: ArreResult<()> {
            Ok(_) => {}
            Err(e) => log_error(e),
        }
    }

    #[func]
    fn on_left_button_up(&mut self) {
        self.on_left_click_behavior.as_mut().map(|behavior| {
            behavior.on_click(&self.content);
        });
    }

    #[func]
    fn on_right_button_up(&mut self) {
        self.on_right_click_behavior.as_mut().map(|behavior| {
            behavior.on_click(&self.content);
        });
    }

    #[func]
    fn on_gui_input(&mut self, event: Gd<InputEvent>) {
        if let Some(event) = event.try_cast::<InputEventMouseButton>() {
            if event.is_pressed() {
                match event.get_button_index() {
                    MouseButton::MOUSE_BUTTON_LEFT => self.on_left_button_up(),
                    MouseButton::MOUSE_BUTTON_RIGHT => self.on_right_button_up(),
                    _ => {}
                }
            }
        }
    }

    pub fn set_item(&mut self, item: Item) {
        self.content = Content::Item(item);
        self.refresh_display();
    }

    pub fn set_list(&mut self, list: List) {
        self.content = Content::List(list);
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

            // components
            on_left_click_behavior: None,
            on_right_click_behavior: None,

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
        }: ArreResult<()> {
            Ok(_) => {}
            Err(e) => log_error(e),
        }
    }
}
