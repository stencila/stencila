use crate::Relation;

/// The direction to represent the flow of information from subject to object
pub enum Direction {
    From,
    To,
}

/// Get the the `Direction` for a `Relation`
pub fn direction(relation: &Relation) -> Direction {
    match relation {
        Relation::Alters(..) => Direction::To,
        Relation::Assigns(..) => Direction::To,
        Relation::Calls => Direction::From,
        Relation::Converts(..) => Direction::To,
        Relation::Declares(..) => Direction::To,
        Relation::Derives => Direction::From,
        Relation::Embeds => Direction::From,
        Relation::Imports(..) => Direction::From,
        Relation::Includes => Direction::From,
        Relation::Links => Direction::To,
        Relation::On(..) => Direction::From,
        Relation::Reads(..) => Direction::From,
        Relation::Requires(..) => Direction::From,
        Relation::Uses(..) => Direction::From,
        Relation::Writes(..) => Direction::To,
    }
}
