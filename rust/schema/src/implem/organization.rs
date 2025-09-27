use crate::Organization;

impl Organization {
    /// Get the name of a [`Organization`]
    pub fn name(&self) -> String {
        self.name
            .clone()
            .unwrap_or_else(|| "Anonymous organization".into())
    }

    /// Get the short name of a [`Organization`]
    pub fn short_name(&self) -> String {
        self.name.clone().unwrap_or_else(|| "Anon".into())
    }
}
