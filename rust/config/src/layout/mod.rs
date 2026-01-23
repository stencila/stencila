//! Site layout configuration types
//!
//! This module contains types for configuring the layout of site pages using
//! a region-based system where components can be placed in any region's sub-regions.
//!
//! ## Architecture
//!
//! The layout consists of regions (header, left-sidebar, top, content, bottom,
//! right-sidebar, footer), each with sub-regions (start, middle, end). Components
//! can be placed in any sub-region.
//!
//! ## Example
//!
//! ```toml
//! [site.layout.header]
//! start = "logo"
//! middle = { type = "nav-links", links = [...] }
//! end = ["icon-links", "color-mode"]
//!
//! [site.layout.left-sidebar]
//! middle = { type = "nav-tree", collapsible = true }
//! ```

mod components;
mod config;
mod overrides;
mod presets;
mod regions;

pub use components::{
    ColorModeStyle, ComponentConfig, ComponentSpec, CopyMarkdownStyle, CustomSocialLink,
    EditOnService, EditSourceStyle, NavGroupsIcons, NavMenuDropdownStyle, NavMenuGroups,
    NavMenuIcons, NavMenuTrigger, NavTreeIcons, PrevNextStyle, SocialLinkPlatform,
    SocialLinksStyle,
};
pub use config::LayoutConfig;
pub use overrides::LayoutOverride;
pub use presets::LayoutPreset;
pub use regions::{RegionConfig, RegionSpec, RowConfig};
