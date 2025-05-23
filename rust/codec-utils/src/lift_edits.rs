use common::similar::{Algorithm, DiffOp, capture_diff_slices};

/// Given three documents,
///
/// - Original (O)
/// - Unedited (U, lossy version of O)
/// - Edited (E, edited version of U)
///
/// Calculate the edits from Unedited to Edited, "lift" them to Original-positions, and
/// apply them to Original to return Original′.
pub fn lift_edits(original: &str, unedited: &str, edited: &str) -> String {
    let o2u_ops = capture_diff_slices(Algorithm::Myers, original.as_bytes(), unedited.as_bytes());

    let (.., u2o_prefixes, u2o_chars) = build_maps(&o2u_ops);

    let u2e_ops = capture_diff_slices(Algorithm::Patience, unedited.as_bytes(), edited.as_bytes());

    // Calculate patch
    let mut patch: Vec<PatchOp> = Vec::new();
    for op in &u2e_ops {
        match *op {
            DiffOp::Equal { .. } => {}

            DiffOp::Delete {
                old_index, old_len, ..
            } => {
                push_deletions(&mut patch, old_index, old_len, &u2o_chars);
            }

            DiffOp::Insert {
                old_index,
                new_len,
                new_index,
                ..
            } => {
                let anchor = u2o_prefixes[old_index]; // position after the B-prefix
                if new_len > 0 {
                    let text = &edited.as_bytes()[new_index..new_index + new_len];
                    patch.push(PatchOp::Insert {
                        pos: anchor,
                        text: String::from_utf8_lossy(text).into_owned(),
                    });
                }
            }

            DiffOp::Replace {
                old_index,
                old_len,
                new_index,
                new_len,
                ..
            } => {
                // old part → deletions in O
                push_deletions(&mut patch, old_index, old_len, &u2o_chars);

                // new part → insertion in O just after the deleted span
                let anchor = u2o_prefixes[old_index + old_len];
                let text = &edited.as_bytes()[new_index..new_index + new_len];
                patch.push(PatchOp::Insert {
                    pos: anchor,
                    text: String::from_utf8_lossy(text).into_owned(),
                });
            }
        }
    }

    // Apply patch
    let mut applied = String::with_capacity(original.len());
    let mut cur = 0usize;
    for op in patch {
        match op {
            PatchOp::Delete { pos, len } => {
                if cur < pos {
                    applied.push_str(&original[cur..pos]);
                }
                cur = pos + len;
            }
            PatchOp::Insert { pos, ref text } => {
                if cur < pos {
                    applied.push_str(&original[cur..pos]);
                    cur = pos;
                }
                applied.push_str(text);
            }
        }
    }
    applied.push_str(&original[cur..]);

    applied
}

/// A patch operation expressed in O-position.
#[derive(Debug)]
enum PatchOp {
    Delete { pos: usize, len: usize },
    Insert { pos: usize, text: String },
}

/// Build maps
///
///   * `o2b_pref`  – prefix map O→U
///   * `b2o_pref`  – prefix map U→O
///   * `b2o_char`  – per-char mapping: for each byte in U, which byte in O?
fn build_maps(dob: &[DiffOp]) -> (Vec<usize>, Vec<usize>, Vec<Option<usize>>) {
    // infer final lengths
    let (mut len_o, mut len_u) = (0usize, 0usize);
    for op in dob {
        match *op {
            DiffOp::Equal { len, .. } => {
                len_o += len;
                len_u += len;
            }
            DiffOp::Delete { old_len, .. } => len_o += old_len,
            DiffOp::Insert { new_len, .. } => len_u += new_len,
            DiffOp::Replace {
                old_len, new_len, ..
            } => {
                len_o += old_len;
                len_u += new_len;
            }
        }
    }

    let mut o2u = vec![0usize; len_o + 1];
    let mut u2o = vec![0usize; len_u + 1];
    let mut char_map = vec![None; len_u]; // length == |B|

    let (mut i, mut j) = (0usize, 0usize);

    for op in dob {
        match *op {
            DiffOp::Equal { len, .. } => {
                for _ in 0..len {
                    i += 1;
                    j += 1;
                    o2u[i] = j;
                    u2o[j] = i;
                    char_map[j - 1] = Some(i - 1);
                }
            }
            DiffOp::Delete { old_len, .. } => {
                for _ in 0..old_len {
                    i += 1;
                    o2u[i] = j;
                }
            }
            DiffOp::Insert { new_len, .. } => {
                for _ in 0..new_len {
                    j += 1;
                    u2o[j] = i;
                    // inserted char: None in char_map
                }
            }
            DiffOp::Replace {
                old_len, new_len, ..
            } => {
                // deletions
                for _ in 0..old_len {
                    i += 1;
                    o2u[i] = j;
                }
                // insertions
                for _ in 0..new_len {
                    j += 1;
                    u2o[j] = i;
                    // replaced chars are *new* wrt O → None in char_map
                }
            }
        }
    }

    debug_assert_eq!(i, len_o);
    debug_assert_eq!(j, len_u);

    (o2u, u2o, char_map)
}

/// Convert a U-segment (old_index, old_len) into one or more contiguous
/// `Delete` ops in O using the fine-grained char map.
fn push_deletions(
    patch: &mut Vec<PatchOp>,
    old_index: usize,
    old_len: usize,
    char_map: &[Option<usize>],
) {
    let mut run_start: Option<usize> = None;
    let mut last_o: usize = 0;

    for pos_o in char_map.iter().skip(old_index).take(old_len).flatten() {
        let pos_o = *pos_o;
        match run_start {
            None => {
                run_start = Some(pos_o);
                last_o = pos_o;
            }
            Some(..) if pos_o == last_o + 1 => {
                // still contiguous
                last_o = pos_o;
            }
            Some(start) => {
                patch.push(PatchOp::Delete {
                    pos: start,
                    len: last_o - start + 1,
                });
                run_start = Some(pos_o);
                last_o = pos_o;
            }
        }
    }
    if let Some(start) = run_start {
        patch.push(PatchOp::Delete {
            pos: start,
            len: last_o - start + 1,
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Identity: nothing changes anywhere.
    #[test]
    fn no_op() {
        let o = "unchanged";
        let u = "unchanged";
        let e = "unchanged";
        assert_eq!(lift_edits(o, u, e), "unchanged");
    }

    /// Pure insertion (O == U; user inserts into E.
    #[test]
    fn simple_insertion() {
        let o = "hello";
        let u = "hello";
        let e = "he--llo";
        assert_eq!(lift_edits(o, u, e), "he--llo");
    }

    /// Pure deletion: O lost a char when forming U; the edit does nothing.
    /// The original char must survive the round-trip.
    #[test]
    fn deletion_only() {
        let o = "abcdef";
        let u = "abdef"; // lost the ‘c’
        let e = "abdef"; // user made no change around that spot
        assert_eq!(lift_edits(o, u, e), "abcdef");
    }

    /// Replace a single character that happens to sit *after*
    /// a deletion in O→U.
    #[test]
    fn single_replacement() {
        let o = "abcdef";
        let u = "abdef"; // lost the ‘c’
        let e = "abDef"; // user upper-cases the ‘d’
        assert_eq!(lift_edits(o, u, e), "abcDef");
    }

    /// Insert into a gap that was created by the lossy conversion.
    #[test]
    fn insert_into_deleted_gap() {
        let o = "abcdef";
        let u = "abdef"; // lost the ‘c’
        let e = "abXdef"; // user inserts ‘X’ where ‘c’ used to be
        assert_eq!(lift_edits(o, u, e), "abXcdef");
    }

    /// Mixed case
    #[test]
    fn round_trip_example() {
        let o = "abcDEFghi";
        let u = "abEFXh";
        let e = "ab--EFXH";
        assert_eq!(lift_edits(o, u, e), "ab--cDEFgHi");
    }

    /// Unicode
    #[test]
    fn unicode() {
        let o = "a🌈cd😾f";
        let u = "abcdf";
        let e = "ab🍩def";
        assert_eq!(lift_edits(o, u, e), "a🌈🍩de😾f");
    }
}
