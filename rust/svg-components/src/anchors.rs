use std::collections::HashMap;

use quick_xml::events::BytesStart;

use crate::compile::parse_element_attrs;
use crate::diagnostics::CompilationMessage;

/// A resolved anchor point with concrete coordinates.
#[derive(Debug, Clone, Copy)]
pub struct Anchor {
    pub x: f64,
    pub y: f64,
}

/// Collect explicit `<s:anchor>` definitions and generate auto-anchors from the viewBox.
///
/// Explicit anchors are `<s:anchor id="name" x="..." y="..."/>` elements.
/// Auto-anchors are generated from the `viewBox` attribute:
/// - `#s:center` — center of the viewBox
/// - `#s:top-left` — origin
/// - `#s:top-right`, `#s:bottom-left`, `#s:bottom-right` — corners
/// - `#s:top-center`, `#s:bottom-center`, `#s:mid-left`, `#s:mid-right` — edge midpoints
/// - `#s:origin` — viewBox origin (same as top-left)
pub fn collect_anchors(
    svg_content: &str,
    messages: &mut Vec<CompilationMessage>,
) -> HashMap<String, Anchor> {
    let mut anchors = HashMap::new();

    // Parse viewBox for auto-anchors
    if let Some(viewbox) = extract_viewbox(svg_content) {
        let (vx, vy, vw, vh) = viewbox;
        let cx = vx + vw / 2.0;
        let cy = vy + vh / 2.0;

        anchors.insert("s:origin".to_string(), Anchor { x: vx, y: vy });
        anchors.insert("s:center".to_string(), Anchor { x: cx, y: cy });
        anchors.insert("s:top-left".to_string(), Anchor { x: vx, y: vy });
        anchors.insert("s:top-right".to_string(), Anchor { x: vx + vw, y: vy });
        anchors.insert("s:bottom-left".to_string(), Anchor { x: vx, y: vy + vh });
        anchors.insert(
            "s:bottom-right".to_string(),
            Anchor {
                x: vx + vw,
                y: vy + vh,
            },
        );
        anchors.insert("s:top-center".to_string(), Anchor { x: cx, y: vy });
        anchors.insert("s:bottom-center".to_string(), Anchor { x: cx, y: vy + vh });
        anchors.insert("s:mid-left".to_string(), Anchor { x: vx, y: cy });
        anchors.insert("s:mid-right".to_string(), Anchor { x: vx + vw, y: cy });
    }

    // Collect explicit <s:anchor> elements
    collect_explicit_anchors(svg_content, &mut anchors, messages);

    anchors
}

/// Extract `viewBox` from the root `<svg>` element.
fn extract_viewbox(svg_content: &str) -> Option<(f64, f64, f64, f64)> {
    use quick_xml::Reader;
    use quick_xml::events::Event;

    let mut reader = Reader::from_str(svg_content);
    loop {
        match reader.read_event() {
            Ok(Event::Start(ref e)) | Ok(Event::Empty(ref e)) => {
                let local = e.local_name();
                if local.as_ref() == b"svg" {
                    for attr in e.attributes().flatten() {
                        if attr.key.as_ref() == b"viewBox" {
                            let value = String::from_utf8_lossy(&attr.value);
                            return parse_viewbox(&value);
                        }
                    }
                }
                return None;
            }
            Ok(Event::Eof) => return None,
            Err(_) => return None,
            _ => {}
        }
    }
}

fn parse_viewbox(s: &str) -> Option<(f64, f64, f64, f64)> {
    let parts: Vec<f64> = s
        .split_whitespace()
        .filter_map(|p| p.parse().ok())
        .collect();
    if parts.len() == 4 {
        Some((parts[0], parts[1], parts[2], parts[3]))
    } else {
        None
    }
}

/// Collect explicit `<s:anchor>` elements from the SVG.
fn collect_explicit_anchors(
    svg_content: &str,
    anchors: &mut HashMap<String, Anchor>,
    messages: &mut Vec<CompilationMessage>,
) {
    use quick_xml::Reader;
    use quick_xml::events::Event;

    let mut reader = Reader::from_str(svg_content);
    loop {
        match reader.read_event() {
            Ok(Event::Start(ref e)) | Ok(Event::Empty(ref e)) => {
                let local = e.local_name();
                let name = String::from_utf8_lossy(local.as_ref()).to_string();
                let full = e.name();
                if full.as_ref() == b"s:anchor" || name == "anchor" && is_s_prefixed(e) {
                    let attrs = parse_element_attrs(e);
                    let id = attrs.get("id");
                    let x = attrs.get("x").and_then(|v| v.parse::<f64>().ok());
                    let y = attrs.get("y").and_then(|v| v.parse::<f64>().ok());

                    match (id, x, y) {
                        (Some(id), Some(x), Some(y)) => {
                            anchors.insert(id.clone(), Anchor { x, y });
                        }
                        (None, _, _) => {
                            messages.push(CompilationMessage::error(
                                "<s:anchor> missing required 'id' attribute",
                            ));
                        }
                        (Some(id), _, _) => {
                            messages.push(CompilationMessage::error(format!(
                                "<s:anchor id=\"{id}\"> missing required 'x' and/or 'y' attribute"
                            )));
                        }
                    }
                }
            }
            Ok(Event::Eof) => break,
            Err(_) => break,
            _ => {}
        }
    }
}

/// Resolve an anchor reference like `#name` into concrete coordinates.
/// Applies optional dx/dy offsets.
pub fn resolve_anchor_ref(
    anchor_ref: &str,
    anchors: &HashMap<String, Anchor>,
    dx: f64,
    dy: f64,
) -> Option<(f64, f64)> {
    let name = anchor_ref.strip_prefix('#')?;
    let anchor = anchors.get(name)?;
    Some((anchor.x + dx, anchor.y + dy))
}

fn is_s_prefixed(e: &BytesStart) -> bool {
    let binding = e.name();
    let full_name = String::from_utf8_lossy(binding.as_ref());
    full_name.starts_with("s:")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn auto_anchors_from_viewbox() -> Result<(), String> {
        let svg = r#"<svg viewBox="0 0 600 400" xmlns:s="https://stencila.io/svg"></svg>"#;
        let mut messages = Vec::new();
        let anchors = collect_anchors(svg, &mut messages);

        assert!(messages.is_empty());

        let center = anchors.get("s:center").ok_or("missing s:center")?;
        assert!((center.x - 300.0).abs() < f64::EPSILON);
        assert!((center.y - 200.0).abs() < f64::EPSILON);

        let tr = anchors.get("s:top-right").ok_or("missing s:top-right")?;
        assert!((tr.x - 600.0).abs() < f64::EPSILON);
        assert!((tr.y - 0.0).abs() < f64::EPSILON);

        let bl = anchors
            .get("s:bottom-left")
            .ok_or("missing s:bottom-left")?;
        assert!((bl.x - 0.0).abs() < f64::EPSILON);
        assert!((bl.y - 400.0).abs() < f64::EPSILON);

        Ok(())
    }

    #[test]
    fn explicit_anchor_collection() -> Result<(), String> {
        let svg = r#"<svg viewBox="0 0 600 400" xmlns:s="https://stencila.io/svg">
            <s:anchor id="peak" x="335" y="100"/>
        </svg>"#;
        let mut messages = Vec::new();
        let anchors = collect_anchors(svg, &mut messages);

        assert!(messages.is_empty());

        let peak = anchors.get("peak").ok_or("missing peak anchor")?;
        assert!((peak.x - 335.0).abs() < f64::EPSILON);
        assert!((peak.y - 100.0).abs() < f64::EPSILON);

        Ok(())
    }

    #[test]
    fn anchor_missing_id_produces_error() {
        use crate::diagnostics::MessageLevel;

        let svg = r#"<svg viewBox="0 0 600 400" xmlns:s="https://stencila.io/svg">
            <s:anchor x="100" y="200"/>
        </svg>"#;
        let mut messages = Vec::new();
        let _anchors = collect_anchors(svg, &mut messages);

        assert_eq!(messages.len(), 1);
        assert_eq!(messages[0].level, MessageLevel::Error);
        assert!(messages[0].message.contains("missing required 'id'"));
    }

    #[test]
    fn resolve_anchor_ref_with_offsets() -> Result<(), String> {
        let mut anchors = HashMap::new();
        anchors.insert("peak".to_string(), Anchor { x: 335.0, y: 100.0 });

        let (x, y) =
            resolve_anchor_ref("#peak", &anchors, 125.0, -45.0).ok_or("failed to resolve")?;
        assert!((x - 460.0).abs() < f64::EPSILON);
        assert!((y - 55.0).abs() < f64::EPSILON);

        Ok(())
    }

    #[test]
    fn explicit_anchor_collection_unescapes_entities() -> Result<(), String> {
        let svg = r#"<svg viewBox="0 0 10 10" xmlns:s="https://stencila.io/svg">
            <s:anchor id="peak&amp;crest" x="1" y="2"/>
        </svg>"#;
        let mut messages = Vec::new();
        let anchors = collect_anchors(svg, &mut messages);

        assert!(messages.is_empty());
        let peak = anchors
            .get("peak&crest")
            .ok_or("missing unescaped anchor id")?;
        assert!((peak.x - 1.0).abs() < f64::EPSILON);
        assert!((peak.y - 2.0).abs() < f64::EPSILON);

        Ok(())
    }
}
