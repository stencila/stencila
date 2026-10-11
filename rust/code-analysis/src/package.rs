/// A package imported by source code.
#[derive(serde::Serialize, serde::Deserialize, Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "camelCase")]
pub struct PackageFact {
    /// Package ecosystem or registry namespace, such as `pypi`, `cran`, or `npm`.
    pub ecosystem: String,

    /// Package name as detected from source.
    pub name: String,
}

impl PackageFact {
    /// Create a source-code package fact.
    pub fn new(ecosystem: impl Into<String>, name: impl Into<String>) -> Self {
        Self {
            ecosystem: ecosystem.into(),
            name: name.into(),
        }
    }
}
