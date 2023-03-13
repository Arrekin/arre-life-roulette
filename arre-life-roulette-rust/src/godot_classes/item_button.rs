use godot::engine::{Control,  Button};
use godot::obj::EngineClass;
use godot::prelude::*;
use crate::godot_classes::view_items::ItemsView;
use crate::item::Item;

#[derive(GodotClass)]
#[class(base=Button)]
pub struct ItemSelectionButton {
    #[base]
    base: Base<Button>,

    // state
    pub item: Item,
}

#[godot_api]
impl ItemSelectionButton {

    #[func]
    fn refresh_display(&mut self) {
        let name = self.item.name.clone();
        self.set_text(name.into());
        let description = self.item.description.clone();
        self.set_tooltip_text(description.into());
    }

    #[func]
    fn on_button_up(&mut self) {
        // from the container to top view level
        let mut parent = self.base.try_get_node_as::<ItemsView>("../../../../ItemsView");
        parent.as_mut().map(|parent| {
            let mut parent = parent.bind_mut();
            parent.item_modify_view.as_mut().map(|view| {
                let mut view = view.bind_mut();
                view.set_mode_edit(self.item.clone());
                view.show();
            });
        });
    }

    pub fn set_item(&mut self, item: Item) {
        self.item = item;
        self.refresh_display();
    }
}

#[godot_api]
impl GodotExt for ItemSelectionButton {
    fn init(base: Base<Self::Base>) -> Self {
        Self {
            base,

            item: Item::default(),
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


