use std::collections::HashMap;

use crate::anchors::Anchor;
use crate::bbox::BBox;
use crate::components::{Attrs, attr_f64, attr_f64_or, attr_str, resolve_position, resolve_target};

/// The estimated geometry of a component for collision detection.
#[derive(Debug, Clone)]
pub struct ComponentGeometry {
    /// Bounding box of the label/text area (if the component has a label).
    pub label_bbox: Option<BBox>,
    /// Line segments produced by the component: (x1, y1, x2, y2).
    pub line_segments: Vec<(f64, f64, f64, f64)>,
}

/// Estimate the geometry of a component for lint collision detection.
///
/// Returns `None` if the component can't be resolved (missing position, etc.).
pub fn estimate_geometry(
    name: &str,
    attrs: &Attrs,
    anchors: &HashMap<String, Anchor>,
) -> Option<ComponentGeometry> {
    match name {
        "badge" => estimate_badge(attrs, anchors),
        "callout" => estimate_callout(attrs, anchors),
        "arrow" => estimate_arrow(attrs, anchors),
        "dimension" => estimate_dimension(attrs, anchors),
        "scale-bar" => estimate_scale_bar(attrs, anchors),
        "brace" => estimate_brace(attrs, anchors),
        "bracket" => estimate_bracket(attrs, anchors),
        "marker" => estimate_marker(attrs, anchors),
        "compass" => estimate_compass(attrs, anchors),
        "crosshair" => estimate_crosshair(attrs, anchors),
        "angle" => estimate_angle(attrs, anchors),
        "roi-rect" => estimate_roi_rect(attrs, anchors),
        "roi-ellipse" => estimate_roi_ellipse(attrs, anchors),
        "roi-polygon" => estimate_roi_polygon(attrs),
        // spotlight and halo are excluded from collision checks;
        // unknown components also return None
        _ => None,
    }
}

/// Estimate text bounding box centered at (cx, cy) with given char-width multiplier and padding.
fn label_bbox_centered(
    cx: f64,
    cy: f64,
    label: &str,
    char_width: f64,
    padding: f64,
    height: f64,
) -> Option<BBox> {
    if label.is_empty() {
        return None;
    }
    let w = label.chars().count() as f64 * char_width + padding;
    Some(BBox::new(cx - w / 2.0, cy - height / 2.0, w, height))
}

/// Estimate text bounding box from a text anchor point. Font size 12, 0.6 multiplier.
fn label_bbox_at(x: f64, y: f64, label: &str) -> Option<BBox> {
    label_bbox_centered(x, y, label, 7.2, 0.0, 14.0)
}

fn estimate_badge(attrs: &Attrs, anchors: &HashMap<String, Anchor>) -> Option<ComponentGeometry> {
    let (x, y) = resolve_position(attrs, "x", "y", Some("from"), "dx", "dy", anchors)?;
    let label = attr_str(attrs, "label", "");
    Some(ComponentGeometry {
        label_bbox: label_bbox_centered(x, y, label, 6.5, 12.0, 18.0),
        line_segments: vec![],
    })
}

fn estimate_callout(attrs: &Attrs, anchors: &HashMap<String, Anchor>) -> Option<ComponentGeometry> {
    let (lx, ly) = resolve_position(attrs, "x", "y", Some("from"), "dx", "dy", anchors)?;
    let label = attr_str(attrs, "label", "");
    let target = resolve_target(attrs, anchors);

    let mut line_segments = Vec::new();
    if let Some((tx, ty)) = target {
        line_segments.push((lx, ly, tx, ty));
    }

    // Callout label position depends on leader line direction
    let shape_height = 22.0;
    let label_position = attr_str(attrs, "label-position", "auto");
    let (text_x, text_y) =
        callout_label_offset(lx, ly, target.as_ref(), label_position, shape_height);

    Some(ComponentGeometry {
        label_bbox: label_bbox_centered(text_x, text_y, label, 7.5, 16.0, shape_height),
        line_segments,
    })
}

fn callout_label_offset(
    lx: f64,
    ly: f64,
    target: Option<&(f64, f64)>,
    position: &str,
    shape_height: f64,
) -> (f64, f64) {
    let gap = shape_height * 0.75;
    match position {
        "above" => (lx, ly - gap),
        "below" => (lx, ly + gap),
        "left" => (lx - gap, ly),
        "right" => (lx + gap, ly),
        _ => {
            let Some(&(tx, ty)) = target else {
                return (lx, ly);
            };
            let dx = tx - lx;
            let dy = ty - ly;
            if dy.abs() >= dx.abs() {
                if dy >= 0.0 {
                    (lx, ly - gap)
                } else {
                    (lx, ly + gap)
                }
            } else {
                (lx, ly - gap)
            }
        }
    }
}

fn estimate_arrow(attrs: &Attrs, anchors: &HashMap<String, Anchor>) -> Option<ComponentGeometry> {
    let (x1, y1) = resolve_position(attrs, "x", "y", Some("from"), "dx", "dy", anchors)?;
    let (x2, y2) = resolve_target(attrs, anchors)?;

    let label = attrs.get("label").map(String::as_str);
    let label_bbox = if let Some(text) = label {
        let mx = f64::midpoint(x1, x2);
        let my = f64::midpoint(y1, y2);
        // Approximate: label above midpoint
        label_bbox_at(mx, my - 10.0, text)
    } else {
        None
    };

    Some(ComponentGeometry {
        label_bbox,
        line_segments: vec![(x1, y1, x2, y2)],
    })
}

fn estimate_dimension(
    attrs: &Attrs,
    anchors: &HashMap<String, Anchor>,
) -> Option<ComponentGeometry> {
    let (x1, y1) = resolve_position(attrs, "x", "y", Some("from"), "dx", "dy", anchors)?;
    let (x2, y2) = resolve_target(attrs, anchors)?;
    let label = attr_str(attrs, "label", "");
    let side_offset = 20.0;

    // Simplified: dimension line is offset perpendicular to the from-to vector
    let dx = x2 - x1;
    let dy = y2 - y1;
    let len = (dx * dx + dy * dy).sqrt();
    let (nx, ny) = if len > 0.0 {
        (-dy / len, dx / len)
    } else {
        (0.0, -1.0)
    };

    let side = attr_str(attrs, "side", "above");
    let (snx, sny) = match side {
        "above" | "right" => (-nx, -ny),
        _ => (nx, ny),
    };

    let ox = snx * side_offset;
    let oy = sny * side_offset;
    let dx1 = x1 + ox;
    let dy1 = y1 + oy;
    let dx2 = x2 + ox;
    let dy2 = y2 + oy;

    let mut segments = vec![
        (dx1, dy1, dx2, dy2), // main dimension line
        (x1, y1, dx1, dy1),   // extension line 1
        (x2, y2, dx2, dy2),   // extension line 2
    ];
    // Cap lines
    let cap_h = 4.0;
    let cnx = nx * cap_h;
    let cny = ny * cap_h;
    segments.push((dx1 - cnx, dy1 - cny, dx1 + cnx, dy1 + cny));
    segments.push((dx2 - cnx, dy2 - cny, dx2 + cnx, dy2 + cny));

    let label_bbox = if label.is_empty() {
        None
    } else {
        let mx = f64::midpoint(dx1, dx2);
        let my = f64::midpoint(dy1, dy2);
        label_bbox_at(mx, my - 10.0, label)
    };

    Some(ComponentGeometry {
        label_bbox,
        line_segments: segments,
    })
}

fn estimate_scale_bar(
    attrs: &Attrs,
    anchors: &HashMap<String, Anchor>,
) -> Option<ComponentGeometry> {
    let (x, y) = resolve_position(attrs, "x", "y", Some("at"), "dx", "dy", anchors)?;
    let length = attr_f64(attrs, "length")?;
    let label = attr_str(attrs, "label", "");
    let side = attr_str(attrs, "side", "bottom");
    let cap_h = 4.0;

    let vertical = matches!(side, "left" | "right");
    let mut segments = if vertical {
        vec![
            (x, y, x, y + length),
            (x - cap_h, y, x + cap_h, y),
            (x - cap_h, y + length, x + cap_h, y + length),
        ]
    } else {
        vec![
            (x, y, x + length, y),
            (x, y - cap_h, x, y + cap_h),
            (x + length, y - cap_h, x + length, y + cap_h),
        ]
    };
    let _ = &mut segments; // suppress unused_mut if no more pushes

    let label_bbox = if label.is_empty() {
        None
    } else if vertical {
        label_bbox_at(x + 8.0, y + length / 2.0, label)
    } else {
        label_bbox_at(x + length / 2.0, y + 16.0, label)
    };

    Some(ComponentGeometry {
        label_bbox,
        line_segments: segments,
    })
}

fn estimate_brace(attrs: &Attrs, anchors: &HashMap<String, Anchor>) -> Option<ComponentGeometry> {
    let (x1, y1) = resolve_position(attrs, "x", "y", Some("from"), "dx", "dy", anchors)?;
    let (x2, y2) = resolve_target(attrs, anchors)?;
    let label = attrs.get("label").map(String::as_str);

    // Approximate the brace as its chord line
    let label_bbox = if let Some(text) = label {
        let mx = f64::midpoint(x1, x2);
        let my = f64::midpoint(y1, y2);
        label_bbox_at(mx, my - 12.0, text)
    } else {
        None
    };

    Some(ComponentGeometry {
        label_bbox,
        line_segments: vec![(x1, y1, x2, y2)],
    })
}

fn estimate_bracket(attrs: &Attrs, anchors: &HashMap<String, Anchor>) -> Option<ComponentGeometry> {
    let (x1, y1) = resolve_position(attrs, "x", "y", Some("from"), "dx", "dy", anchors)?;
    let (x2, y2) = resolve_target(attrs, anchors)?;
    let label = attrs.get("label").map(String::as_str);

    let label_bbox = if let Some(text) = label {
        let mx = f64::midpoint(x1, x2);
        let my = f64::midpoint(y1, y2);
        label_bbox_at(mx, my - 12.0, text)
    } else {
        None
    };

    Some(ComponentGeometry {
        label_bbox,
        line_segments: vec![(x1, y1, x2, y2)],
    })
}

fn estimate_marker(attrs: &Attrs, anchors: &HashMap<String, Anchor>) -> Option<ComponentGeometry> {
    let (x, y) = resolve_position(attrs, "x", "y", Some("at"), "dx", "dy", anchors)?;
    let size = attr_f64_or(attrs, "size", 20.0);
    let label = attr_str(attrs, "label", "");
    let label_position = attr_str(attrs, "label-position", "right");

    let label_bbox = if label.is_empty() {
        None
    } else {
        let (lx, ly) = match label_position {
            "above" => (x, y - size / 2.0 - 4.0),
            "below" => (x, y + size / 2.0 + 14.0),
            "left" => (x - size / 2.0 - 4.0, y + 4.0),
            _ => (x + size / 2.0 + 4.0, y + 4.0),
        };
        label_bbox_at(lx, ly, label)
    };

    // No line segments for marker — it's a symbol stamp
    Some(ComponentGeometry {
        label_bbox,
        line_segments: vec![],
    })
}

fn estimate_compass(attrs: &Attrs, anchors: &HashMap<String, Anchor>) -> Option<ComponentGeometry> {
    let (x, y) = resolve_position(attrs, "x", "y", Some("at"), "dx", "dy", anchors)?;
    let size = attr_f64_or(attrs, "size", 50.0);
    let r = size / 2.0;
    let variant = attr_str(attrs, "variant", "arrow");

    let mut segments = if variant == "full" {
        vec![
            (x, y - r, x, y + r), // vertical axis
            (x - r, y, x + r, y), // horizontal axis
        ]
    } else {
        vec![(x, y - r, x, y + r * 0.3)] // single arrow
    };
    let _ = &mut segments;

    // Label "N" above
    let label_bbox = label_bbox_at(x, y - r - 4.0, "N");

    Some(ComponentGeometry {
        label_bbox,
        line_segments: segments,
    })
}

fn estimate_crosshair(
    attrs: &Attrs,
    anchors: &HashMap<String, Anchor>,
) -> Option<ComponentGeometry> {
    let (cx, cy) = resolve_position(attrs, "cx", "cy", Some("at"), "dx", "dy", anchors)?;
    let size = attr_f64_or(attrs, "size", 20.0);
    let gap = attr_f64_or(attrs, "gap", 4.0);
    let label = attr_str(attrs, "label", "");
    let label_position = attr_str(attrs, "label-position", "right");

    let segments = vec![
        (cx - size, cy, cx - gap, cy),
        (cx + gap, cy, cx + size, cy),
        (cx, cy - size, cx, cy - gap),
        (cx, cy + gap, cx, cy + size),
    ];

    let label_bbox = if label.is_empty() {
        None
    } else {
        let (lx, ly) = match label_position {
            "above" => (cx, cy - size - 6.0),
            "below" => (cx, cy + size + 14.0),
            "left" => (cx - size - 6.0, cy + 4.0),
            _ => (cx + size + 6.0, cy + 4.0),
        };
        label_bbox_at(lx, ly, label)
    };

    Some(ComponentGeometry {
        label_bbox,
        line_segments: segments,
    })
}

fn estimate_angle(attrs: &Attrs, anchors: &HashMap<String, Anchor>) -> Option<ComponentGeometry> {
    let (vx, vy) = resolve_position(attrs, "x", "y", Some("at"), "dx", "dy", anchors)?;
    let r = attr_f64_or(attrs, "r", 30.0);
    let label = attrs.get("label").map(String::as_str);

    // Approximate the arc as a bounding box around the vertex
    let label_bbox = if let Some(text) = label {
        label_bbox_at(vx + r + 12.0, vy, text)
    } else {
        None
    };

    // No line segments — arcs are hard to represent as line segments for collision
    Some(ComponentGeometry {
        label_bbox,
        line_segments: vec![],
    })
}

fn estimate_roi_rect(
    attrs: &Attrs,
    anchors: &HashMap<String, Anchor>,
) -> Option<ComponentGeometry> {
    let (x, y, w, h) = if let (Some(x), Some(y)) = (attr_f64(attrs, "x"), attr_f64(attrs, "y")) {
        let w = attr_f64_or(attrs, "width", 50.0);
        let h = attr_f64_or(attrs, "height", 50.0);
        (x, y, w, h)
    } else if let (Some(start), Some(end)) = (
        resolve_position(attrs, "x", "y", Some("from"), "dx", "dy", anchors),
        resolve_target(attrs, anchors),
    ) {
        let x = start.0.min(end.0);
        let y = start.1.min(end.1);
        (x, y, (end.0 - start.0).abs(), (end.1 - start.1).abs())
    } else {
        return None;
    };

    let label = attr_str(attrs, "label", "");
    let label_position = attr_str(attrs, "label-position", "above");

    let segments = vec![
        (x, y, x + w, y),         // top
        (x + w, y, x + w, y + h), // right
        (x + w, y + h, x, y + h), // bottom
        (x, y + h, x, y),         // left
    ];

    let label_bbox = if label.is_empty() {
        None
    } else {
        let (lx, ly) = match label_position {
            "below" => (x + w / 2.0, y + h + 16.0),
            "center" => (x + w / 2.0, y + h / 2.0),
            "left" => (x - 8.0, y + h / 2.0),
            "right" => (x + w + 8.0, y + h / 2.0),
            _ => (x + w / 2.0, y - 8.0),
        };
        label_bbox_at(lx, ly, label)
    };

    Some(ComponentGeometry {
        label_bbox,
        line_segments: segments,
    })
}

fn estimate_roi_ellipse(
    attrs: &Attrs,
    anchors: &HashMap<String, Anchor>,
) -> Option<ComponentGeometry> {
    let (cx, cy) = resolve_position(attrs, "cx", "cy", Some("at"), "dx", "dy", anchors)?;
    let rx = attr_f64(attrs, "rx")?;
    let ry = attr_f64(attrs, "ry")?;

    let label = attr_str(attrs, "label", "");
    let label_position = attr_str(attrs, "label-position", "above");

    // Approximate ellipse as 4 line segments (bounding diamond)
    let segments = vec![
        (cx, cy - ry, cx + rx, cy),
        (cx + rx, cy, cx, cy + ry),
        (cx, cy + ry, cx - rx, cy),
        (cx - rx, cy, cx, cy - ry),
    ];

    let label_bbox = if label.is_empty() {
        None
    } else {
        let (lx, ly) = match label_position {
            "below" => (cx, cy + ry + 16.0),
            "center" => (cx, cy),
            "left" => (cx - rx - 8.0, cy),
            "right" => (cx + rx + 8.0, cy),
            _ => (cx, cy - ry - 8.0),
        };
        label_bbox_at(lx, ly, label)
    };

    Some(ComponentGeometry {
        label_bbox,
        line_segments: segments,
    })
}

fn estimate_roi_polygon(attrs: &Attrs) -> Option<ComponentGeometry> {
    let points_str = attrs.get("points")?;
    let points: Vec<(f64, f64)> = points_str
        .split_whitespace()
        .filter_map(|pair| {
            let mut parts = pair.split(',');
            let x = parts.next()?.parse::<f64>().ok()?;
            let y = parts.next()?.parse::<f64>().ok()?;
            Some((x, y))
        })
        .collect();

    if points.len() < 3 {
        return None;
    }

    let label = attr_str(attrs, "label", "");
    let label_position = attr_str(attrs, "label-position", "above");

    // Polygon edges as line segments
    let mut segments = Vec::with_capacity(points.len());
    for i in 0..points.len() {
        let j = (i + 1) % points.len();
        segments.push((points[i].0, points[i].1, points[j].0, points[j].1));
    }

    let label_bbox = if label.is_empty() {
        None
    } else {
        let min_x = points.iter().map(|(x, _)| *x).fold(f64::INFINITY, f64::min);
        let max_x = points
            .iter()
            .map(|(x, _)| *x)
            .fold(f64::NEG_INFINITY, f64::max);
        let min_y = points.iter().map(|(_, y)| *y).fold(f64::INFINITY, f64::min);
        let max_y = points
            .iter()
            .map(|(_, y)| *y)
            .fold(f64::NEG_INFINITY, f64::max);
        let w = max_x - min_x;
        let h = max_y - min_y;
        let (lx, ly) = match label_position {
            "below" => (min_x + w / 2.0, max_y + 16.0),
            "center" => (min_x + w / 2.0, min_y + h / 2.0),
            _ => (min_x + w / 2.0, min_y - 8.0),
        };
        label_bbox_at(lx, ly, label)
    };

    Some(ComponentGeometry {
        label_bbox,
        line_segments: segments,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn attrs(pairs: &[(&str, &str)]) -> Attrs {
        pairs
            .iter()
            .map(|(k, v)| ((*k).to_string(), (*v).to_string()))
            .collect()
    }

    #[test]
    fn badge_geometry() {
        let a = attrs(&[("x", "100"), ("y", "50"), ("label", "Hello")]);
        let g = estimate_geometry("badge", &a, &HashMap::new());
        assert!(g.is_some());
        let g = g.expect("should have geometry");
        assert!(g.label_bbox.is_some());
        assert!(g.line_segments.is_empty());
    }

    #[test]
    fn arrow_geometry() {
        let a = attrs(&[("x", "10"), ("y", "10"), ("to-x", "100"), ("to-y", "100")]);
        let g = estimate_geometry("arrow", &a, &HashMap::new());
        assert!(g.is_some());
        let g = g.expect("should have geometry");
        assert!(g.label_bbox.is_none()); // no label
        assert_eq!(g.line_segments.len(), 1);
    }

    #[test]
    fn callout_with_leader() {
        let a = attrs(&[
            ("x", "100"),
            ("y", "50"),
            ("label", "Note"),
            ("to-x", "200"),
            ("to-y", "200"),
        ]);
        let g = estimate_geometry("callout", &a, &HashMap::new());
        assert!(g.is_some());
        let g = g.expect("should have geometry");
        assert!(g.label_bbox.is_some());
        assert_eq!(g.line_segments.len(), 1); // leader line
    }

    #[test]
    fn spotlight_excluded() {
        let a = attrs(&[("cx", "100"), ("cy", "100"), ("r", "50")]);
        assert!(estimate_geometry("spotlight", &a, &HashMap::new()).is_none());
    }

    #[test]
    fn halo_excluded() {
        let a = attrs(&[("cx", "100"), ("cy", "100"), ("r", "20")]);
        assert!(estimate_geometry("halo", &a, &HashMap::new()).is_none());
    }

    #[test]
    fn roi_rect_geometry() {
        let a = attrs(&[
            ("x", "10"),
            ("y", "10"),
            ("width", "100"),
            ("height", "50"),
            ("label", "Region"),
        ]);
        let g = estimate_geometry("roi-rect", &a, &HashMap::new());
        assert!(g.is_some());
        let g = g.expect("should have geometry");
        assert!(g.label_bbox.is_some());
        assert_eq!(g.line_segments.len(), 4); // 4 edges
    }
}
