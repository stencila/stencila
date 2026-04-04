use std::collections::HashSet;

use quick_xml::Reader;
use quick_xml::events::Event;

/// Built-in defs definitions keyed by their id.
///
/// Each entry is `(id, svg_fragment)` where the fragment is a complete
/// `<marker>` or `<symbol>` element to be placed inside `<defs>`.
const BUILTIN_DEFS: &[(&str, &str)] = &[
    (
        "s:arrow-closed",
        r#"<marker id="s:arrow-closed" viewBox="0 0 10 10" refX="10" refY="5" markerWidth="8" markerHeight="8" orient="auto-start-reverse"><path d="M 0 0 L 10 5 L 0 10 z" fill="currentColor" stroke="none"/></marker>"#,
    ),
    (
        "s:arrow-open",
        r#"<marker id="s:arrow-open" viewBox="0 0 10 10" refX="10" refY="5" markerWidth="8" markerHeight="8" orient="auto-start-reverse"><path d="M 0 0 L 10 5 L 0 10" fill="none" stroke="currentColor" stroke-width="1.5"/></marker>"#,
    ),
    (
        "s:arrow-dot",
        r#"<marker id="s:arrow-dot" viewBox="0 0 10 10" refX="5" refY="5" markerWidth="6" markerHeight="6"><circle cx="5" cy="5" r="4" fill="currentColor" stroke="none"/></marker>"#,
    ),
    (
        "s:marker-circle",
        r#"<symbol id="s:marker-circle" viewBox="0 0 20 20"><circle cx="10" cy="10" r="8" fill="none" stroke="currentColor" stroke-width="2"/></symbol>"#,
    ),
    (
        "s:marker-cross",
        r#"<symbol id="s:marker-cross" viewBox="0 0 20 20"><line x1="4" y1="4" x2="16" y2="16" stroke="currentColor" stroke-width="2"/><line x1="16" y1="4" x2="4" y2="16" stroke="currentColor" stroke-width="2"/></symbol>"#,
    ),
    (
        "s:marker-pin",
        r#"<symbol id="s:marker-pin" viewBox="0 0 20 30"><path d="M10 0 C4.5 0 0 4.5 0 10 C0 18 10 30 10 30 C10 30 20 18 20 10 C20 4.5 15.5 0 10 0 Z" fill="currentColor"/><circle cx="10" cy="10" r="4" fill="white"/></symbol>"#,
    ),
    (
        "s:marker-star",
        r#"<symbol id="s:marker-star" viewBox="0 0 20 20"><polygon points="10,1 12.5,7.5 19,7.5 14,12 16,19 10,15 4,19 6,12 1,7.5 7.5,7.5" fill="currentColor"/></symbol>"#,
    ),
    (
        "s:cap-line",
        r#"<marker id="s:cap-line" viewBox="0 0 2 10" refX="1" refY="5" markerWidth="2" markerHeight="10" orient="auto"><line x1="1" y1="0" x2="1" y2="10" stroke="currentColor" stroke-width="1.5"/></marker>"#,
    ),
];

/// Scan SVG content for references to built-in defs and return
/// the set of referenced def ids.
pub fn scan_refs(svg_content: &str) -> HashSet<String> {
    let mut refs = HashSet::new();

    // Scan for url(#s:...) references
    let mut pos = 0;
    while let Some(start) = svg_content[pos..].find("url(#s:") {
        let abs_start = pos + start + 4; // skip "url("
        if let Some(end) = svg_content[abs_start..].find(')') {
            let id = &svg_content[abs_start + 1..abs_start + end]; // skip '#'
            refs.insert(id.to_string());
        }
        pos = abs_start + 1;
    }

    // Scan for href="#s:..." references
    let mut pos = 0;
    while let Some(start) = svg_content[pos..].find("href=\"#s:") {
        let abs_start = pos + start + 6; // skip href="
        if let Some(end) = svg_content[abs_start..].find('"') {
            let id = &svg_content[abs_start + 1..abs_start + end]; // skip '#'
            refs.insert(id.to_string());
        }
        pos = abs_start + 1;
    }

    // Also scan for xlink:href="#s:..." references (legacy SVG)
    let mut pos = 0;
    while let Some(start) = svg_content[pos..].find("xlink:href=\"#s:") {
        let abs_start = pos + start + 12; // skip xlink:href="
        if let Some(end) = svg_content[abs_start..].find('"') {
            let id = &svg_content[abs_start + 1..abs_start + end];
            refs.insert(id.to_string());
        }
        pos = abs_start + 1;
    }

    refs
}

/// Check if the SVG already has a `<defs>` section (to determine where to inject).
fn has_existing_defs(svg_content: &str) -> bool {
    let mut reader = Reader::from_str(svg_content);
    let mut depth = 0u32;
    loop {
        match reader.read_event() {
            Ok(Event::Start(ref e)) => {
                depth += 1;
                let local = e.local_name();
                if depth == 2 && local.as_ref() == b"defs" {
                    return true;
                }
            }
            Ok(Event::End(_)) => {
                depth = depth.saturating_sub(1);
            }
            Ok(Event::Eof) => return false,
            Err(_) => return false,
            _ => {}
        }
    }
}

/// Generate the `<defs>` block containing only the referenced built-in definitions.
pub fn generate_defs_block(referenced: &HashSet<String>) -> Option<String> {
    let mut defs_content = String::new();

    for &(id, fragment) in BUILTIN_DEFS {
        if referenced.contains(id) {
            defs_content.push_str(fragment);
        }
    }

    if defs_content.is_empty() {
        None
    } else {
        Some(format!("<defs>{defs_content}</defs>"))
    }
}

/// Inject the built-in defs block into the SVG, right after the opening `<svg>` tag.
///
/// If a `<defs>` already exists, the built-in definitions are prepended inside it.
/// If no `<defs>` exists, a new one is inserted after the `<svg>` opening tag.
pub fn inject_defs(svg_content: &str, defs_block: &str) -> String {
    if has_existing_defs(svg_content) {
        // Insert after the opening <defs> tag
        if let Some(pos) = svg_content.find("<defs>") {
            let insert_pos = pos + 6;
            let mut result = String::with_capacity(svg_content.len() + defs_block.len());
            result.push_str(&svg_content[..insert_pos]);
            // Extract just the inner content from the defs block (strip outer <defs></defs>)
            let inner = defs_block
                .strip_prefix("<defs>")
                .and_then(|s| s.strip_suffix("</defs>"))
                .unwrap_or(defs_block);
            result.push_str(inner);
            result.push_str(&svg_content[insert_pos..]);
            return result;
        }
    }

    // No existing <defs>, insert after opening <svg ...>
    let mut reader = Reader::from_str(svg_content);
    loop {
        match reader.read_event() {
            Ok(Event::Start(ref e)) => {
                let local = e.local_name();
                if local.as_ref() == b"svg" {
                    let offset = reader.buffer_position() as usize;
                    let mut result =
                        String::with_capacity(svg_content.len() + defs_block.len() + 10);
                    result.push_str(&svg_content[..offset]);
                    result.push_str(defs_block);
                    result.push_str(&svg_content[offset..]);
                    return result;
                }
            }
            Ok(Event::Eof) | Err(_) => return svg_content.to_string(),
            _ => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scan_url_refs() {
        let svg = r#"<line marker-end="url(#s:arrow-closed)"/>"#;
        let refs = scan_refs(svg);
        assert!(refs.contains("s:arrow-closed"));
        assert_eq!(refs.len(), 1);
    }

    #[test]
    fn scan_href_refs() {
        let svg = "<use href=\"#s:marker-pin\" width=\"20\" height=\"30\"/>";
        let refs = scan_refs(svg);
        assert!(refs.contains("s:marker-pin"));
    }

    #[test]
    fn tree_shaken_defs_only_includes_referenced() {
        let mut refs = HashSet::new();
        refs.insert("s:arrow-closed".to_string());

        let defs = generate_defs_block(&refs);
        assert!(defs.is_some());
        let defs = defs.expect("should have defs");
        assert!(defs.contains("s:arrow-closed"));
        assert!(!defs.contains("s:arrow-open"));
        assert!(!defs.contains("s:marker-pin"));
    }

    #[test]
    fn empty_refs_produces_no_defs() {
        let refs = HashSet::new();
        assert!(generate_defs_block(&refs).is_none());
    }

    #[test]
    fn inject_defs_after_svg_tag() {
        let svg =
            r#"<svg viewBox="0 0 100 100"><rect x="0" y="0" width="100" height="100"/></svg>"#;
        let defs = "<defs><marker id=\"test\"/></defs>";
        let result = inject_defs(svg, defs);
        assert!(result.starts_with(r#"<svg viewBox="0 0 100 100"><defs>"#));
        assert!(result.contains("<marker id=\"test\"/>"));
    }

    #[test]
    fn inject_defs_into_existing_defs() {
        let svg = r#"<svg viewBox="0 0 100 100"><defs><pattern id="existing"/></defs></svg>"#;
        let defs = "<defs><marker id=\"new\"/></defs>";
        let result = inject_defs(svg, defs);
        // Should contain both the new marker and the existing pattern
        assert!(result.contains("<marker id=\"new\"/>"));
        assert!(result.contains("<pattern id=\"existing\"/>"));
        // Should only have one <defs>...</defs> section
        assert_eq!(result.matches("<defs>").count(), 1);
    }
}
