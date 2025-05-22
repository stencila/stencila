//! Given three documents Original, Base (imperfect conversion of original),
//! and Edited (edited version of base) this lift the edits that turned B → E back onto O.

use common::similar::{Algorithm, DiffOp, capture_diff_slices};

/// An patch operation expressed in `O`-coordinates.
#[derive(Debug)]
enum PatchOp {
    Delete { pos: usize, len: usize },
    Insert { pos: usize, text: String },
}

/// Top-level helper: apply the B → E edits to `O` and return O′.
pub fn lift_edits(o: &str, b: &str, e: &str) -> String {
    let d_ob = capture_diff_slices(Algorithm::Myers, o.as_bytes(), b.as_bytes());

    let (_o2b_pref, b2o_pref, b2o_char) = build_maps(&d_ob);

    let d_be = capture_diff_slices(Algorithm::Patience, b.as_bytes(), e.as_bytes());

    let mut patch: Vec<PatchOp> = Vec::new();

    for op in &d_be {
        match *op {
            DiffOp::Equal { .. } => {}

            DiffOp::Delete {
                old_index, old_len, ..
            } => {
                push_deletions(&mut patch, old_index, old_len, &b2o_char);
            }

            DiffOp::Insert {
                old_index,
                new_len,
                new_index,
                ..
            } => {
                let anchor = b2o_pref[old_index]; // position after the B-prefix
                if new_len > 0 {
                    let text = &e.as_bytes()[new_index..new_index + new_len];
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
                push_deletions(&mut patch, old_index, old_len, &b2o_char);

                // new part → insertion in O just after the deleted span
                let anchor = b2o_pref[old_index + old_len];
                let text = &e.as_bytes()[new_index..new_index + new_len];
                patch.push(PatchOp::Insert {
                    pos: anchor,
                    text: String::from_utf8_lossy(text).into_owned(),
                });
            }
        }
    }

    apply_patch(o, &patch)
}

/// Build maps
///
///   * `o2b_pref`  – prefix map O→B
///   * `b2o_pref`  – prefix map B→O
///   * `b2o_char`  – per-char mapping: for each byte in B, which byte in O?
fn build_maps(dob: &[DiffOp]) -> (Vec<usize>, Vec<usize>, Vec<Option<usize>>) {
    // infer final lengths
    let (mut len_o, mut len_b) = (0usize, 0usize);
    for op in dob {
        match *op {
            DiffOp::Equal { len, .. } => {
                len_o += len;
                len_b += len;
            }
            DiffOp::Delete { old_len, .. } => len_o += old_len,
            DiffOp::Insert { new_len, .. } => len_b += new_len,
            DiffOp::Replace {
                old_len, new_len, ..
            } => {
                len_o += old_len;
                len_b += new_len;
            }
        }
    }

    let mut o2b = vec![0usize; len_o + 1];
    let mut b2o = vec![0usize; len_b + 1];
    let mut char_map = vec![None; len_b]; // length == |B|

    let (mut i, mut j) = (0usize, 0usize);

    for op in dob {
        match *op {
            DiffOp::Equal { len, .. } => {
                for _ in 0..len {
                    i += 1;
                    j += 1;
                    o2b[i] = j;
                    b2o[j] = i;
                    char_map[j - 1] = Some(i - 1);
                }
            }
            DiffOp::Delete { old_len, .. } => {
                for _ in 0..old_len {
                    i += 1;
                    o2b[i] = j;
                }
            }
            DiffOp::Insert { new_len, .. } => {
                for _ in 0..new_len {
                    j += 1;
                    b2o[j] = i;
                    // inserted char: None in char_map
                }
            }
            DiffOp::Replace {
                old_len, new_len, ..
            } => {
                // deletions
                for _ in 0..old_len {
                    i += 1;
                    o2b[i] = j;
                }
                // insertions
                for _ in 0..new_len {
                    j += 1;
                    b2o[j] = i;
                    // replaced chars are *new* wrt O → None in char_map
                }
            }
        }
    }

    debug_assert_eq!(i, len_o);
    debug_assert_eq!(j, len_b);
    (o2b, b2o, char_map)
}

/// Convert a B-segment (old_index, old_len) into one or more contiguous
/// `Delete` ops in O using the fine-grained char map.
fn push_deletions(
    patch: &mut Vec<PatchOp>,
    old_index: usize,
    old_len: usize,
    char_map: &[Option<usize>],
) {
    let mut run_start: Option<usize> = None;
    let mut last_o: usize = 0;

    for j in old_index..old_index + old_len {
        if let Some(pos_o) = char_map[j] {
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
    }
    if let Some(start) = run_start {
        patch.push(PatchOp::Delete {
            pos: start,
            len: last_o - start + 1,
        });
    }
}

/// Apply the ordered operations to original
fn apply_patch(original: &str, ops: &[PatchOp]) -> String {
    let mut out = String::with_capacity(original.len());
    let mut cur = 0usize;
    for op in ops {
        match *op {
            PatchOp::Delete { pos, len } => {
                if cur < pos {
                    out.push_str(&original[cur..pos]);
                }
                cur = pos + len;
            }
            PatchOp::Insert { pos, ref text } => {
                if cur < pos {
                    out.push_str(&original[cur..pos]);
                    cur = pos;
                }
                out.push_str(text);
            }
        }
    }
    out.push_str(&original[cur..]);
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Identity: nothing changes anywhere.
    #[test]
    fn no_op() {
        let o = "unchanged";
        let b = "unchanged";
        let e = "unchanged";
        assert_eq!(lift_edits(o, b, e), "unchanged");
    }

    /// Pure insertion (O == B; user inserts into E.
    #[test]
    fn simple_insertion() {
        let o = "hello";
        let b = "hello";
        let e = "he--llo";
        assert_eq!(lift_edits(o, b, e), "he--llo");
    }

    /// Pure deletion: O lost a char when forming B; the edit does nothing.
    /// The original char must survive the round-trip.
    #[test]
    fn deletion_only() {
        let o = "abcdef";
        let b = "abdef"; // lost the ‘c’
        let e = "abdef"; // user made no change around that spot
        assert_eq!(lift_edits(o, b, e), "abcdef");
    }

    /// Replace a single character that happens to sit *after*
    /// a deletion in O→B.
    #[test]
    fn single_replacement() {
        let o = "abcdef";
        let b = "abdef"; // lost the ‘c’
        let e = "abDef"; // user upper-cases the ‘d’
        assert_eq!(lift_edits(o, b, e), "abcDef");
    }

    /// Insert into a gap that was created by the lossy conversion.
    #[test]
    fn insert_into_deleted_gap() {
        let o = "abcdef";
        let b = "abdef"; // lost the ‘c’
        let e = "abXdef"; // user inserts ‘X’ where ‘c’ used to be
        assert_eq!(lift_edits(o, b, e), "abXcdef");
    }

    /// Mixed case
    #[test]
    fn round_trip_example() {
        let o = "abcDEFghi";
        let b = "abEFXh";
        let e = "ab--EFXH";
        assert_eq!(lift_edits(o, b, e), "ab--cDEFgHi");
    }

    /// Unicode
    #[test]
    fn unicode() {
        let o = "a🌈cd😾f";
        let b = "abcdf";
        let e = "ab🍩def";
        assert_eq!(lift_edits(o, b, e), "a🌈🍩de😾f");
    }
}
