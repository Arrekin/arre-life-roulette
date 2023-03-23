use crate::utils::Id;

pub type ItemTagId = Id<ItemTag>;

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct ItemTag {
    pub id: ItemTagId,
    pub name: String,
    pub description: Option<String>,
}