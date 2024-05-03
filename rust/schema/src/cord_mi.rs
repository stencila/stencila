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

    // Verifiers
    let verifiers = (mi & 0b00011000) >> 3;
    if verifiers > 0 {
        display.push_str(if mi & 0b00100000 == 0 { "Hv" } else { "Mv" });
        if verifiers > 1 {
            display.push_str(if mi & 0b01000000 == 0 { "Hv" } else { "Mv" });
        }
        if verifiers > 2 {
            display.push_str(if mi & 0b10000000 == 0 { "Hv" } else { "Mv" });
        }
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

    let mut hv = false;
    let mut mv = false;
    let verifiers = (mi & 0b00011000) >> 3;
    if verifiers > 0 {
        if mi & 0b00100000 == 0 {
            hv = true;
        } else {
            mv = true;
        };

        if verifiers > 1 {
            if mi & 0b01000000 == 0 {
                hv = true;
            } else {
                mv = true;
            };
        }

        if verifiers > 2 {
            if mi & 0b10000000 == 0 {
                hv = true;
            } else {
                mv = true;
            };
        }
    }

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
/// Set the 'human edited' bit and clear 'machine edited' and
/// all verification bits.
pub fn human_edited(mi: u8) -> u8 {
    (mi | 0b00000010) & 0b00000011
}

/// The run was edited by a machine
///
/// Set the 'machine edited' bit and clear 'human edited' and
/// all verification bits.
pub fn machine_edited(mi: u8) -> u8 {
    (mi | 0b00000100) & 0b00000101
}

/// The run was verified by a human
///
/// Increment the two 'verification count' bits
pub fn human_verified(mi: u8) -> u8 {
    verified(mi)
}

/// The run was verified by a machine
///
/// Increment the two 'verification count' bits and set the
/// appropriate 'verifier' bit to indicate machine verified.
pub fn machine_verified(mi: u8) -> u8 {
    verified(mi) | 0b00100000
}

/// The run was verified by a machine
///
/// Increment the two 'verification count' bits and set the
/// appropriate 'verifier' bit.
fn verified(mi: u8) -> u8 {
    // Get current number of verifiers
    let verifiers = (mi & 0b00011000) >> 3;

    let mi = if verifiers < 3 {
        // Increment the number of verifiers
        (mi & 0b11100111) | ((verifiers + 1) << 3)
    } else {
        mi
    };

    // Mask to isolate verifier bits
    let mask = 0b11100000;

    // Extract the verifiers
    let bits = mi & mask;

    // Shift these bits to the left by 1
    let shifted_bits = (bits << 1) & 0b11000000; // Mask to discard overflow beyond the 7th bit

    // Clear the original bit positions in the value
    let mi_cleared = mi & !mask;

    // Set the new positions of the shifted bits
    mi_cleared | shifted_bits
}
