use godot::engine::{Button, ButtonVirtual};
use godot::prelude::*;
use crate::item::Item;
use crate::list::List;

#[derive(Clone)]
pub enum Content {
    Empty,
    Item(Item),
    List(List),
}

#[derive(GodotClass)]
#[class(base=Button)]
pub struct SelectionButton {
    #[base]
    base: Base<Button>,

    // components
    pub on_click_behavior: Option<Box<dyn OnClickBehavior>>,

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
    fn on_button_up(&mut self) {
        let behavior = self.on_click_behavior.as_mut().unwrap();
        behavior.on_click(&self.content);
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

            on_click_behavior: None,

            content: Content::Empty,
        }
    }

    fn ready(&mut self) {
        let self_reference = self.base.share();
        self.connect(
            "button_up".into(),
            Callable::from_object_method(self_reference, "on_button_up"),
            0,
        );
    }
}

pub trait OnClickBehavior {
    fn on_click(&mut self, content: &Content);
}



