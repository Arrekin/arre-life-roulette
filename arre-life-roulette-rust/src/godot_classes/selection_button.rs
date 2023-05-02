use godot::engine::{Button, ButtonVirtual, InputEvent, InputEventMouseButton};
use godot::engine::global::MouseButton;
use godot::prelude::*;
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
#[class(base=Button)]
pub struct SelectionButton {
    #[base]
    base: Base<Button>,

    // components
    pub on_left_click_behavior: Option<Box<dyn OnClickBehavior>>,
    pub on_right_click_behavior: Option<Box<dyn OnClickBehavior>>,

    // state
    pub content: Content,
}

#[godot_api]
impl SelectionButton {

    #[func]
    fn refresh_display(&mut self) {
        let (name, description) = match &self.content {
            Content::Empty => ("".into(), "".into()),
            Content::Item(item) => (item.name.clone(), item.description.clone()),
            Content::List(list) => (list.name.clone(), list.description.clone()),
        };
        self.set_text(name.into());
        self.set_tooltip_text(description.into());
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
impl ButtonVirtual for SelectionButton {
    fn init(base: Base<Self::Base>) -> Self {
        Self {
            base,

            // components
            on_left_click_behavior: None,
            on_right_click_behavior: None,

            content: Content::Empty,
        }
    }

    fn ready(&mut self) {
        let self_reference = self.base.share();
        self.connect(
            "gui_input".into(),
            Callable::from_object_method(self_reference, "on_gui_input"),
            0,
        );
    }
}
