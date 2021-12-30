/// A subject-relation-object triple
pub type Triple = (Resource, Relation, Resource);

/// A set of triples
pub type Triples = Vec<Triple>;

/// A relation-object pair
pub type Pair = (Relation, Resource);

/// A set of pairs
pub type Pairs = Vec<Pair>;

/// A set of subjects and their relation-object pairs
///
/// Like `Triples` but grouped by subject.
pub type Relations = Vec<(Resource, Pairs)>;

pub mod resources;
pub use resources::Resource;
pub use resources::ResourceId;
pub use resources::ResourceDependencies;

pub mod relations;
pub use relations::Relation;

mod directions;
pub use directions::{direction, Direction};
