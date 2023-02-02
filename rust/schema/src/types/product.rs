//! Generated file, do not edit

use crate::prelude::*;

use super::block::Block;
use super::brand::Brand;
use super::image_object_or_string::ImageObjectOrString;
use super::property_value_or_string::PropertyValueOrString;
use super::string::String;

/// Any offered product or service. For example, a pair of shoes; a haircut; or an episode of a TV show streamed online.
#[derive(Debug, Defaults, Clone, PartialEq, Serialize, Deserialize)]
#[serde(crate = "common::serde")]
pub struct Product {
    /// The type of this item
    r#type: MustBe!("Product"),

    /// The identifier for this item
    id: String,

    /// Non-core optional fields
    #[serde(flatten)]
    options: Box<ProductOptions>,
}

#[derive(Debug, Defaults, Clone, PartialEq, Serialize, Deserialize)]
#[serde(crate = "common::serde")]
pub struct ProductOptions {
    /// Alternate names (aliases) for the item.
    alternate_names: Option<Vec<String>>,

    /// A description of the item.
    description: Option<Vec<Block>>,

    /// Any kind of identifier for any kind of Thing.
    identifiers: Option<Vec<PropertyValueOrString>>,

    /// Images of the item.
    images: Option<Vec<ImageObjectOrString>>,

    /// The name of the item.
    name: Option<String>,

    /// The URL of the item.
    url: Option<String>,

    /// Brands that the product is labelled with.
    brands: Option<Vec<Brand>>,

    /// The logo of the product.
    logo: Option<ImageObjectOrString>,

    /// Product identification code.
    product_id: Option<String>,
}
