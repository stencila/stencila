use itertools::Itertools;
use similar::{Algorithm, DiffOp, capture_diff_slices};

/// Rebases edits from one document version to another, preserving user intent.
///
/// This function solves the "three-way merge" problem where:
///
/// - You have an original document (O)
/// - You create a lossy version (U) from O (e.g. converting formats)
/// - A user makes edits to U, producing E
/// - You want to apply those same logical edits back to O, producing O′
///
/// # Algorithm
///
/// 1. Computes the diff from U→E to identify what the user changed
/// 2. Maps those changes back to corresponding positions in O using the O→U diff  
/// 3. Applies the mapped changes to O, preserving both user edits and original content
///
/// # Parameters
///
/// - `original`: The source document (O) - contains full fidelity content
/// - `unedited`: The processed/lossy version (U) - derived from O, presented to user  
/// - `edited`: The user-modified version (E) - user's edits applied to U
///
/// # Returns
///
/// A new string (O′) that combines:
/// - The user's logical edits from the U→E transformation
/// - The original content from O that was lost in the U version
///
/// # Examples
///
/// ```rust
/// let original = "**bold** text here";     // O: Markdown with formatting  
/// let unedited = "bold text here";         // U: Plain text version without formatting
/// let edited = "BOLD text here";           // E: user changed "bold" to "BOLD"
/// let result = codec_utils::rebase_edits(original, unedited, edited);
/// assert_eq!(result, "**BOLD** text here"); // O′: edit applied, formatting preserved
/// ```
pub fn rebase_edits(original: &str, unedited: &str, edited: &str) -> String {
    // If there is no difference between unedited and edited then there are no edits
    // so just return the original
    if edited == unedited {
        return original.to_string();
    }

    // If there is no difference between original and unedited then there is no lifting
    // of edits to do, so just return edited
    if original == unedited {
        return edited.to_string();
    }

    let original = original.chars().collect_vec();
    let unedited = unedited.chars().collect_vec();
    let edited = edited.chars().collect_vec();

    // TODO: use objective, reproducible methods to evaluate suitability of alternative diff
    // algorithms. Currently using Myers for both
    // diffs as ad-hoc testing found it produced fewer conflicts that Myers+Patience. Also
    // Patience can take a very long time on large document.

    let o2u_ops = capture_diff_slices(Algorithm::Myers, &original, &unedited);

    let (.., u2o_prefixes, u2o_chars) = build_maps(&o2u_ops);

    let u2e_ops = capture_diff_slices(Algorithm::Myers, &unedited, &edited);

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
                let anchor = u2o_prefixes[old_index]; // position after the U-prefix
                if new_len > 0 {
                    let text = edited[new_index..new_index + new_len].iter().collect();
                    patch.push(PatchOp::Insert { pos: anchor, text });
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
                let text = edited[new_index..new_index + new_len].iter().collect();
                patch.push(PatchOp::Insert { pos: anchor, text });
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
                    let text: String = original[cur..pos].iter().collect();
                    applied.push_str(&text);
                }
                cur = pos + len;
            }
            PatchOp::Insert { pos, ref text } => {
                if cur < pos {
                    let text: String = original[cur..pos].iter().collect();
                    applied.push_str(&text);
                    cur = pos;
                }
                applied.push_str(text);
            }
        }
    }
    let text: String = original[cur..].iter().collect();
    applied.push_str(&text);

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

    pub use common_dev::proptest::prelude::*;

    /// Nothing changes anywhere.
    #[test]
    fn no_changes() {
        let o = "unchanged";
        let u = "unchanged";
        let e = "unchanged";
        assert_eq!(rebase_edits(o, u, e), "unchanged");
    }

    /// No edits.
    #[test]
    fn no_edits() {
        let o = "abcde";
        let u = "ade";
        let e = "ade";
        assert_eq!(rebase_edits(o, u, e), "abcde");
    }

    /// Pure insertion (O == U; user inserts into E.
    #[test]
    fn simple_insertion() {
        let o = "hello";
        let u = "hello";
        let e = "he--llo";
        assert_eq!(rebase_edits(o, u, e), "he--llo");
    }

    /// Pure deletion: O lost a char when forming U; the edit does nothing.
    /// The original char must survive the round-trip.
    #[test]
    fn deletion_only() {
        let o = "abcdef";
        let u = "abdef"; // lost the ‘c’
        let e = "abdef"; // user made no change around that spot
        assert_eq!(rebase_edits(o, u, e), "abcdef");
    }

    /// Replace a single character that happens to sit *after*
    /// a deletion in O→U.
    #[test]
    fn single_replacement() {
        let o = "abcdef";
        let u = "abdef"; // lost the ‘c’
        let e = "abDef"; // user upper-cases the ‘d’
        assert_eq!(rebase_edits(o, u, e), "abcDef");
    }

    /// Insert into a gap that was created by the lossy conversion.
    #[test]
    fn insert_into_deleted_gap() {
        let o = "abcdef";
        let u = "abdef"; // lost the ‘c’
        let e = "abXdef"; // user inserts ‘X’ where ‘c’ used to be
        assert_eq!(rebase_edits(o, u, e), "abXcdef");
    }

    #[test]
    fn replace_across_gap() {
        let o = "The quick brown fox";
        let u = "The uick brown fx"; // lost 'q' and 'o'
        let e = "The UICK brown fX"; // replaced next characters
        assert_eq!(rebase_edits(o, u, e), "The qUICK brown foX");
    }

    /// Mixed case
    #[test]
    fn round_trip_example() {
        let o = "abcDEFghi";
        let u = "abEFXh";
        let e = "ab--EFXH";
        assert_eq!(rebase_edits(o, u, e), "ab--cDEFgHi");
    }

    /// Unicode
    #[test]
    fn unicode() {
        let o = "a🌈cd😾f";
        let u = "abcdf";
        let e = "ab🍩def";
        assert_eq!(rebase_edits(o, u, e), "a🌈🍩de😾f");
    }

    proptest! {
        /// If there are no “edits” (unedited == edited),
        /// lift_edits must return the original.
        #[test]
        fn no_edit_yields_original(original in ".{0,50}", unedited in ".{0,50}") {
            let result = rebase_edits(&original, &unedited, &unedited);
            prop_assert_eq!(result, original);
        }

        /// If the “lossy” step did nothing (original == unedited),
        /// then lift_edits should just replay the edits on original directly,
        /// i.e. result == edited.
        #[test]
        fn no_loss_replays_edits(original in ".{0,50}", edited in ".{0,50}") {
            let result = rebase_edits(&original, &original, &edited);
            prop_assert_eq!(result, edited);
        }

        /// Fuzz-test the “full” path: when there are real differences
        /// in both the O→U (lossy) step and the U→E (edit) step.
        /// Only checks for panics (e.g. out of bounds errors)
        #[test]
        fn fuzz_full_diff(
            original in ".{0,50}",
            unedited in ".{0,50}",
            edited   in ".{0,50}",
        ) {
            // run the function (check it does not  panic)
            let _result = rebase_edits(&original, &unedited, &edited);

            // trivial assertion so proptest counts it as a test
            prop_assert!(true);
        }
    }
}
