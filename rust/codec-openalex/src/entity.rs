use codec::{common::serde::Deserialize, schema::Node};

use crate::{
    author::Author, funder::Funder, institution::Institution, publisher::Publisher, work::Work,
};

/// An OpenAlex API entity
///
/// See https://docs.openalex.org/api-entities/entities-overview
///
/// Note: This enum is kept for potential future use but is no longer used
/// in the main codec logic, which now uses ID-based type detection.
#[derive(Deserialize)]
#[serde(crate = "codec::common::serde")]
pub enum Entity {
    Author(Author),
    Funder(Funder),
    Publisher(Publisher),
    Institution(Institution),
    Work(Work),
}

impl From<Entity> for Node {
    fn from(entity: Entity) -> Self {
        match entity {
            Entity::Author(entity) => Node::Person(entity.into()),
            Entity::Funder(entity) => Node::Organization(entity.into()),
            Entity::Publisher(entity) => Node::Organization(entity.into()),
            Entity::Institution(entity) => Node::Organization(entity.into()),
            Entity::Work(entity) => Node::CreativeWork(entity.into()),
        }
    }
}
