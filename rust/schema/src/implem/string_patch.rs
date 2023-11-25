use crate::{StringOperation, StringPatch};

impl StringPatch {
    /// Apply the patch to a string and return a new, patched string
    pub fn apply(&self, string: &str) -> String {
        let mut string = String::from(string);
        self.apply_to(&mut string);
        string
    }

    /// Apply the patch to a mutable string in-place
    pub fn apply_to(&self, string: &mut String) {
        for op in &self.operations {
            match op {
                // Insert
                StringOperation {
                    start_position,
                    end_position: None,
                    value: Some(value),
                    ..
                } => string.insert_str(*start_position as usize, value),

                // Delete
                StringOperation {
                    start_position,
                    end_position: Some(end_position),
                    value: None,
                    ..
                } => string.replace_range((*start_position as usize)..(*end_position as usize), ""),

                // Replace
                StringOperation {
                    start_position,
                    end_position: Some(end_position),
                    value: Some(value),
                    ..
                } => string
                    .replace_range((*start_position as usize)..(*end_position as usize), value),

                // No op, ignore
                StringOperation {
                    end_position: None,
                    value: None,
                    ..
                } => {}
            }
        }
    }
}
