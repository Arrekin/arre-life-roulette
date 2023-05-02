use godot::builtin::{Callable};
use godot::engine::{Control, ControlVirtual, Button, GridContainer};
use godot::engine::node::InternalMode;
use godot::engine::packed_scene::GenEditState;
use godot::prelude::*;
use crate::errors::{ArreError, ArreResult};
use crate::godot_classes::singletons::globals::{Globals};
use crate::godot_classes::resources::SELECTION_BUTTON_PREFAB;
use crate::godot_classes::selection_button::{Content, OnClickBehavior, SelectionButton};
use crate::godot_classes::singletons::logger::log_error;
use crate::godot_classes::singletons::signals::Signals;
use crate::godot_classes::utils::{GdHolder, get_singleton};
use crate::godot_classes::view_list_modify::ListModifyView;
use crate::godot_classes::view_roll::RollView;
use crate::list::{List, list_get_all};

#[derive(GodotClass)]
#[class(base=Control)]
pub struct ListsView {
    #[base]
    base: Base<Control>,

    // cached sub-scenes
    list_selection_button: Gd<PackedScene>,

    // cached UI elements
    list_add_button: GdHolder<Button>,
    list_roll_view: GdHolder<RollView>,
    list_modify_view: GdHolder<ListModifyView>,
    lists_grid: GdHolder<GridContainer>,
    lists_grid_elements: Vec<Gd<SelectionButton>>,

    // state
    lists: Vec<List>,
}

#[godot_api]
impl ListsView {
    #[func]
    fn on_list_add_button_up(&mut self) {
        match try {
            let mut view = self.list_modify_view.ok_mut()?.bind_mut();
            view.set_mode_add();
            view.set_visible(true);
        }: ArreResult<()> {
            Ok(_) => {},
            Err(e) => { log_error(e); }
        }
    }

    #[func]
    fn on_view_selected(&mut self) {
        self.refresh_lists_list();
        self.show();
    }

    #[func]
    fn refresh_lists_list(&mut self) {
        match try {
            let self_reference = self.base.share().cast::<Self>();
            // Get current list of all lists from the DB
            let globals = get_singleton::<Globals>("Globals");
            let connection = &globals.bind().connection;
            self.lists = list_get_all(connection)?;

            // Clear old and create a button for each item
            self.lists_grid_elements.drain(..).for_each(|mut list_btn| list_btn.bind_mut().queue_free());
            let new_lists = self.lists.iter().map(|list| {
                    let instance = self.list_selection_button
                        .instantiate(GenEditState::GEN_EDIT_STATE_DISABLED)
                        .ok_or(ArreError::NullGd("ListsView::refresh_lists_list::list_selection_button".into()))?;
                    self.lists_grid.ok_mut()?.add_child(instance.share(), false, InternalMode::INTERNAL_MODE_DISABLED);
                    let mut button = instance.cast::<SelectionButton>();
                    {
                        let mut button = button.bind_mut();
                        button.set_list(list.clone());
                        button.on_left_click_behavior = Some(Box::new(OnClickBehaviorShowListRollView {
                            parent: self_reference.share(),
                        }));
                        button.on_right_click_behavior = Some(Box::new(OnClickBehaviorShowListModifyView {
                            parent: self_reference.share(),
                        }));
                    }
                    Ok(button)
                }
            ).collect::<ArreResult<Vec<_>>>()?;
            self.lists_grid_elements.extend(new_lists);
        }: ArreResult<()> {
            Ok(_) => {},
            Err(e) => { log_error(e); }
        }
    }
}

#[godot_api]
impl ControlVirtual for ListsView {
    fn init(base: Base<Self::Base>) -> Self {
        Self {
            base,
            list_selection_button: load(SELECTION_BUTTON_PREFAB),

            list_add_button: GdHolder::default(),
            list_roll_view: GdHolder::default(),
            list_modify_view: GdHolder::default(),
            lists_grid: GdHolder::default(),
            lists_grid_elements: vec![],

            lists: vec![],
        }
    }
    fn ready(&mut self) {
        match try {
            let base = &self.base;
            self.list_add_button = GdHolder::from_path(base, "VBoxContainer/MarginContainer/ListAddDialogButton");
            self.list_add_button.ok_mut()?.connect(
                "button_up".into(),
                Callable::from_object_method(self.base.share(), "on_list_add_button_up"),
                0,
            );
            self.list_roll_view = GdHolder::from_path(base, "../../RollView");
            self.list_roll_view.ok_mut()?.bind_mut().connect(
                "dialog_closed".into(),
                Callable::from_object_method(self.base.share(), "refresh_lists_list"),
                0,
            );
            self.list_modify_view = GdHolder::from_path(base, "../../ListModifyView");
            self.list_modify_view.ok_mut()?.bind_mut().connect(
                "dialog_closed".into(),
                Callable::from_object_method(self.base.share(), "refresh_lists_list"),
                0,
            );
            self.lists_grid = GdHolder::from_path(base, "VBoxContainer/ListsListScrollContainer/ListsListGridContainer");

            if self.is_visible() {
                self.refresh_lists_list();
            }

            // Get singleton and connect to global signals(show / hide)
            let mut signals = get_singleton::<Signals>("Signals");
            {
                let mut signals = signals.bind_mut();
                signals.connect(
                    "item_view_tab_selected".into(),
                    Callable::from_object_method(self.base.share(), "hide"),
                    0,
                );
                signals.connect(
                    "list_view_tab_selected".into(),
                    Callable::from_object_method(self.base.share(), "on_view_selected"),
                    0,
                );
                signals.connect(
                    "tag_view_tab_selected".into(),
                    Callable::from_object_method(self.base.share(), "hide"),
                    0,
                );
            }
        }: ArreResult<()> {
            Ok(_) => {},
            Err(e) => { log_error(e); }
        }
    }
}

struct OnClickBehaviorShowListModifyView {
    pub parent: Gd<ListsView>,
}

impl OnClickBehavior for OnClickBehaviorShowListModifyView {
    fn on_click(&mut self, content: &Content) {
        if let Content::List(list) = content {
            let mut parent = self.parent.bind_mut();
            parent.list_modify_view.ok_mut().map(|view| {
                let mut view = view.bind_mut();
                view.set_mode_edit(list.clone());
                view.show();
            }).unwrap_or_else(|e| log_error(e));
        }
    }
}

struct OnClickBehaviorShowListRollView {
    pub parent: Gd<ListsView>,
}

impl OnClickBehavior for OnClickBehaviorShowListRollView {
    fn on_click(&mut self, content: &Content) {
        if let Content::List(list) = content {
            let mut parent = self.parent.bind_mut();
            parent.list_roll_view.ok_mut().map(|view| {
                let mut view = view.bind_mut();
                view.set_list(list.clone()).unwrap_or_else(|e| log_error(e));
                view.refresh_view();
                view.show();
            }).unwrap_or_else(|e| log_error(e));
        }
    }
}