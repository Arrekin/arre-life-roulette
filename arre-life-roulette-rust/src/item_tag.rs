use crate::utils::Id;

pub type ItemTagId = Id<ItemTag>;

#[derive(Debug, Clone)]
pub struct ItemTag {
    pub id: ItemTagId,
    pub name: String,
    pub description: Option<String>,
}