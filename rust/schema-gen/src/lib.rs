pub mod schema;
pub mod schemas;

#[cfg(feature = "docs")]
mod docs_codecs;

#[cfg(feature = "docs")]
mod docs_types;

mod json_ld;
mod json_schema;
mod python;
mod rust;
mod typescript;
