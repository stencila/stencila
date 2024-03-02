use std::cmp::Ordering;

use crate::ExecutionStatus;

impl PartialOrd for ExecutionStatus {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        use ExecutionStatus::*;
        use Ordering::*;
        Some(match self {
            Interrupted => Greater,
            Exceptions => match other {
                Interrupted => Less,
                _ => Greater,
            },
            Errors => match other {
                Interrupted | Exceptions => Less,
                _ => Greater,
            },
            Warnings => match other {
                Interrupted | Exceptions | Errors => Less,
                _ => Greater,
            },
            Succeeded => match other {
                Interrupted | Exceptions | Errors | Warnings => Less,
                _ => Greater,
            },
            Running => match other {
                Interrupted | Exceptions | Errors | Warnings | Succeeded => Less,
                _ => Greater,
            },
            _ => return None,
        })
    }
}
