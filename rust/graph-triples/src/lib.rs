/// A subject-relation-object triple
pub type Triple = (Resource, Relation, Resource);

/// A set of triples
pub type Triples = Vec<Triple>;

/// A relation-object pair
pub type Pair = (Relation, Resource);

/// A set of pairs
pub type Pairs = Vec<Pair>;

pub mod resources;
pub use resources::Resource;

pub mod relations;
pub use relations::Relation;

mod directions;
pub use directions::{direction, Direction};

mod schemas;
pub use schemas::schemas;
