use godot::builtin::{Callable};
use godot::engine::{Control, ControlVirtual, Button, GridContainer};
use godot::engine::node::InternalMode;
use godot::engine::packed_scene::GenEditState;
use godot::prelude::*;
use crate::errors::{ArreError, ArreResult};
use crate::godot_classes::singletons::globals::{Globals};
use crate::godot_classes::resources::SELECTION_BUTTON_PREFAB;
use crate::godot_classes::selection_button::{SelectionButton, OnClickBehavior, Content};
use crate::godot_classes::singletons::logger::log_error;
use crate::godot_classes::singletons::signals::Signals;
use crate::godot_classes::utils::{GdHolder, get_singleton};
use crate::godot_classes::view_item_modify::ItemModifyView;
use crate::item::{Item, item_get_all};

#[derive(GodotClass)]
#[class(base=Control)]
pub struct ItemsView {
    #[base]
    base: Base<Control>,

    // cached sub-scenes
    item_selection_button: Gd<PackedScene>,

    // cached UI elements
    pub item_add_button: GdHolder<Button>,
    pub item_modify_view: GdHolder<ItemModifyView>,
    pub items_grid: GdHolder<GridContainer>,
    pub items_grid_elements: Vec<Gd<SelectionButton>>,

    // state
    items: Vec<Item>,
}

#[godot_api]
impl ItemsView {
    #[func]
    fn on_item_add_button_up(&mut self) {
        match try {
            self.item_modify_view.ok_mut().map(|view| {
                let mut view = view.bind_mut();
                view.set_mode_add();
                view.show();
            })?;
        }: ArreResult<()> {
            Ok(_) => {},
            Err(e) => { log_error(e); }
        }
    }

    #[func]
    fn on_view_selected(&mut self) {
        self.refresh_items_list();
        self.show();
    }

    #[func]
    fn refresh_items_list(&mut self) {
        match try {
            let self_reference = self.base.share().cast::<Self>();
            // Get current list of all items from the DB
            let globals = get_singleton::<Globals>("Globals");
            let connection = &globals.bind().connection;
            self.items = item_get_all(connection)?;

            // Clear old and create a button for each item
            self.items_grid_elements.drain(..).for_each(|mut item_btn| item_btn.bind_mut().queue_free());
            let new_items = self.items.iter().map(
                |item| {
                    let instance = self.item_selection_button
                        .instantiate(GenEditState::GEN_EDIT_STATE_DISABLED)
                        .ok_or(ArreError::InstantiateFailed(
                                SELECTION_BUTTON_PREFAB.into(),
                                "ItemsView::refresh_items_list".into())
                            )?;
                    self.items_grid.ok_mut()?.add_child(instance.share(), false, InternalMode::INTERNAL_MODE_DISABLED);
                    let mut button = instance.cast::<SelectionButton>();
                    {
                        let mut button = button.bind_mut();
                        button.set_item(item.clone());
                        button.on_left_click_behavior = Some(Box::new(OnClickBehaviorShowItemModifyView {
                            parent: self_reference.share(),
                        }));
                    }
                    Ok(button)
                }
            ).collect::<ArreResult<Vec<_>>>()?;
            self.items_grid_elements.extend(new_items);
        }: ArreResult<()> {
            Ok(_) => {},
            Err(e) => { log_error(e); }
        }
    }
}

#[godot_api]
impl ControlVirtual for ItemsView {
    fn init(base: Base<Self::Base>) -> Self {
        Self {
            base,
            item_selection_button: load(SELECTION_BUTTON_PREFAB),

            item_add_button: GdHolder::default(),
            item_modify_view: GdHolder::default(),
            items_grid: GdHolder::default(),
            items_grid_elements: vec![],

            items: vec![],
        }
    }
    fn ready(&mut self) {
        match try {
            let base = &self.base;
            self.item_add_button = GdHolder::from_path(base, "VBoxContainer/MarginContainer/ItemAddDialogButton");
            self.item_add_button.ok_mut()?.connect(
                "button_up".into(),
                Callable::from_object_method(self.base.share(), "on_item_add_button_up"),
                0,
            );
            self.item_modify_view = GdHolder::from_path(base, "../../ItemModifyView");
            self.item_modify_view.ok_mut()?.bind_mut().connect(
                "dialog_closed".into(),
                Callable::from_object_method(self.base.share(), "refresh_items_list"),
                0,
            );
            self.items_grid = GdHolder::from_path(base,"VBoxContainer/ItemsListScrollContainer/ItemsListGridContainer");

            if self.is_visible() {
                self.refresh_items_list();
            }

            // Get singleton and connect to global signals(show / hide)
            let mut signals = get_singleton::<Signals>("Signals");
            {
                let mut signals = signals.bind_mut();
                signals.connect(
                    "item_view_tab_selected".into(),
                    Callable::from_object_method(self.base.share(), "on_view_selected"),
                    0,
                );
                signals.connect(
                    "list_view_tab_selected".into(),
                    Callable::from_object_method(self.base.share(), "hide"),
                    0,
                );
                signals.connect(
                    "tag_view_tab_selected".into(),
                    Callable::from_object_method(self.base.share(), "hide"),
                    0,
                );
            }
        }: ArreResult<()> {
            Ok(_) => {}
            Err(e) => log_error(e),
        }
    }
}

struct OnClickBehaviorShowItemModifyView {
    pub parent: Gd<ItemsView>,
}

impl OnClickBehavior for OnClickBehaviorShowItemModifyView {
    fn on_click(&mut self, content: &Content) {
        if let Content::Item(item) = content {
            let mut parent = self.parent.bind_mut();
            parent.item_modify_view.ok_mut().map(|view| {
                let mut view = view.bind_mut();
                view.set_mode_edit(item.clone());
                view.show();
            }).unwrap_or_else(|e| log_error(e));
        }
    }
}