/// A subject-relation-object triple
pub type Triple = (Resource, Relation, Resource);

pub mod resources;
pub use resources::Resource;

pub mod relations;
pub use relations::Relation;

mod directions;
pub use directions::{direction, Direction};
