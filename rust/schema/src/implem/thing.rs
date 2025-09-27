use crate::Thing;

impl Thing {
    /// Get the name of a [`Thing`]
    pub fn name(&self) -> String {
        self.options
            .name
            .clone()
            .unwrap_or_else(|| "Anonymous thing".into())
    }

    /// Get the short name of a [`Thing`]
    pub fn short_name(&self) -> String {
        self.options.name.clone().unwrap_or_else(|| "Anon".into())
    }
}
