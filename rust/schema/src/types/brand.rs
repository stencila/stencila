//! Generated file, do not edit

use crate::prelude::*;

use super::block::Block;
use super::image_object_or_string::ImageObjectOrString;
use super::property_value_or_string::PropertyValueOrString;
use super::string::String;

/// A brand used by an organization or person for labeling a product, product group, or similar.
#[skip_serializing_none]
#[derive(Debug, Defaults, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", crate = "common::serde")]
pub struct Brand {
    /// The type of this item
    pub r#type: MustBe!("Brand"),

    /// The identifier for this item
    pub id: Option<String>,

    /// The name of the item.
    pub name: String,

    /// Non-core optional fields
    #[serde(flatten)]
    pub options: Box<BrandOptions>,
}

#[skip_serializing_none]
#[derive(Debug, Defaults, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", crate = "common::serde")]
pub struct BrandOptions {
    /// Alternate names (aliases) for the item.
    pub alternate_names: Option<Vec<String>>,

    /// A description of the item.
    pub description: Option<Vec<Block>>,

    /// Any kind of identifier for any kind of Thing.
    pub identifiers: Option<Vec<PropertyValueOrString>>,

    /// Images of the item.
    pub images: Option<Vec<ImageObjectOrString>>,

    /// The URL of the item.
    pub url: Option<String>,

    /// A logo associated with the brand.
    pub logo: Option<ImageObjectOrString>,

    /// Reviews of the brand.
    pub reviews: Option<Vec<String>>,
}

impl Brand {
    #[rustfmt::skip]
    pub fn new(name: String) -> Self {
        Self {
            name,
            ..Default::default()
        }
    }
}
