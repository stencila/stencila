//! Sitemap generation for static sites
//!
//! Generates XML and text sitemap files from rendered site routes.

mod entry;
mod generate;

pub use entry::SitemapEntry;
pub use generate::{SitemapStats, generate_sitemaps};
