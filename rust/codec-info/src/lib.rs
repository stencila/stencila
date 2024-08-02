mod losses;
mod mapping;
mod messages;
mod poshmap;
mod positions;
mod shifter;

pub use losses::*;
pub use mapping::*;
pub use messages::*;
pub use poshmap::*;
pub use positions::*;
pub use shifter::*;

/// Information which may be returned when decoding content to a node
#[derive(Default)]
pub struct DecodeInfo {
    /// Any messages generated while decoding
    pub messages: Messages,

    /// The losses when the decoding content to a node
    pub losses: Losses,

    /// The mapping between content locations and the decoded node and its children
    pub mapping: Mapping,
}

impl DecodeInfo {
    /// Create an empty set on decoding information
    pub fn none() -> Self {
        Self {
            messages: Messages::none(),
            losses: Losses::none(),
            mapping: Mapping::none(),
        }
    }
}

/// Information which may be returned when encoding a node to content
#[derive(Default)]

pub struct EncodeInfo {
    /// The losses when encoding the node to content
    pub losses: Losses,

    /// The mapping between content location and the node and its children
    pub mapping: Mapping,
}

impl EncodeInfo {
    /// Create an empty set on encodings information
    pub fn none() -> Self {
        Self {
            losses: Losses::none(),
            mapping: Mapping::none(),
        }
    }
}
