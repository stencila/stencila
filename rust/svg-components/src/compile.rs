use std::collections::HashMap;

use quick_xml::Reader;
use quick_xml::events::Event;

use crate::anchors::collect_anchors;
use crate::components::{Attrs, ComponentContext, expand_component};
use crate::defs::{generate_defs_block, inject_defs, scan_refs};
use crate::diagnostics;

/// The result of compiling an SVG overlay.
#[derive(Debug)]
pub struct CompilationResult {
    /// The compiled SVG with all `s:` elements expanded to standard SVG.
    /// `None` if the overlay contains only standard SVG (pass-through).
    pub compiled: Option<String>,

    /// Diagnostic messages generated during compilation.
    pub messages: Vec<diagnostics::CompilationMessage>,
}

/// Compile an SVG, expanding `s:` custom elements into standard SVG.
///
/// If the SVG contains no `s:` custom elements, `compiled` is `None`
/// (pass-through behavior — the source SVG is used directly).
///
/// The compilation pipeline:
/// 1. Detect whether any `s:` elements are present
/// 2. Collect explicit `<s:anchor>` definitions and auto-anchors from viewBox
/// 3. Expand all `s:` elements into standard SVG fragments
/// 4. Scan for `url(#s:...)` and `href="#s:..."` references to built-in defs
/// 5. Tree-shake and inject only referenced `<defs>` from the built-in library
/// 6. Serialize back to SVG string
pub fn compile(source: &str) -> CompilationResult {
    let mut messages = Vec::new();

    // Quick check: does the source contain any s: custom elements?
    // Standard SVG refs to s: defs (like url(#s:arrow-closed)) are allowed
    // but don't require component expansion.
    let has_custom_elements = has_s_elements(source);
    let has_s_refs = source.contains("url(#s:") || source.contains("href=\"#s:");

    if !has_custom_elements && !has_s_refs {
        // Pure standard SVG — pass through unchanged
        return CompilationResult {
            compiled: None,
            messages,
        };
    }

    // Collect anchors
    let anchors = collect_anchors(source, &mut messages);

    // If we have custom elements, expand them
    let expanded = if has_custom_elements {
        expand_s_elements(source, &anchors, &mut messages)
    } else {
        source.to_string()
    };

    // Scan for defs references in the expanded output
    let refs = scan_refs(&expanded);

    // Inject tree-shaken defs
    let output = if let Some(defs_block) = generate_defs_block(&refs) {
        inject_defs(&expanded, &defs_block)
    } else {
        expanded
    };

    CompilationResult {
        compiled: Some(output),
        messages,
    }
}

/// Get the element name as an owned String.
fn element_name(e: &quick_xml::events::BytesStart) -> String {
    let binding = e.name();
    String::from_utf8_lossy(binding.as_ref()).to_string()
}

/// Get the end element name as an owned String.
fn end_element_name(e: &quick_xml::events::BytesEnd) -> String {
    let binding = e.name();
    String::from_utf8_lossy(binding.as_ref()).to_string()
}

/// Check if the SVG contains any `s:` prefixed elements (not just refs).
fn has_s_elements(source: &str) -> bool {
    let mut reader = Reader::from_str(source);
    loop {
        match reader.read_event() {
            Ok(Event::Start(ref e)) | Ok(Event::Empty(ref e)) => {
                if element_name(e).starts_with("s:") {
                    return true;
                }
            }
            Ok(Event::Eof) => return false,
            Err(_) => return false,
            _ => {}
        }
    }
}

/// Expand all `s:` elements in the SVG, leaving standard SVG unchanged.
fn expand_s_elements(
    source: &str,
    anchors: &HashMap<String, crate::anchors::Anchor>,
    messages: &mut Vec<diagnostics::CompilationMessage>,
) -> String {
    let mut reader = Reader::from_str(source);
    let mut output = String::with_capacity(source.len());
    let mut skip_depth = 0u32;

    loop {
        match reader.read_event() {
            Ok(Event::Start(ref e)) => {
                let name = element_name(e);

                if let Some(component_name) = name.strip_prefix("s:") {
                    // strip "s:" prefix

                    if component_name == "anchor" {
                        // Anchors are collected earlier and stripped from output
                        skip_depth = 1;
                        continue;
                    }

                    let attrs = parse_element_attrs(e);
                    let mut ctx = ComponentContext { anchors, messages };

                    if let Some(svg) = expand_component(component_name, &attrs, &mut ctx) {
                        wrap_component(&mut output, component_name, &attrs, &svg);
                    } else {
                        messages.push(diagnostics::CompilationMessage::warning(format!(
                            "Unknown s: component: <s:{component_name}>"
                        )));
                    }

                    // Skip any content inside the s: element
                    skip_depth = 1;
                } else {
                    // Standard SVG element — pass through
                    if skip_depth > 0 {
                        skip_depth += 1;
                    } else {
                        write_element(&mut output, e, false);
                    }
                }
            }
            Ok(Event::Empty(ref e)) => {
                let name = element_name(e);

                if let Some(component_name) = name.strip_prefix("s:") {
                    if component_name == "anchor" {
                        // Anchors already collected, strip from output
                        continue;
                    }

                    let attrs = parse_element_attrs(e);
                    let mut ctx = ComponentContext { anchors, messages };

                    if let Some(svg) = expand_component(component_name, &attrs, &mut ctx) {
                        wrap_component(&mut output, component_name, &attrs, &svg);
                    } else {
                        messages.push(diagnostics::CompilationMessage::warning(format!(
                            "Unknown s: component: <s:{component_name}>"
                        )));
                    }
                } else if skip_depth == 0 {
                    write_element(&mut output, e, true);
                }
            }
            Ok(Event::End(ref e)) => {
                let name = end_element_name(e);

                if name.starts_with("s:") {
                    skip_depth = skip_depth.saturating_sub(1);
                } else if skip_depth > 0 {
                    skip_depth -= 1;
                    if skip_depth == 0 {
                        write_end_event(&mut output, e);
                    }
                } else {
                    write_end_event(&mut output, e);
                }
            }
            Ok(Event::Text(ref e)) => {
                if skip_depth == 0 {
                    output.push_str(&String::from_utf8_lossy(e.as_ref()));
                }
            }
            Ok(Event::CData(ref e)) => {
                if skip_depth == 0 {
                    output.push_str("<![CDATA[");
                    output.push_str(&String::from_utf8_lossy(e.as_ref()));
                    output.push_str("]]>");
                }
            }
            Ok(Event::Comment(ref e)) => {
                if skip_depth == 0 {
                    output.push_str("<!--");
                    output.push_str(&String::from_utf8_lossy(e.as_ref()));
                    output.push_str("-->");
                }
            }
            Ok(Event::Decl(ref e)) => {
                output.push_str("<?");
                output.push_str(&String::from_utf8_lossy(e.as_ref()));
                output.push_str("?>");
            }
            Ok(Event::PI(ref e)) => {
                output.push_str("<?");
                output.push_str(&String::from_utf8_lossy(e.as_ref()));
                output.push_str("?>");
            }
            Ok(Event::DocType(ref e)) => {
                output.push_str("<!DOCTYPE ");
                output.push_str(&String::from_utf8_lossy(e.as_ref()));
                output.push('>');
            }
            Ok(Event::Eof) => break,
            Err(e) => {
                messages.push(diagnostics::CompilationMessage::error(format!(
                    "SVG parse error: {e}"
                )));
                break;
            }
        }
    }

    output
}

pub(crate) fn parse_element_attrs(e: &quick_xml::events::BytesStart) -> Attrs {
    e.attributes()
        .flatten()
        .map(|attr| {
            let key = String::from_utf8_lossy(attr.key.as_ref()).to_string();
            let val = attr
                .unescape_value()
                .map(|v| v.to_string())
                .unwrap_or_else(|_| String::from_utf8_lossy(&attr.value).to_string());
            (key, val)
        })
        .collect()
}

fn write_element(output: &mut String, e: &quick_xml::events::BytesStart, self_closing: bool) {
    output.push('<');
    let binding = e.name();
    output.push_str(&String::from_utf8_lossy(binding.as_ref()));
    for attr in e.attributes().flatten() {
        output.push(' ');
        output.push_str(&String::from_utf8_lossy(attr.key.as_ref()));
        output.push_str("=\"");
        output.push_str(&String::from_utf8_lossy(&attr.value));
        output.push('"');
    }
    if self_closing {
        output.push_str("/>");
    } else {
        output.push('>');
    }
}

/// Wrap expanded component SVG in a `<g>` with `s:` namespace attributes
/// preserving the original component name and attributes for reverse compilation.
fn wrap_component(output: &mut String, component_name: &str, attrs: &Attrs, svg: &str) {
    output.push_str("<g s:component=\"");
    output.push_str(component_name);
    output.push('"');
    // Write original attributes in sorted order for deterministic output
    let mut keys: Vec<&String> = attrs.keys().collect();
    keys.sort();
    for key in keys {
        let value = &attrs[key];
        output.push_str(" s:");
        output.push_str(key);
        output.push_str("=\"");
        output.push_str(&xml_escape(value));
        output.push('"');
    }
    output.push('>');
    output.push_str(svg);
    output.push_str("</g>");
}

/// Escape XML special characters in attribute values.
pub(crate) fn xml_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

fn write_end_event(output: &mut String, e: &quick_xml::events::BytesEnd) {
    output.push_str("</");
    let binding = e.name();
    output.push_str(&String::from_utf8_lossy(binding.as_ref()));
    output.push('>');
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pass_through_standard_svg() {
        let svg =
            r#"<svg viewBox="0 0 600 400"><rect x="10" y="10" width="100" height="100"/></svg>"#;
        let result = compile(svg);
        assert!(result.compiled.is_none());
        assert!(result.messages.is_empty());
    }

    #[test]
    fn standard_svg_with_s_defs_refs() {
        let svg = r##"<svg viewBox="0 0 600 400"><line x1="0" y1="0" x2="100" y2="100" marker-end="url(#s:arrow-closed)"/></svg>"##;
        let result = compile(svg);

        assert!(result.compiled.is_some());
        let compiled = result
            .compiled
            .as_deref()
            .expect("should have compiled output");
        assert!(compiled.contains("s:arrow-closed"));
        assert!(compiled.contains("<defs>"));
        assert!(result.messages.is_empty());
    }

    #[test]
    fn standard_svg_nested_inside_component_is_skipped() {
        let svg = r#"<svg viewBox="0 0 100 100" xmlns:s="https://stencila.io/svg"><s:badge x="50" y="50" label="Test"><rect x="0" y="0" width="10" height="10"/></s:badge></svg>"#;
        let result = compile(svg);
        let compiled = result.compiled.expect("should compile");

        assert!(!compiled.contains("<rect x=\"0\" y=\"0\" width=\"10\" height=\"10\"/>"));
        assert!(roxmltree::Document::parse(&compiled).is_ok());
    }

    #[test]
    fn expand_callout_with_anchors() {
        let svg = r##"<svg viewBox="0 0 600 400" xmlns:s="https://stencila.io/svg"><s:anchor id="peak" x="335" y="100"/><s:callout from="#peak" dx="125" dy="-45" label="Peak" to="#peak"/></svg>"##;
        let result = compile(svg);

        assert!(result.compiled.is_some());
        let compiled = result
            .compiled
            .as_deref()
            .expect("should have compiled output");
        assert!(compiled.contains("<text"));
        assert!(compiled.contains("Peak"));
        // Should contain a leader line since "to" is specified
        assert!(compiled.contains(r#"stroke="currentColor""#));
        assert!(result.messages.is_empty());
    }

    #[test]
    fn expand_scale_bar() {
        let svg = r##"<svg viewBox="0 0 600 400" xmlns:s="https://stencila.io/svg"><s:scale-bar x="40" y="326" length="130" label="20 μm"/></svg>"##;
        let result = compile(svg);

        assert!(result.compiled.is_some());
        let compiled = result
            .compiled
            .as_deref()
            .expect("should have compiled output");
        assert!(compiled.contains("20 μm"));
        // Should have main line and end caps
        assert!(compiled.contains("<line"));
        assert!(result.messages.is_empty());
    }

    #[test]
    fn malformed_element_produces_error() {
        let svg =
            r#"<svg viewBox="0 0 600 400" xmlns:s="https://stencila.io/svg"><s:arrow/></svg>"#;
        let result = compile(svg);

        assert!(!result.messages.is_empty());
        assert!(result.messages[0].message.contains("requires"));
    }

    #[test]
    fn unknown_component_produces_warning() {
        let svg = r#"<svg viewBox="0 0 600 400" xmlns:s="https://stencila.io/svg"><s:unknown-thing x="10" y="10"/></svg>"#;
        let result = compile(svg);

        assert!(!result.messages.is_empty());
        assert_eq!(result.messages[0].level, diagnostics::MessageLevel::Warning);
        assert!(result.messages[0].message.contains("Unknown"));
    }

    #[test]
    fn auto_anchor_center_resolves() {
        let svg = r##"<svg viewBox="0 0 600 400" xmlns:s="https://stencila.io/svg"><s:callout from="#s:center" label="Center"/></svg>"##;
        let result = compile(svg);

        assert!(result.compiled.is_some());
        let compiled = result
            .compiled
            .as_deref()
            .expect("should have compiled output");
        assert!(compiled.contains("300")); // center x of 600-wide viewBox
        assert!(compiled.contains("200")); // center y of 400-tall viewBox
        assert!(result.messages.is_empty());
    }

    #[test]
    fn arrow_elbow_horizontal_first() {
        let svg = r##"<svg viewBox="0 0 600 400" xmlns:s="https://stencila.io/svg"><s:arrow x="100" y="100" to-x="300" to-y="200" curve="elbow" corner="horizontal-first"/></svg>"##;
        let result = compile(svg);

        assert!(result.compiled.is_some());
        let compiled = result
            .compiled
            .as_deref()
            .expect("should have compiled output");
        assert!(compiled.contains("<polyline"));
        assert!(compiled.contains("100,100 300,100 300,200"));
        assert!(result.messages.is_empty());
    }

    #[test]
    fn callout_pill_no_leader() {
        let svg = r#"<svg viewBox="0 0 600 400" xmlns:s="https://stencila.io/svg"><s:callout x="460" y="55" label="n = 1,024" shape="pill"/></svg>"#;
        let result = compile(svg);

        assert!(result.compiled.is_some());
        let compiled = result
            .compiled
            .as_deref()
            .expect("should have compiled output");
        assert!(compiled.contains("<rect"));
        assert!(compiled.contains("<text"));
        assert!(compiled.contains("n = 1,024"));
        // Should NOT have a line element (no to/to-x/to-y)
        assert!(!compiled.contains("<line"));
        assert!(result.messages.is_empty());
    }

    #[test]
    fn compass_default_arrow_variant() {
        let svg = r#"<svg viewBox="0 0 600 400" xmlns:s="https://stencila.io/svg"><s:compass x="40" y="360" size="50"/></svg>"#;
        let result = compile(svg);

        assert!(result.compiled.is_some());
        let compiled = result
            .compiled
            .as_deref()
            .expect("should have compiled output");
        // Should have an arrow line and "N" label
        assert!(compiled.contains("<line"));
        assert!(compiled.contains(">N<"));
        assert!(result.messages.is_empty());
    }

    #[test]
    fn compass_full_variant_with_custom_axes() {
        let svg = r#"<svg viewBox="0 0 600 400" xmlns:s="https://stencila.io/svg"><s:compass x="40" y="360" size="50" variant="full" axes="A/P D/V"/></svg>"#;
        let result = compile(svg);

        assert!(result.compiled.is_some());
        let compiled = result
            .compiled
            .as_deref()
            .expect("should have compiled output");
        assert!(compiled.contains(">A<"));
        assert!(compiled.contains(">P<"));
        assert!(compiled.contains(">D<"));
        assert!(compiled.contains(">V<"));
        assert!(result.messages.is_empty());
    }

    #[test]
    fn marker_with_symbol_and_label() {
        let svg = r#"<svg viewBox="0 0 600 400" xmlns:s="https://stencila.io/svg"><s:marker x="100" y="200" symbol="pin" label="Site A"/></svg>"#;
        let result = compile(svg);

        assert!(result.compiled.is_some());
        let compiled = result
            .compiled
            .as_deref()
            .expect("should have compiled output");
        assert!(compiled.contains("<use"));
        assert!(compiled.contains("s:marker-pin"));
        assert!(compiled.contains("Site A"));
        assert!(result.messages.is_empty());
    }

    #[test]
    fn mixed_raw_svg_and_components() {
        let svg = r#"<svg viewBox="0 0 600 400" xmlns:s="https://stencila.io/svg"><rect x="10" y="10" width="100" height="100" fill="red"/><s:callout x="200" y="100" label="Hello"/><circle cx="300" cy="300" r="50"/></svg>"#;
        let result = compile(svg);

        assert!(result.compiled.is_some());
        let compiled = result
            .compiled
            .as_deref()
            .expect("should have compiled output");
        // Raw SVG preserved
        assert!(compiled.contains("<rect"));
        assert!(compiled.contains("fill=\"red\""));
        assert!(compiled.contains("<circle"));
        // Component expanded
        assert!(compiled.contains("Hello"));
        assert!(result.messages.is_empty());
    }
}
