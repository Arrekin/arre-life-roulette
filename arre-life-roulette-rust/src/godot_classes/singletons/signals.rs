use godot::engine::{Node, NodeVirtual};
use godot::prelude::*;

#[derive(GodotClass)]
#[class(base=Node)]
pub struct Signals {
    #[base]
    base: Base<Node>,
}

#[godot_api]
impl Signals {
    // signals
    #[signal]
    fn item_view_tab_selected();
    #[signal]
    fn list_view_tab_selected();
    #[signal]
    fn tag_view_tab_selected();
}

#[godot_api]
impl NodeVirtual for Signals {
    fn init(base: Base<Self::Base>) -> Self {
        Self {
            base,
        }
    }
}