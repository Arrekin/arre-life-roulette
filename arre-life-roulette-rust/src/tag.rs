use crate::utils::Id;

pub type TagId = Id<Tag>;

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Tag {
    pub id: Option<TagId>,
    pub name: String,
    pub color: String,
}