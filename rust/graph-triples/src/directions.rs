use crate::Relation;

/// The direction to represent the flow of information from subject to object
pub enum Direction {
    From,
    To,
}

/// Get the the `Direction` for a `Relation`
pub fn direction(relation: &Relation) -> Direction {
    match relation {
        Relation::Assign(..) => Direction::To,
        Relation::Alter(..) => Direction::To,
        Relation::Convert(..) => Direction::To,
        Relation::Declare(..) => Direction::To,
        Relation::Embed => Direction::From,
        Relation::Import(..) => Direction::From,
        Relation::Include => Direction::From,
        Relation::Link => Direction::To,
        Relation::Read(..) => Direction::From,
        Relation::Use(..) => Direction::From,
        Relation::Write(..) => Direction::To,
        Relation::Require(..) => Direction::From,
    }
}
