use std::collections::{HashMap, HashSet};

use quick_xml::Reader;
use quick_xml::events::Event;

use crate::anchors::{Anchor, collect_anchors, resolve_anchor_ref};
use crate::bbox::BBox;
use crate::compile::parse_element_attrs;
use crate::component_attrs::{is_valid_attr, validate_enum_attr};
use crate::component_bbox::{ComponentGeometry, estimate_geometry};
use crate::diagnostics::CompilationMessage;

/// The result of linting an SVG overlay.
#[derive(Debug)]
pub struct LintResult {
    pub messages: Vec<CompilationMessage>,
}

/// A collected component with its geometry and metadata for post-pass analysis.
struct LintedComponent {
    name: String,
    geometry: Option<ComponentGeometry>,
    position: Option<(f64, f64)>,
    suppress_collision: bool,
    /// Byte offset of the element start in the source, for location in spatial warnings.
    byte_offset: usize,
}

/// Lint an SVG overlay, checking for layout, reference, and attribute issues.
///
/// This performs static analysis without rendering. It checks:
/// 1. Text/text collisions (overlapping label bounding boxes)
/// 2. Text/line collisions (labels overlapping line segments)
/// 3. Out-of-bounds components (outside viewBox)
/// 4. Anchor crowding (too many components at the same position)
/// 5. Dangling anchor references
/// 6. Unused anchors
/// 7. Missing xmlns:s namespace
/// 8. Unknown attributes on components
/// 9. Invalid enum attribute values
#[must_use]
pub fn lint(source: &str) -> LintResult {
    let mut messages = Vec::new();

    // Collect anchors (reuses the same logic as compile)
    let anchors = collect_anchors(source, &mut messages);

    // Extract viewBox for out-of-bounds checking
    let viewbox_bbox = extract_viewbox_bbox(source);
    if viewbox_bbox.is_none() && source.contains("<s:") {
        messages.push(CompilationMessage::warning(
            "SVG has no viewBox attribute; out-of-bounds checking is disabled",
        ));
    }

    // Check for xmlns:s namespace
    check_namespace(source, &mut messages);

    // Parse all s: elements and collect geometry + references
    let mut components = Vec::new();
    let mut referenced_anchors: HashSet<String> = HashSet::new();
    let mut defined_anchors: HashSet<String> = HashSet::new();

    // Track which anchors were explicitly defined (not auto-generated)
    collect_defined_anchor_ids(source, &mut defined_anchors);

    // Parse and validate each s: element
    parse_s_elements(
        source,
        &anchors,
        &mut messages,
        &mut components,
        &mut referenced_anchors,
    );

    // --- Post-pass analysis ---

    // Rule 5: Unused anchors (defined but never referenced)
    for anchor_id in &defined_anchors {
        if !referenced_anchors.contains(anchor_id) {
            messages.push(CompilationMessage::warning(format!(
                "Anchor '{anchor_id}' is defined but never referenced"
            )));
        }
    }

    // Rules 1a, 1b, 2, 3: Spatial checks
    check_spatial(source, &components, viewbox_bbox.as_ref(), &mut messages);

    LintResult { messages }
}

fn extract_viewbox_bbox(source: &str) -> Option<BBox> {
    let mut reader = Reader::from_str(source);
    loop {
        match reader.read_event() {
            Ok(Event::Start(ref e) | Event::Empty(ref e)) => {
                let local = e.local_name();
                if local.as_ref() == b"svg" {
                    for attr in e.attributes().flatten() {
                        if attr.key.as_ref() == b"viewBox" {
                            let value = String::from_utf8_lossy(&attr.value);
                            let parts: Vec<f64> = value
                                .split_whitespace()
                                .filter_map(|p| p.parse().ok())
                                .collect();
                            if parts.len() == 4 {
                                return Some(BBox::new(parts[0], parts[1], parts[2], parts[3]));
                            }
                        }
                    }
                }
                return None;
            }
            Ok(Event::Eof) | Err(_) => return None,
            _ => {}
        }
    }
}

fn check_namespace(source: &str, messages: &mut Vec<CompilationMessage>) {
    // Only check if there are s: elements to lint
    if !source.contains("<s:") {
        return;
    }

    let mut reader = Reader::from_str(source);
    loop {
        let offset = reader.buffer_position() as usize;
        match reader.read_event() {
            Ok(Event::Start(ref e) | Event::Empty(ref e)) => {
                let local = e.local_name();
                if local.as_ref() == b"svg" {
                    for attr in e.attributes().flatten() {
                        let key = String::from_utf8_lossy(attr.key.as_ref());
                        if key == "xmlns:s" {
                            return; // Found it
                        }
                    }
                    messages.push(
                        CompilationMessage::warning(
                            "Missing xmlns:s namespace declaration on <svg> element; \
                             add xmlns:s=\"https://stencila.io/svg\" for proper rendering",
                        )
                        .at_offset(source, offset),
                    );
                    return;
                }
            }
            Ok(Event::Eof) | Err(_) => return,
            _ => {}
        }
    }
}

fn collect_defined_anchor_ids(source: &str, defined: &mut HashSet<String>) {
    let mut reader = Reader::from_str(source);
    loop {
        match reader.read_event() {
            Ok(Event::Start(ref e) | Event::Empty(ref e)) => {
                let name = element_name(e);
                if name == "s:anchor" {
                    let attrs = parse_element_attrs(e);
                    if let Some(id) = attrs.get("id") {
                        defined.insert(id.clone());
                    }
                }
            }
            Ok(Event::Eof) | Err(_) => break,
            _ => {}
        }
    }
}

fn element_name(e: &quick_xml::events::BytesStart) -> String {
    let binding = e.name();
    String::from_utf8_lossy(binding.as_ref()).to_string()
}

fn parse_s_elements(
    source: &str,
    anchors: &HashMap<String, Anchor>,
    messages: &mut Vec<CompilationMessage>,
    components: &mut Vec<LintedComponent>,
    referenced_anchors: &mut HashSet<String>,
) {
    let mut reader = Reader::from_str(source);
    let mut suppress_next_collision = false;

    loop {
        // Capture position before reading — this is the byte offset of the
        // start of the next event in the source string.
        let event_offset = reader.buffer_position() as usize;

        match reader.read_event() {
            Ok(Event::Comment(ref e)) => {
                let text = String::from_utf8_lossy(e.as_ref());
                let trimmed = text.trim();
                if trimmed == "lint-ignore collision" {
                    suppress_next_collision = true;
                }
            }
            Ok(Event::Start(ref e) | Event::Empty(ref e)) => {
                let name = element_name(e);
                if let Some(component_name) = name.strip_prefix("s:") {
                    if component_name == "anchor" {
                        suppress_next_collision = false;
                        continue;
                    }

                    let attrs = parse_element_attrs(e);

                    // Rule 7 & 8: Validate attributes (sorted for deterministic output)
                    let mut keys: Vec<&String> = attrs.keys().collect();
                    keys.sort();
                    for key in keys {
                        if !is_valid_attr(component_name, key) {
                            messages.push(
                                CompilationMessage::warning(format!(
                                    "<s:{component_name}> has unknown attribute '{key}'"
                                ))
                                .at_offset(source, event_offset),
                            );
                        }
                        if let Some(value) = attrs.get(key.as_str())
                            && let Some(valid_values) =
                                validate_enum_attr(component_name, key, value)
                        {
                            messages.push(
                                    CompilationMessage::warning(format!(
                                        "<s:{component_name}> attribute '{key}' has invalid value '{value}'; \
                                         expected one of: {}",
                                        valid_values.join(", ")
                                    ))
                                    .at_offset(source, event_offset),
                                );
                        }
                    }

                    // Rule 4: Check anchor references exist
                    collect_and_check_anchor_refs(
                        component_name,
                        &attrs,
                        anchors,
                        messages,
                        referenced_anchors,
                        source,
                        event_offset,
                    );

                    // Estimate geometry for spatial checks
                    let geometry = estimate_geometry(component_name, &attrs, anchors);
                    let position = resolve_primary_position(component_name, &attrs, anchors);

                    components.push(LintedComponent {
                        name: component_name.to_string(),
                        geometry,
                        position,
                        suppress_collision: suppress_next_collision,
                        byte_offset: event_offset,
                    });

                    suppress_next_collision = false;
                }
                // Standard SVG elements do NOT clear the suppress flag —
                // only s:* elements consume it. This allows constructs like:
                //   <!-- lint-ignore collision -->
                //   <g>
                //     <s:badge .../>
                //   </g>
            }
            Ok(Event::Eof) | Err(_) => break,
            _ => {
                // Non-comment, non-element events don't clear suppress flag
            }
        }
    }
}

/// Resolve the primary position of a component for crowding analysis.
fn resolve_primary_position(
    name: &str,
    attrs: &crate::components::Attrs,
    anchors: &HashMap<String, Anchor>,
) -> Option<(f64, f64)> {
    use crate::components::{resolve_position, resolve_target};

    match name {
        "badge" | "callout" => resolve_position(attrs, "x", "y", Some("from"), "dx", "dy", anchors),
        "arrow" | "dimension" | "brace" | "bracket" => {
            // Use midpoint of from-to
            let start = resolve_position(attrs, "x", "y", Some("from"), "dx", "dy", anchors);
            let end = resolve_target(attrs, anchors);
            match (start, end) {
                (Some((x1, y1)), Some((x2, y2))) => {
                    Some((f64::midpoint(x1, x2), f64::midpoint(y1, y2)))
                }
                (Some(pos), None) | (None, Some(pos)) => Some(pos),
                _ => None,
            }
        }
        "compass" | "marker" | "scale-bar" => {
            resolve_position(attrs, "x", "y", Some("at"), "dx", "dy", anchors)
        }
        "crosshair" | "halo" | "spotlight" | "roi-ellipse" => {
            resolve_position(attrs, "cx", "cy", Some("at"), "dx", "dy", anchors)
        }
        "angle" => resolve_position(attrs, "x", "y", Some("at"), "dx", "dy", anchors),
        _ => None,
    }
}

fn collect_and_check_anchor_refs(
    component_name: &str,
    attrs: &crate::components::Attrs,
    anchors: &HashMap<String, Anchor>,
    messages: &mut Vec<CompilationMessage>,
    referenced: &mut HashSet<String>,
    source: &str,
    byte_offset: usize,
) {
    // Attributes that can contain anchor references
    let ref_attrs = ["from", "to", "at"];

    for attr_name in &ref_attrs {
        if let Some(value) = attrs.get(*attr_name)
            && let Some(name) = value.strip_prefix('#')
        {
            referenced.insert(name.to_string());
            if resolve_anchor_ref(value, anchors, 0.0, 0.0).is_none() {
                messages.push(
                    CompilationMessage::error(format!(
                        "<s:{component_name}> references anchor '{value}' which does not exist"
                    ))
                    .at_offset(source, byte_offset),
                );
            }
        }
    }
}

fn check_spatial(
    source: &str,
    components: &[LintedComponent],
    viewbox: Option<&BBox>,
    messages: &mut Vec<CompilationMessage>,
) {
    check_collisions(source, components, messages);
    check_bounds(source, components, viewbox, messages);
    check_crowding(source, components, messages);
}

fn check_collisions(
    source: &str,
    components: &[LintedComponent],
    messages: &mut Vec<CompilationMessage>,
) {
    let n = components.len();

    // Rule 1a: Text/text collision
    for i in 0..n {
        if components[i].suppress_collision {
            continue;
        }
        let Some(ref gi) = components[i].geometry else {
            continue;
        };
        let Some(ref bbox_i) = gi.label_bbox else {
            continue;
        };

        for j in (i + 1)..n {
            if components[j].suppress_collision {
                continue;
            }
            let Some(ref gj) = components[j].geometry else {
                continue;
            };
            if let Some(ref bbox_j) = gj.label_bbox
                && bbox_i.intersects(bbox_j)
            {
                messages.push(
                    CompilationMessage::warning(format!(
                        "Label collision: <s:{}> and <s:{}> labels overlap",
                        components[i].name, components[j].name
                    ))
                    .at_offset(source, components[i].byte_offset),
                );
            }
        }
    }

    // Rule 1b: Text/line collision (label of one component overlaps line of another)
    for i in 0..n {
        if components[i].suppress_collision {
            continue;
        }
        let Some(ref gi) = components[i].geometry else {
            continue;
        };
        let Some(ref label_bbox) = gi.label_bbox else {
            continue;
        };

        for j in 0..n {
            if i == j || components[j].suppress_collision {
                continue;
            }
            let Some(ref gj) = components[j].geometry else {
                continue;
            };
            for &(x1, y1, x2, y2) in &gj.line_segments {
                if label_bbox.intersects_line(x1, y1, x2, y2) {
                    messages.push(
                        CompilationMessage::warning(format!(
                            "Text/line collision: <s:{}> label overlaps <s:{}> line",
                            components[i].name, components[j].name
                        ))
                        .at_offset(source, components[i].byte_offset),
                    );
                    break;
                }
            }
        }
    }
}

fn check_bounds(
    source: &str,
    components: &[LintedComponent],
    viewbox: Option<&BBox>,
    messages: &mut Vec<CompilationMessage>,
) {
    let Some(vb) = viewbox else { return };

    for comp in components {
        let Some(ref geom) = comp.geometry else {
            continue;
        };
        if let Some(ref bbox) = geom.label_bbox
            && !bbox.within(vb)
        {
            messages.push(
                CompilationMessage::warning(format!(
                    "<s:{}> label extends outside the viewBox",
                    comp.name
                ))
                .at_offset(source, comp.byte_offset),
            );
        }
        for &(x1, y1, x2, y2) in &geom.line_segments {
            if !vb.contains_point(x1, y1) || !vb.contains_point(x2, y2) {
                messages.push(
                    CompilationMessage::warning(format!(
                        "<s:{}> extends outside the viewBox",
                        comp.name
                    ))
                    .at_offset(source, comp.byte_offset),
                );
                break;
            }
        }
    }
}

fn check_crowding(
    source: &str,
    components: &[LintedComponent],
    messages: &mut Vec<CompilationMessage>,
) {
    let crowding_threshold = 5.0;
    let mut groups: Vec<(f64, f64, Vec<usize>)> = Vec::new();

    for (idx, comp) in components.iter().enumerate() {
        let Some((x, y)) = comp.position else {
            continue;
        };
        let mut found = false;
        for group in &mut groups {
            let dx = x - group.0;
            let dy = y - group.1;
            if (dx * dx + dy * dy).sqrt() <= crowding_threshold {
                group.2.push(idx);
                found = true;
                break;
            }
        }
        if !found {
            groups.push((x, y, vec![idx]));
        }
    }

    for group in &groups {
        if group.2.len() >= 3 {
            let names: Vec<&str> = group
                .2
                .iter()
                .map(|&i| components[i].name.as_str())
                .collect();
            let first_offset = components[group.2[0]].byte_offset;
            messages.push(
                CompilationMessage::warning(format!(
                    "Anchor crowding: {} components at approximately ({}, {}): {}",
                    group.2.len(),
                    group.0 as i64,
                    group.1 as i64,
                    names.join(", ")
                ))
                .at_offset(source, first_offset),
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::diagnostics::MessageLevel;

    #[test]
    fn clean_svg_no_warnings() {
        let svg = r#"<svg viewBox="0 0 600 400" xmlns:s="https://stencila.io/svg">
            <s:badge x="100" y="50" label="A"/>
            <s:badge x="400" y="350" label="B"/>
        </svg>"#;
        let result = lint(svg);
        assert!(
            result.messages.is_empty(),
            "Expected no messages, got: {:?}",
            result.messages
        );
    }

    #[test]
    fn missing_namespace() {
        let svg = r#"<svg viewBox="0 0 600 400">
            <s:badge x="100" y="50" label="Test"/>
        </svg>"#;
        let result = lint(svg);
        assert!(
            result
                .messages
                .iter()
                .any(|m| m.message.contains("xmlns:s"))
        );
    }

    #[test]
    fn dangling_anchor_ref() {
        let svg = r##"<svg viewBox="0 0 600 400" xmlns:s="https://stencila.io/svg">
            <s:callout from="#nonexistent" label="Oops"/>
        </svg>"##;
        let result = lint(svg);
        assert!(
            result
                .messages
                .iter()
                .any(|m| m.level == MessageLevel::Error && m.message.contains("nonexistent"))
        );
    }

    #[test]
    fn unused_anchor() {
        let svg = r#"<svg viewBox="0 0 600 400" xmlns:s="https://stencila.io/svg">
            <s:anchor id="unused-point" x="100" y="100"/>
            <s:badge x="300" y="200" label="Test"/>
        </svg>"#;
        let result = lint(svg);
        assert!(
            result
                .messages
                .iter()
                .any(|m| m.message.contains("unused-point"))
        );
    }

    #[test]
    fn unknown_attribute() {
        let svg = r#"<svg viewBox="0 0 600 400" xmlns:s="https://stencila.io/svg">
            <s:badge x="100" y="50" label="Test" bogus="true"/>
        </svg>"#;
        let result = lint(svg);
        assert!(result.messages.iter().any(|m| m.message.contains("bogus")));
    }

    #[test]
    fn invalid_enum_value() {
        let svg = r#"<svg viewBox="0 0 600 400" xmlns:s="https://stencila.io/svg">
            <s:arrow x="10" y="10" to-x="100" to-y="100" curve="wobbly"/>
        </svg>"#;
        let result = lint(svg);
        assert!(result.messages.iter().any(|m| m.message.contains("wobbly")));
    }

    #[test]
    fn label_collision() {
        let svg = r#"<svg viewBox="0 0 600 400" xmlns:s="https://stencila.io/svg">
            <s:badge x="100" y="50" label="First badge"/>
            <s:badge x="101" y="50" label="Second badge"/>
        </svg>"#;
        let result = lint(svg);
        assert!(
            result
                .messages
                .iter()
                .any(|m| m.message.contains("collision")),
            "Expected collision warning, got: {:?}",
            result.messages
        );
    }

    #[test]
    fn label_collision_suppressed() {
        let svg = r#"<svg viewBox="0 0 600 400" xmlns:s="https://stencila.io/svg">
            <!-- lint-ignore collision -->
            <s:badge x="100" y="50" label="First badge"/>
            <s:badge x="101" y="50" label="Second badge"/>
        </svg>"#;
        let result = lint(svg);
        assert!(
            !result
                .messages
                .iter()
                .any(|m| m.message.contains("collision")),
            "Expected no collision warning when suppressed, got: {:?}",
            result.messages
        );
    }

    #[test]
    fn suppression_survives_standard_svg_elements() {
        // The suppress flag should not be cleared by standard SVG elements
        // between the comment and the s:* element.
        let svg = r#"<svg viewBox="0 0 600 400" xmlns:s="https://stencila.io/svg">
            <!-- lint-ignore collision -->
            <g>
            <s:badge x="100" y="50" label="First badge"/>
            </g>
            <s:badge x="101" y="50" label="Second badge"/>
        </svg>"#;
        let result = lint(svg);
        assert!(
            !result
                .messages
                .iter()
                .any(|m| m.message.contains("collision")),
            "Suppression should survive intervening <g>, got: {:?}",
            result.messages
        );
    }

    #[test]
    fn out_of_bounds() {
        let svg = r#"<svg viewBox="0 0 100 100" xmlns:s="https://stencila.io/svg">
            <s:badge x="-50" y="50" label="Way off screen"/>
        </svg>"#;
        let result = lint(svg);
        assert!(
            result
                .messages
                .iter()
                .any(|m| m.message.contains("outside the viewBox")),
            "Expected out-of-bounds warning, got: {:?}",
            result.messages
        );
    }

    #[test]
    fn anchor_crowding() {
        let svg = r#"<svg viewBox="0 0 600 400" xmlns:s="https://stencila.io/svg">
            <s:badge x="100" y="100" label="A"/>
            <s:badge x="101" y="101" label="B"/>
            <s:badge x="102" y="100" label="C"/>
        </svg>"#;
        let result = lint(svg);
        assert!(
            result
                .messages
                .iter()
                .any(|m| m.message.contains("crowding")),
            "Expected crowding warning, got: {:?}",
            result.messages
        );
    }

    #[test]
    fn no_viewbox_warns() {
        let svg = r#"<svg xmlns:s="https://stencila.io/svg">
            <s:badge x="100" y="50" label="Test"/>
        </svg>"#;
        let result = lint(svg);
        assert!(
            result
                .messages
                .iter()
                .any(|m| m.message.contains("viewBox"))
        );
    }

    #[test]
    fn text_line_collision() {
        let svg = r#"<svg viewBox="0 0 600 400" xmlns:s="https://stencila.io/svg">
            <s:badge x="50" y="50" label="Label"/>
            <s:arrow x="0" y="50" to-x="100" to-y="50"/>
        </svg>"#;
        let result = lint(svg);
        assert!(
            result
                .messages
                .iter()
                .any(|m| m.message.contains("Text/line collision")),
            "Expected text/line collision, got: {:?}",
            result.messages
        );
    }

    #[test]
    fn used_anchor_not_flagged() {
        let svg = r##"<svg viewBox="0 0 600 400" xmlns:s="https://stencila.io/svg">
            <s:anchor id="peak" x="300" y="100"/>
            <s:callout from="#peak" dx="50" dy="-30" label="Peak"/>
        </svg>"##;
        let result = lint(svg);
        assert!(
            !result
                .messages
                .iter()
                .any(|m| m.message.contains("peak") && m.message.contains("never referenced")),
            "Anchor 'peak' should not be flagged as unused, got: {:?}",
            result.messages
        );
    }
}
