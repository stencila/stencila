//! Device presets for viewport configuration

use clap::ValueEnum;
use serde::{Deserialize, Serialize};

use crate::browser::ColorScheme;

/// Device presets with predefined viewport dimensions
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ValueEnum)]
#[serde(rename_all = "kebab-case")]
pub enum DevicePreset {
    /// Laptop (1440x900 @2x DPR)
    Laptop,
    /// Desktop (1920x1080 @1x DPR)
    Desktop,
    /// Mobile (390x844 @3x DPR)
    Mobile,
    /// Tablet (768x1024 @2x DPR)
    Tablet,
    /// Tablet Landscape (1024x768 @2x DPR)
    TabletLandscape,
}

impl DevicePreset {
    /// Get viewport configuration for this device
    pub fn viewport(self) -> ViewportConfig {
        match self {
            Self::Laptop => ViewportConfig {
                width: 1440,
                height: 900,
                dpr: 2.0,
                color_scheme: ColorScheme::default(),
            },
            Self::Desktop => ViewportConfig {
                width: 1920,
                height: 1080,
                dpr: 1.0,
                color_scheme: ColorScheme::default(),
            },
            Self::Mobile => ViewportConfig {
                width: 390,
                height: 844,
                dpr: 3.0,
                color_scheme: ColorScheme::default(),
            },
            Self::Tablet => ViewportConfig {
                width: 768,
                height: 1024,
                dpr: 2.0,
                color_scheme: ColorScheme::default(),
            },
            Self::TabletLandscape => ViewportConfig {
                width: 1024,
                height: 768,
                dpr: 2.0,
                color_scheme: ColorScheme::default(),
            },
        }
    }
}

/// Viewport configuration (width, height, device pixel ratio, color scheme)
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct ViewportConfig {
    /// Viewport width in pixels
    pub width: u32,

    /// Viewport height in pixels
    pub height: u32,

    /// Device pixel ratio
    pub dpr: f32,

    /// Color scheme preference
    pub color_scheme: ColorScheme,
}

impl Default for ViewportConfig {
    fn default() -> Self {
        // Default to desktop viewport
        Self {
            width: 1920,
            height: 1080,
            dpr: 1.0,
            color_scheme: ColorScheme::default(),
        }
    }
}
