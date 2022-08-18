use crate::Relation;

/// The direction to represent the flow of information from subject to object
pub enum Direction {
    From,
    To,
}

/// Get the the `Direction` for a `Relation`
pub fn direction(relation: &Relation) -> Direction {
    match relation {
        Relation::Assigns(..) => Direction::To,
        Relation::Alters(..) => Direction::To,
        Relation::Converts(..) => Direction::To,
        Relation::Declares(..) => Direction::To,
        Relation::Embed => Direction::From,
        Relation::Imports(..) => Direction::From,
        Relation::Includes => Direction::From,
        Relation::Calls => Direction::From,
        Relation::Links => Direction::To,
        Relation::Reads(..) => Direction::From,
        Relation::Uses(..) => Direction::From,
        Relation::Writes(..) => Direction::To,
        Relation::Requires(..) => Direction::From,
    }
}
