use crate::{prelude::*, SuggestionStatus};

impl SuggestionStatus {
    pub fn to_keyword(&self) -> &str {
        match self {
            Self::Original => "original",
            Self::Accepted => "accept",
            Self::Rejected => "reject",
        }
    }

    pub fn from_keyword(keyword: &str) -> Result<Self> {
        match keyword {
            "orig" | "original" => Ok(Self::Original),
            "accept" | "accepted" => Ok(Self::Accepted),
            "reject" | "rejected" => Ok(Self::Rejected),
            _ => bail!("Unrecognized keyword for suggestion status: {keyword}"),
        }
    }
}
