use godot::engine::{FlowContainer, FlowContainerVirtual, PackedScene};
use godot::engine::node::InternalMode;
use godot::engine::packed_scene::GenEditState;
use godot::prelude::*;
use crate::errors::{ArreError, ArreResult};
use crate::godot_classes::element_card::{Content, ElementCard};
use crate::godot_classes::resources::ELEMENT_CARD_PREFAB;
use crate::godot_classes::singletons::logger::log_error;
use crate::item::Item;

#[derive(GodotClass)]
#[class(base=FlowContainer)]
pub struct CardsFlowContainer {
    #[base]
    base: Base<FlowContainer>,

    // cached sub-scenes
    element_card_prefab: Gd<PackedScene>,

    // cached internal UI elements
    pub item_cards: Vec<Gd<ElementCard>>,
}

#[godot_api]
impl CardsFlowContainer {
    pub fn set_cards<F>(
        &mut self,
        contents: Vec<impl Into<Content>>,
        card_configure: F,
    )
    where F: Fn(GdMut<ElementCard>)
    {
        match try {
            // Clear old cards and then create a new card for each item
            self.item_cards.drain(..).for_each(|mut card| card.bind_mut().queue_free());
            let new_cards = contents.into_iter().map(
                |content| {
                    let instance = self.element_card_prefab
                        .instantiate(GenEditState::GEN_EDIT_STATE_DISABLED)
                        .ok_or(ArreError::InstantiateFailed(
                            ELEMENT_CARD_PREFAB.into(),
                            "CardsFlowContainer::set_cards".into()
                        ))?;
                    self.add_child(instance.share(), false, InternalMode::INTERNAL_MODE_DISABLED);
                    let mut card = instance.try_cast::<ElementCard>()
                        .ok_or(ArreError::CastFailed("ElementCard".into(), "CardsFlowContainer::set_cards".into()))?;
                    {
                        let mut card = card.bind_mut();
                        card.set_content(content);
                        (card_configure)(card);
                    }
                    Ok(card)
                }
            ).collect::<ArreResult<Vec<_>>>()?;
            self.item_cards.extend(new_cards);
        }: ArreResult<()> {
            Ok(_) => {},
            Err(e) => { log_error(e); }
        }
    }
}

#[godot_api]
impl FlowContainerVirtual for CardsFlowContainer {
    fn init(base: Base<Self::Base>) -> Self {
        Self {
            base,
            element_card_prefab: load(ELEMENT_CARD_PREFAB),

            // cached internal UI elements
            item_cards: vec![],
        }
    }
}