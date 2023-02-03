//! Generated file, do not edit

use crate::prelude::*;

use super::block::Block;
use super::brand::Brand;
use super::image_object_or_string::ImageObjectOrString;
use super::property_value_or_string::PropertyValueOrString;
use super::string::String;

/// Any offered product or service. For example, a pair of shoes; a haircut; or an episode of a TV show streamed online.
#[derive(Debug, Defaults, Clone, PartialEq, Serialize, Deserialize, Reconcile, Hydrate)]
#[serde(rename_all = "camelCase", crate = "common::serde")]
pub struct Product {
    /// The type of this item
    #[autosurgeon(with = "autosurgeon_must_be")]
    pub r#type: MustBe!("Product"),

    /// The identifier for this item
    #[key]
    pub id: Option<String>,

    /// Non-core optional fields
    #[serde(flatten)]
    pub options: Box<ProductOptions>,
}

#[derive(Debug, Defaults, Clone, PartialEq, Serialize, Deserialize, Reconcile, Hydrate)]
#[serde(rename_all = "camelCase", crate = "common::serde")]
pub struct ProductOptions {
    /// Alternate names (aliases) for the item.
    pub alternate_names: Option<Vec<String>>,

    /// A description of the item.
    pub description: Option<Vec<Block>>,

    /// Any kind of identifier for any kind of Thing.
    pub identifiers: Option<Vec<PropertyValueOrString>>,

    /// Images of the item.
    pub images: Option<Vec<ImageObjectOrString>>,

    /// The name of the item.
    pub name: Option<String>,

    /// The URL of the item.
    pub url: Option<String>,

    /// Brands that the product is labelled with.
    pub brands: Option<Vec<Brand>>,

    /// The logo of the product.
    pub logo: Option<ImageObjectOrString>,

    /// Product identification code.
    pub product_id: Option<String>,
}
