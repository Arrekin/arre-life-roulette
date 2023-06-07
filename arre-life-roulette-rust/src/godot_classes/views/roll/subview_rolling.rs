use godot::engine::{Tween, VBoxContainer, VBoxContainerVirtual};
use godot::prelude::*;
use rand::prelude::IteratorRandom;
use rand::Rng;
use crate::errors::{ArreResult, ArreError, BoxedError};
use crate::godot_classes::element_card::ElementCard;
use crate::godot_classes::singletons::globals::Globals;
use crate::godot_classes::utils::get_singleton;
use crate::godot_classes::singletons::logger::log_error;
use crate::godot_classes::utils::{GdHolder};
use crate::godot_classes::views::roll::view_roll::{RollState, RollView};
use crate::item::{Item, item_get};

const ROLL_ANIMATION_DURATION: f64 = 10.; // seconds
const ROLL_FLIP_TIME: f64 = 0.2;

#[derive(GodotClass)]
#[class(base=VBoxContainer)]
pub struct RollRollingSubview {
    #[base]
    base: Base<VBoxContainer>,

    // cached internal UI elements
    pub roll_cards: [GdHolder<ElementCard>; 3],

    // cached external UI elements
    pub roll_view: GdHolder<RollView>,

    // state
    rng: rand::rngs::ThreadRng,
    is_animating: bool,
    animation_time: f64, // total time of the ongoing animation
    reduction_time: f64, // remove item from pool every `reduction_time` seconds
    time_till_reduction: f64, // counter till next reduction
    time_till_flip: [f64; 3], // counter till next card flip
   // card_flip_tween: [Gd<Tween>; 3],
    items_pool: Vec<Item>,

}

#[godot_api]
impl RollRollingSubview {

    pub fn animate(&mut self, eligible_items: Vec<Item>) {
        self.is_animating = true;
        self.animation_time = 0.;
        self.reduction_time = ROLL_ANIMATION_DURATION / eligible_items.len() as f64;
        self.time_till_reduction = self.reduction_time;
        self.time_till_flip = [ROLL_FLIP_TIME; 3];
        self.items_pool = eligible_items;
    }

    fn progress_animation(&mut self, delta: f64) -> ArreResult<()> {
        self.animation_time += delta;
        self.time_till_reduction -= delta;
        self.time_till_flip.iter_mut().for_each(|e| *e -= delta);
        if self.time_till_reduction < 0. {
            self.time_till_reduction = self.reduction_time;
            self.items_pool.remove(self.rng.gen_range(0..self.items_pool.len()));
        }
        if self.items_pool.len() == 1 {
            self.is_animating = false;
            let item = self.items_pool.pop().unwrap();

            let mut roll_view = self.roll_view.ok_mut()?.bind_mut();
            roll_view.roll_state_change_request(RollState::WorkAssigned{item});
        }
        for card_idx in 0..3 {
            if self.time_till_flip[card_idx] < 0. {
                self.time_till_flip[card_idx] = ROLL_FLIP_TIME + self.rng.gen::<f64>() / 10.;

                let mut card = self.roll_cards[card_idx].ok_mut()?;
                {
                    let mut card = card.bind_mut();
                    card.set_content(self.items_pool[self.rng.gen_range(0..self.items_pool.len())].clone());
                }
                let mut twin = self.base
                    .create_tween()
                    .ok_or(ArreError::CreateTweenFailed("ElementCard".into(), "RollRollingSubview::progress_animation".into()))?;
                twin.tween_property(card.share().upcast(), "modulate".into(), Color::from_rgba(1.0, 1.0, 0.0, 0.8).to_variant(), ROLL_FLIP_TIME / 1.25);
                twin.tween_property(card.share().upcast(), "modulate".into(), Color::WHITE.to_variant(), 0.);
            }
        }

        Ok(())
    }
}

#[godot_api]
impl VBoxContainerVirtual for RollRollingSubview {
    fn init(base: Base<Self::Base>) -> Self {
        Self {
            base,

            // cached internal UI elements
            roll_cards: [GdHolder::default(), GdHolder::default(), GdHolder::default()],

            // cached external UI elements
            roll_view: GdHolder::default(),

            // state
            rng: rand::thread_rng(),
            is_animating: false,
            animation_time: 0.,
            reduction_time: 0.,
            time_till_reduction: 0.,
            time_till_flip: [0., 0., 0.],
            //card_flip_tween: [Tween::new(), Tween::new(), Tween::new()],
            items_pool: Vec::new(),
        }
    }
    fn ready(&mut self) {
        match try {
            let base = &self.base;
            // cached internal UI elements
            self.roll_cards = [
                GdHolder::from_path(base, "MarginContainer/HBoxContainer/RollCard1"),
                GdHolder::from_path(base, "MarginContainer/HBoxContainer/RollCard2"),
                GdHolder::from_path(base, "MarginContainer/HBoxContainer/RollCard3"),
            ];
            // cached external UI elements
            // self.roll_view is set from RollView::ready()
        } {
            Ok(_) => {}
            Err::<_, BoxedError>(e) => log_error(e),
        }
    }

    fn process(&mut self, delta: f64) {
        match try {
            if self.is_animating { self.progress_animation(delta)?; }
        } {
            Ok(_) => {}
            Err::<_, BoxedError>(e) => log_error(e),
        }
    }
}
