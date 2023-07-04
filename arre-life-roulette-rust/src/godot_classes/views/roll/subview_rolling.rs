use godot::engine::{ScrollContainer, VBoxContainer, VBoxContainerVirtual};
use godot::prelude::*;
use rand::prelude::SliceRandom;
use rand::Rng;
use crate::errors::{ArreResult, ArreError, BoxedError};
use crate::godot_classes::element_card::ElementCard;
use crate::godot_classes::resources::ELEMENT_CARD_PREFAB;
use crate::godot_classes::singletons::logger::log_error;
use crate::godot_classes::utils::{GdHolder};
use crate::godot_classes::views::roll::view_roll::{RollState, RollView};
use crate::item::{Item};

const ROLL_ANIMATION_DURATION: f64 = 10.; // seconds
const ROLL_CARDS_ROWS: usize = 100;

#[derive(GodotClass)]
#[class(base=VBoxContainer)]
pub struct RollRollingSubview {
    #[base]
    base: Base<VBoxContainer>,

    // cached sub-scenes
    element_card_prefab: Gd<PackedScene>,

    // cached internal UI elements
    pub scrolls: [GdHolder<ScrollContainer>; 3],

    // cached external UI elements
    pub roll_view: GdHolder<RollView>,

    // state
    rng: rand::rngs::ThreadRng,
    is_animating: bool,
    animation_time: f64, // total time of the ongoing animation
    chosen_item: Item,

}

#[godot_api]
impl RollRollingSubview {

    pub fn animate(&mut self, mut eligible_items: Vec<Item>) -> ArreResult<()> {
        self.is_animating = true;
        self.animation_time = 0.;

        // Animation takes ROLL_ANIMATION_DURATION seconds, during which we display ROLL_CARDS_ROWS cards.
        // Eligible cards are slowly reduced and so later rows must respect this reduction.
        let mut cards = [Array::new(), Array::new(), Array::new()];
        for scroll_idx in 0..3 {
            let scroll = self.scrolls[scroll_idx].ok_mut()?;
            let vbox = GdHolder::<VBoxContainer>::from_path(&self.base, format!("{}/VBoxContainer", scroll.get_path()));
            cards[scroll_idx] = vbox.ok()?.get_children();
        }
        for row in 0..ROLL_CARDS_ROWS {
            if eligible_items.len() > ROLL_CARDS_ROWS - row {
                eligible_items.remove(self.rng.gen_range(0..eligible_items.len()));
            } else if row == ROLL_CARDS_ROWS - 1 && eligible_items.len() > 1 {
                while eligible_items.len() > 1 {
                    eligible_items.remove(self.rng.gen_range(0..eligible_items.len()));
                }
            }
            for scroll_idx in 0..3 {
                let mut card = GdHolder::<ElementCard>::from_gd(cards[scroll_idx].get(row));
                let mut card = card.ok_mut()?.bind_mut();
                let card_item = eligible_items.choose(&mut self.rng).ok_or(ArreError::UnexpectedNone("RollRollingSubview::animate".to_string()))?;
                card.set_content(card_item.clone());
            }
        }
        self.chosen_item = eligible_items.pop().ok_or(ArreError::UnexpectedNone("RollRollingSubview::animate".to_string()))?;
        Ok(())
    }

    fn progress_animation(&mut self, delta: f64) -> ArreResult<()> {
        self.animation_time += delta;
        if self.animation_time >= ROLL_ANIMATION_DURATION {
            self.is_animating = false;
            let mut roll_view = self.roll_view.ok_mut()?.bind_mut();
            roll_view.roll_state_change_request(RollState::WorkAssigned{item: self.chosen_item.clone()});
        }
        for scroll_idx in 0..3 {
            let scroll = self.scrolls[scroll_idx].ok_mut()?;
            let mut vbox = GdHolder::<VBoxContainer>::from_path(&self.base, format!("{}/VBoxContainer", scroll.get_path()));
            let vbox = vbox.ok_mut()?;

            // Apply a power to time before logarithm to slow down the growth rate
            let adjusted_time = (self.animation_time / ROLL_ANIMATION_DURATION).powf(0.20) * ROLL_ANIMATION_DURATION;
            // We use a scaling factor to make sure we reach max_position when time is max_time.
            let scale_factor = (vbox.get_size().y as f64 / ROLL_ANIMATION_DURATION.log10()).max(0.);
            let new_scroll_position = scale_factor * adjusted_time.log10();
            scroll.set_v_scroll(new_scroll_position.round() as i32);
        }
        Ok(())
    }
}

#[godot_api]
impl VBoxContainerVirtual for RollRollingSubview {
    fn init(base: Base<Self::Base>) -> Self {
        Self {
            base,

            element_card_prefab: load(ELEMENT_CARD_PREFAB),

            // cached internal UI elements
            scrolls: [GdHolder::default(), GdHolder::default(), GdHolder::default()],

            // cached external UI elements
            roll_view: GdHolder::default(),

            // state
            rng: rand::thread_rng(),
            is_animating: false,
            animation_time: 0.,
            chosen_item: Item::default(),
        }
    }
    fn ready(&mut self) {
        match try {
            let base = &self.base;
            // cached internal UI elements
            self.scrolls = [
                GdHolder::from_path(base, "MarginContainer/HBoxContainer/ScrollContainer1"),
                GdHolder::from_path(base, "MarginContainer/HBoxContainer/ScrollContainer2"),
                GdHolder::from_path(base, "MarginContainer/HBoxContainer/ScrollContainer3"),
            ];
            for scroll in self.scrolls.iter_mut() {
                let scroll = scroll.ok_mut()?;
                let mut vbox = GdHolder::<VBoxContainer>::from_path(&self.base, format!("{}/VBoxContainer", scroll.get_path()));
                let vbox = vbox.ok_mut()?;
                for _ in 0..ROLL_CARDS_ROWS {
                    let new_card = self.element_card_prefab
                        .try_instantiate_as::<ElementCard>()
                        .ok_or(ArreError::InstantiateFailed("ElementCard".into(), "RollRollingSubview::ready".into()))?;
                    vbox.add_child(new_card.share().upcast());
                }

            }
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
