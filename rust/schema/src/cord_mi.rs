//! Module for creating, modifying and displaying Machine Influence bytes
//! for `Cord`s
//!
//! The eight bits represent:
//!
//! 0: 0 = human written, 1 = machine generated
//! 1: 1 = human edited
//! 2: 1 = machine edited
//! 3 + 4: the number or verifications 0, 1, 2, or 3
//! 5, 6, 7: the verified 0 = human, 1 = machine (latest first)

/// Display the Machine Influence byte as a string
pub fn display(mi: u8) -> String {
    let mut display = String::new();

    // Initial writer: human or machine
    if mi & 0b00000001 == 0 {
        display.push_str("Hw");
    } else {
        display.push_str("Mw");
    }

    // Last editor: human or machine
    if mi & 0b00000010 != 0 {
        display.push_str("He");
    } else if mi & 0b00000100 != 0 {
        display.push_str("Me");
    }

    // Human verifiers
    let human_verifiers = (mi & 0b00111000) >> 3;
    if human_verifiers > 0 {
        if human_verifiers > 1 {
            display.push_str(&human_verifiers.to_string());
        }
        display.push_str("Hv");
    }

    // Machine verifiers
    let machine_verifiers = (mi & 0b11000000) >> 6;
    if machine_verifiers > 0 {
        if machine_verifiers > 1 {
            display.push_str(&machine_verifiers.to_string());
        }
        display.push_str("Mv");
    }

    display
}

/// The Machine Influence Category
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Mic {
    // Human only
    HwHeHv = 0,
    HwHe = 1,
    HwHv = 2,
    Hw = 3,

    // TODO: the ordering and grouping of the following
    // needs to be reviewed

    // Human written, machine verified
    HwMv = 4,

    // Machine written, human edited
    MwHeHv = 5,
    MwHe = 6,
    MwHeMv = 7,

    // Human written, machine edited
    HwMeHv = 8,
    HwMe = 9,
    HwMeMv = 10,

    // Machine written, human verified
    MwHv = 11,
    MwMeHv = 12,

    // Machine only
    Mw = 13,
    MwMv = 14,
    MwMe = 15,
    MwMeMv = 16,
}

/// Get the category of Machine Influence
pub fn category(mi: u8) -> Mic {
    let hw = mi & 0b00000001 == 0;

    let he = mi & 0b00000010 != 0;
    let me = mi & 0b00000100 != 0;

    let hv = ((mi & 0b00111000) >> 3) > 0;
    let mv = ((mi & 0b11000000) >> 6) > 0;

    use Mic::*;
    if hw {
        if he {
            if hv {
                HwHeHv
            } else if mv {
                HwMv
            } else {
                HwHe
            }
        } else if me {
            if hv {
                HwMeHv
            } else if mv {
                HwMeMv
            } else {
                HwMe
            }
        } else {
            if hv {
                HwHv
            } else if mv {
                HwMv
            } else {
                Hw
            }
        }
    } else {
        if he {
            if hv {
                MwHeHv
            } else if mv {
                MwHeMv
            } else {
                MwHe
            }
        } else if me {
            if hv {
                MwMeHv
            } else if mv {
                MwMeMv
            } else {
                MwMe
            }
        } else {
            if hv {
                MwHv
            } else if mv {
                MwMv
            } else {
                Mw
            }
        }
    }
}

/// The run was written bStringy a human
pub fn human_written() -> u8 {
    0b00000000
}

/// The run was written by a machine
pub fn machine_written() -> u8 {
    0b00000001
}

/// The run was edited by a human
///
/// Set the "human edited" bit and clear the "machine edited" bit
/// and all verification bits.
pub fn human_edited(mi: u8) -> u8 {
    (mi | 0b00000010) & 0b00000011
}

/// The run was edited by a machine
///
/// Set the "machine edited" bit and clear the "human edited" bit
/// and all verification bits.
pub fn machine_edited(mi: u8) -> u8 {
    (mi | 0b00000100) & 0b00000101
}

/// The run was verified by a human
///
/// Increment the three "human verifications" bits.
pub fn human_verified(mi: u8) -> u8 {
    let verifiers = (mi & 0b00111000) >> 3;
    if verifiers < 7 {
        (mi & 0b11000111) | ((verifiers + 1) << 3)
    } else {
        mi
    }
}

/// The run was verified by a machine
///
/// Increment the two "machine verification" bits.
pub fn machine_verified(mi: u8) -> u8 {
    let verifiers = (mi & 0b11000000) >> 6;
    if verifiers < 3 {
        (mi & 0b00111111) | ((verifiers + 1) << 6)
    } else {
        mi
    }
}
