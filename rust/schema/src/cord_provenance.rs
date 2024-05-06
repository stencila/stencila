//! Module for creating, modifying and displaying provenance
//! of a `CordRun` as a single byte
//!
//! The eight bits of the byte represent:
//!
//! 0:        0 = human written, 1 = machine written
//! 1:        1 = human edited
//! 2:        1 = machine edited
//! 3, 4, 5:  number of human verifiers, 0-7
//! 6, 7:     number of machine verifiers, 0-3

use crate::ProvenanceCategory;

/// Display the provenance byte as a string
pub fn display(prov: u8) -> String {
    let mut display = String::new();

    // Initial writer: human or machine
    if prov & 0b00000001 == 0 {
        display.push_str("Hw");
    } else {
        display.push_str("Mw");
    }

    // Last editor: human or machine
    if prov & 0b00000010 != 0 {
        display.push_str("He");
    } else if prov & 0b00000100 != 0 {
        display.push_str("Me");
    }

    // Human verifiers
    let human_verifiers = (prov & 0b00111000) >> 3;
    if human_verifiers > 0 {
        if human_verifiers > 1 {
            display.push_str(&human_verifiers.to_string());
        }
        display.push_str("Hv");
    }

    // Machine verifiers
    let machine_verifiers = (prov & 0b11000000) >> 6;
    if machine_verifiers > 0 {
        if machine_verifiers > 1 {
            display.push_str(&machine_verifiers.to_string());
        }
        display.push_str("Mv");
    }

    display
}

/// Get the category of Machine Influence
pub fn category(prov: u8) -> ProvenanceCategory {
    let hw = prov & 0b00000001 == 0;

    let he = prov & 0b00000010 != 0;
    let me = prov & 0b00000100 != 0;

    let hv = ((prov & 0b00111000) >> 3) > 0;
    let mv = ((prov & 0b11000000) >> 6) > 0;

    use ProvenanceCategory::*;
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
        } else if hv {
            HwHv
        } else if mv {
            HwMv
        } else {
            Hw
        }
    } else if he {
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
    } else if hv {
        MwHv
    } else if mv {
        MwMv
    } else {
        Mw
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
pub fn human_edited(prov: u8) -> u8 {
    (prov | 0b00000010) & 0b00000011
}

/// The run was edited by a machine
///
/// Set the "machine edited" bit and clear the "human edited" bit
/// and all verification bits.
pub fn machine_edited(prov: u8) -> u8 {
    (prov | 0b00000100) & 0b00000101
}

/// The run was verified by a human
///
/// Increment the three "human verifications" bits.
pub fn human_verified(prov: u8) -> u8 {
    let verifiers = (prov & 0b00111000) >> 3;
    if verifiers < 7 {
        (prov & 0b11000111) | ((verifiers + 1) << 3)
    } else {
        prov
    }
}

/// The run was verified by a machine
///
/// Increment the two "machine verification" bits.
pub fn machine_verified(prov: u8) -> u8 {
    let verifiers = (prov & 0b11000000) >> 6;
    if verifiers < 3 {
        (prov & 0b00111111) | ((verifiers + 1) << 6)
    } else {
        prov
    }
}
