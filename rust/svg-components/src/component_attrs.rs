/// Per-component attribute specification for validation.
pub struct ComponentAttrSpec {
    /// All valid attributes for this component (excluding universal pass-through attrs).
    pub valid_attrs: &'static [&'static str],
    /// Attributes with enumerated valid values.
    pub enum_attrs: &'static [(&'static str, &'static [&'static str])],
}

/// SVG presentation attributes that are always valid on any component
/// (these are passed through to the output SVG).
const SVG_PASS_THROUGH_ATTRS: &[&str] = &[
    "class",
    "style",
    "font-size",
    "font-family",
    "font-weight",
    "font-style",
    "stroke-width",
    "stroke-dasharray",
    "stroke-linecap",
    "stroke-linejoin",
    "stroke-opacity",
    "fill-opacity",
    "transform",
    "filter",
    "clip-path",
    "pointer-events",
    "visibility",
    "display",
    "cursor",
    "data-*",
];

/// Universal component attributes recognized by the framework
/// (used for positioning, color, and component identity).
const UNIVERSAL_ATTRS: &[&str] = &[
    "dx",
    "dy",
    "fill",
    "stroke",
    "color",
    "text",
    "background",
    "opacity",
];

/// Look up the attribute specification for a component by name.
pub fn component_attr_spec(name: &str) -> Option<&'static ComponentAttrSpec> {
    match name {
        "angle" => Some(&ANGLE_SPEC),
        "arrow" => Some(&ARROW_SPEC),
        "badge" => Some(&BADGE_SPEC),
        "brace" => Some(&BRACE_SPEC),
        "bracket" => Some(&BRACKET_SPEC),
        "callout" => Some(&CALLOUT_SPEC),
        "compass" => Some(&COMPASS_SPEC),
        "crosshair" => Some(&CROSSHAIR_SPEC),
        "dimension" => Some(&DIMENSION_SPEC),
        "halo" => Some(&HALO_SPEC),
        "marker" => Some(&MARKER_SPEC),
        "roi-ellipse" => Some(&ROI_ELLIPSE_SPEC),
        "roi-polygon" => Some(&ROI_POLYGON_SPEC),
        "roi-rect" => Some(&ROI_RECT_SPEC),
        "scale-bar" => Some(&SCALE_BAR_SPEC),
        "spotlight" => Some(&SPOTLIGHT_SPEC),
        "anchor" => Some(&ANCHOR_SPEC),
        _ => None,
    }
}

/// Check if an attribute name is valid for a given component.
pub fn is_valid_attr(component_name: &str, attr_name: &str) -> bool {
    // Universal attrs are always valid
    if UNIVERSAL_ATTRS.contains(&attr_name) {
        return true;
    }

    // SVG presentation pass-through attrs
    if SVG_PASS_THROUGH_ATTRS.contains(&attr_name) || attr_name.starts_with("data-") {
        return true;
    }

    // Check component-specific attrs
    if let Some(spec) = component_attr_spec(component_name) {
        return spec.valid_attrs.contains(&attr_name);
    }

    // Unknown component — don't flag attrs
    true
}

/// Validate an attribute value against its enum constraint.
/// Returns `Some(valid_values)` if the value is invalid, `None` if valid or unconstrained.
pub fn validate_enum_attr<'a>(
    component_name: &str,
    attr_name: &str,
    attr_value: &str,
) -> Option<&'a [&'a str]> {
    let spec = component_attr_spec(component_name)?;
    for &(name, values) in spec.enum_attrs {
        if name == attr_name && !values.contains(&attr_value) {
            return Some(values);
        }
    }
    None
}

// --- Component attribute specifications ---

static ANGLE_SPEC: ComponentAttrSpec = ComponentAttrSpec {
    valid_attrs: &[
        "x", "y", "at", "from", "from-x", "from-y", "to", "to-x", "to-y", "r", "label",
    ],
    enum_attrs: &[],
};

static ARROW_SPEC: ComponentAttrSpec = ComponentAttrSpec {
    valid_attrs: &[
        "x",
        "y",
        "from",
        "to",
        "to-x",
        "to-y",
        "curve",
        "corner",
        "tip",
        "tip-style",
        "label",
        "label-position",
        "label-angle",
    ],
    enum_attrs: &[
        ("curve", &["straight", "elbow", "quad", "cubic"]),
        ("tip", &["end", "start", "both", "none"]),
        ("corner", &["horizontal-first", "vertical-first"]),
        ("label-position", &["above", "below"]),
        ("label-angle", &["along", "horizontal", "vertical"]),
    ],
};

static BADGE_SPEC: ComponentAttrSpec = ComponentAttrSpec {
    valid_attrs: &["x", "y", "from", "label"],
    enum_attrs: &[],
};

static BRACE_SPEC: ComponentAttrSpec = ComponentAttrSpec {
    valid_attrs: &[
        "x", "y", "from", "to", "to-x", "to-y", "side", "bulge", "label",
    ],
    enum_attrs: &[("side", &["above", "below", "left", "right"])],
};

static BRACKET_SPEC: ComponentAttrSpec = ComponentAttrSpec {
    valid_attrs: &[
        "x", "y", "from", "to", "to-x", "to-y", "side", "depth", "variant", "label",
    ],
    enum_attrs: &[
        ("side", &["above", "below", "left", "right"]),
        ("variant", &["square", "round"]),
    ],
};

static CALLOUT_SPEC: ComponentAttrSpec = ComponentAttrSpec {
    valid_attrs: &[
        "x",
        "y",
        "from",
        "to",
        "to-x",
        "to-y",
        "label",
        "label-position",
        "shape",
        "curve",
        "tip-style",
    ],
    enum_attrs: &[
        (
            "label-position",
            &["above", "below", "left", "right", "auto"],
        ),
        ("shape", &["none", "rect", "pill", "circle"]),
        ("curve", &["straight", "elbow", "quad", "cubic"]),
    ],
};

static COMPASS_SPEC: ComponentAttrSpec = ComponentAttrSpec {
    valid_attrs: &["x", "y", "at", "size", "variant", "axes"],
    enum_attrs: &[("variant", &["arrow", "full"])],
};

static CROSSHAIR_SPEC: ComponentAttrSpec = ComponentAttrSpec {
    valid_attrs: &[
        "cx",
        "cy",
        "at",
        "size",
        "gap",
        "ring",
        "label",
        "label-position",
    ],
    enum_attrs: &[
        ("ring", &["true", "false"]),
        ("label-position", &["above", "below", "left", "right"]),
    ],
};

static DIMENSION_SPEC: ComponentAttrSpec = ComponentAttrSpec {
    valid_attrs: &[
        "x",
        "y",
        "from",
        "to",
        "to-x",
        "to-y",
        "label",
        "label-position",
        "label-angle",
        "side",
    ],
    enum_attrs: &[
        ("label-position", &["above", "below"]),
        ("label-angle", &["along", "horizontal", "vertical"]),
        ("side", &["above", "below", "left", "right"]),
    ],
};

static HALO_SPEC: ComponentAttrSpec = ComponentAttrSpec {
    valid_attrs: &["cx", "cy", "at", "r", "width"],
    enum_attrs: &[],
};

static MARKER_SPEC: ComponentAttrSpec = ComponentAttrSpec {
    valid_attrs: &["x", "y", "at", "symbol", "size", "label", "label-position"],
    enum_attrs: &[
        (
            "symbol",
            &[
                "circle",
                "square",
                "pin",
                "diamond",
                "triangle",
                "triangle-down",
                "cross",
                "plus",
                "star",
            ],
        ),
        ("label-position", &["right", "above", "below", "left"]),
    ],
};

static ROI_ELLIPSE_SPEC: ComponentAttrSpec = ComponentAttrSpec {
    valid_attrs: &[
        "cx",
        "cy",
        "at",
        "rx",
        "ry",
        "label",
        "label-position",
        "stroke-style",
    ],
    enum_attrs: &[
        (
            "label-position",
            &["above", "below", "center", "left", "right"],
        ),
        ("stroke-style", &["solid", "dashed", "dotted"]),
    ],
};

static ROI_POLYGON_SPEC: ComponentAttrSpec = ComponentAttrSpec {
    valid_attrs: &["points", "label", "label-position", "stroke-style"],
    enum_attrs: &[
        (
            "label-position",
            &["above", "below", "center", "left", "right"],
        ),
        ("stroke-style", &["solid", "dashed", "dotted"]),
    ],
};

static ROI_RECT_SPEC: ComponentAttrSpec = ComponentAttrSpec {
    valid_attrs: &[
        "x",
        "y",
        "from",
        "to",
        "width",
        "height",
        "label",
        "label-position",
        "stroke-style",
    ],
    enum_attrs: &[
        (
            "label-position",
            &["above", "below", "center", "left", "right"],
        ),
        ("stroke-style", &["solid", "dashed", "dotted"]),
    ],
};

static SCALE_BAR_SPEC: ComponentAttrSpec = ComponentAttrSpec {
    valid_attrs: &["x", "y", "at", "length", "label", "label-position", "side"],
    enum_attrs: &[
        ("label-position", &["below", "above"]),
        ("side", &["bottom", "top", "left", "right"]),
    ],
};

static SPOTLIGHT_SPEC: ComponentAttrSpec = ComponentAttrSpec {
    valid_attrs: &[
        "cx", "cy", "at", "r", "rx", "ry", "shape", "width", "height",
    ],
    enum_attrs: &[("shape", &["circle", "rect"])],
};

static ANCHOR_SPEC: ComponentAttrSpec = ComponentAttrSpec {
    valid_attrs: &["id", "x", "y"],
    enum_attrs: &[],
};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn known_component_attrs() {
        assert!(is_valid_attr("badge", "x"));
        assert!(is_valid_attr("badge", "label"));
        assert!(is_valid_attr("badge", "from"));
        // Universal attrs
        assert!(is_valid_attr("badge", "fill"));
        assert!(is_valid_attr("badge", "stroke"));
        assert!(is_valid_attr("badge", "dx"));
    }

    #[test]
    fn unknown_attr_on_known_component() {
        assert!(!is_valid_attr("badge", "bogus"));
        assert!(!is_valid_attr("arrow", "ring"));
    }

    #[test]
    fn pass_through_svg_attrs() {
        assert!(is_valid_attr("badge", "class"));
        assert!(is_valid_attr("arrow", "stroke-width"));
        assert!(is_valid_attr("callout", "data-id"));
    }

    #[test]
    fn enum_validation_valid() {
        assert!(validate_enum_attr("arrow", "curve", "elbow").is_none());
        assert!(validate_enum_attr("arrow", "tip", "both").is_none());
    }

    #[test]
    fn enum_validation_invalid() {
        let values = validate_enum_attr("arrow", "curve", "wobbly");
        assert!(values.is_some());
        assert!(values.expect("should be Some").contains(&"straight"));
    }

    #[test]
    fn enum_validation_unconstrained_attr() {
        assert!(validate_enum_attr("arrow", "label", "anything").is_none());
    }

    #[test]
    fn unknown_component_passes_all() {
        assert!(is_valid_attr("unknown-thing", "whatever"));
    }
}
