use godot::engine::{FlowContainer, FlowContainerVirtual, PackedScene};
use godot::engine::node::InternalMode;
use godot::engine::packed_scene::GenEditState;
use godot::prelude::*;
use crate::errors::{ArreError, ArreResult, BoxedError};
use crate::godot_classes::element_card::{Content, ElementCard};
use crate::godot_classes::resources::ELEMENT_CARD_PREFAB;
use crate::godot_classes::singletons::buses::BusType;
use crate::godot_classes::singletons::logger::log_error;

#[derive(GodotClass)]
#[class(base=FlowContainer)]
pub struct CardsFlowContainer {
    #[base]
    base: Base<FlowContainer>,

    // buses
    pub bus_card_left_click: BusType<InstanceId>,
    pub bus_card_right_click: BusType<InstanceId>,

    // cached sub-scenes
    element_card_prefab: Gd<PackedScene>,

    // cached internal UI elements
    pub item_cards: Vec<Gd<ElementCard>>,
}

#[godot_api]
impl CardsFlowContainer {
    pub fn set_cards(&mut self, contents: Vec<impl Into<Content>>)
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
                        card.bus_left_click = self.bus_card_left_click.cloned()?;
                        card.bus_right_click = self.bus_card_right_click.cloned()?;
                    }
                    Ok(card)
                }
            ).collect::<ArreResult<Vec<_>>>()?;
            self.item_cards.extend(new_cards);
        } {
            Ok(_) => {},
            Err::<_, BoxedError>(e) => log_error(e)
        }
    }
}

#[godot_api]
impl FlowContainerVirtual for CardsFlowContainer {
    fn init(base: Base<Self::Base>) -> Self {
        Self {
            base,

            // buses
            bus_card_left_click: BusType::new_shared(),
            bus_card_right_click: BusType::new_shared(),

            // cached sub-scenes
            element_card_prefab: load(ELEMENT_CARD_PREFAB),

            // cached internal UI elements
            item_cards: vec![],
        }
    }
}