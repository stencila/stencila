use std::collections::BTreeMap;

use serde_json::Value;

/// Generate a DOCX `styles.xml` file from a Stencila [`Theme`]
///
/// Generates DOCX styles that mirror Stencila's default template structure
/// to ensure compatibility with documents created by Stencila+Pandoc.
///
/// # DOCX Style to Style Builder Mapping
///
/// The style names and hierarchy were derived from Stencila's default DOCX template (which is based on
/// Pandoc's default DOCX template with some additional styles). To see these:
///
///   unzip rust/codec-docx/templates/default.docx -d default
///   cat default/word/styles.xml
///
/// **Implemented DOCX Styles** (✓):
/// - ✓ `Normal` (paragraph) → [`build_normal_style()`] - Base paragraph style
/// - ✓ `Heading1-9` (paragraph) → [`build_heading_style()`] - Hierarchical heading styles
/// - ✓ `BodyText` (paragraph) → [`build_body_text_style()`] - Standard body paragraphs
/// - ✓ `BlockText` (paragraph) → [`build_block_text_style()`] - Block quotes
/// - ✓ `DefaultParagraphFont` (character) → [`build_default_paragraph_font()`] - Base character style
/// - ✓ `BodyTextChar` (character) → [`build_body_text_char_style()`] - Body text character variant
/// - ✓ `Heading1Char-9Char` (character) → [`build_heading_char_styles()`] - Heading character variants
/// - ✓ `VerbatimChar` (character) → [`build_verbatim_char_style()`] - Inline code
/// - ✓ `Hyperlink` (character) → [`build_hyperlink_style()`] - Link formatting
/// - ✓ `Table` (table) → [`build_table_style()`] - Default table style
///
/// **Not Yet Implemented** (✗):
///
/// *Paragraph Styles*:
/// - ✗ `Title` - Document title
/// - ✗ `Subtitle` - Document subtitle
/// - ✗ `Author` - Author name
/// - ✗ `Date` - Document date
/// - ✗ `Abstract` / `AbstractTitle` - Abstract section
/// - ✗ `Caption` - Generic caption style
/// - ✗ `TableCaption` / `ImageCaption` - Specific caption types
/// - ✗ `Figure` / `CaptionedFigure` - Figure containers
/// - ✗ `List` - List paragraphs
/// - ✗ `FirstParagraph` - First paragraph after heading
/// - ✗ `Compact` - Compact paragraph spacing
/// - ✗ `Bibliography` - Bibliography entries
/// - ✗ `FootnoteText` / `FootnoteBlockText` - Footnote content
/// - ✗ `DefinitionTerm` / `Definition` - Definition lists
/// - ✗ `Index` / `IndexHeading` - Index content
/// - ✗ `TOCHeading` - Table of contents heading
///
/// *Character Styles*:
/// - ✗ `TitleChar` / `SubtitleChar` - Title/subtitle character variants
/// - ✗ `SectionNumber` - Section numbering
/// - ✗ `FootnoteCharacters` / `FootnoteReference` - Footnote markers
/// - ✗ `EndnoteCharacters` / `EndnoteReference` - Endnote markers
/// - ✗ `Quotation` - Inline quotations
/// - ✗ `Reproducible` / `ReproducibleHighlighted` - Reproducible code markers
/// - ✗ `Output` / `OutputHighlighted` - Code output formatting
/// - ✗ `Rubies` - Ruby text (East Asian annotations)
///
/// ## Style Inheritance (w:basedOn)
///
/// The w:basedOn attribute establishes a hierarchy where child styles inherit
/// properties from parent styles unless explicitly overridden:
///
/// - Normal (base)
///   - Heading1-9 (based on Normal)
///   - BodyText (based on Normal)
///     - BlockText (based on BodyText)
///
/// - DefaultParagraphFont (base character style)
///   - BodyTextChar (based on DefaultParagraphFont)
///     - VerbatimChar (based on BodyTextChar)
///     - Hyperlink (based on BodyTextChar)
///   - Heading1Char-9Char (based on DefaultParagraphFont)
///
/// This inheritance model matches Pandoc's approach and provides:
/// - Consistent styling when documents are edited in Word/LibreOffice
/// - Predictable behavior when styles are customized
/// - Compatibility with existing DOCX workflows and templates
///
/// ## Style Types
///
/// - **Paragraph styles** (w:type="paragraph"): Applied to entire paragraphs
/// - **Character styles** (w:type="character"): Applied to text runs within paragraphs
/// - **Linked styles**: Many paragraph styles have linked character variants (e.g., Heading1 ↔ Heading1Char)
///   allowing the same formatting to be applied at either paragraph or character level
/// - **Table styles** (w:type="table"): Define table appearance and cell formatting
///
/// # CSS File to Style Builder Mapping
///
/// This mapping shows which theme CSS files from `web/src/themes/base/` are used by
/// which style builder functions. Files marked with ✓ have implementations below.
/// Files marked with ✗ are not yet mapped to DOCX styles.
///
/// **Implemented Mappings**:
/// - ✓ `tokens-semantic.css` → [`build_doc_defaults()`], [`build_normal_style()`]
/// - ✓ `headings.css` → [`build_heading_style()`], [`build_heading_char_styles()`]
/// - ✓ `paragraphs.css` → [`build_normal_style()`], [`build_body_text_style()`]
/// - ✓ `code.css` → [`build_verbatim_char_style()`]
/// - ✓ `links.css` → [`build_hyperlink_style()`]
/// - ✓ `quotes.css` → [`build_block_text_style()`]
/// - ✓ `tables.css` → [`build_table_style()`]
///
/// **Not Yet Mapped** (could be implemented in future):
/// - ✗ `admonitions.css` - Callout/alert boxes → Custom paragraph styles with borders/shading (w:pBdr, w:shd)
/// - ✗ `breaks.css` - Page/section breaks → Page break runs (w:br w:type="page")
/// - ✗ `captions.css` - Figure/table captions → Extended caption paragraph styles (partially in table style)
/// - ✗ `citations.css` - Citation formatting → Custom character/paragraph styles for bibliography
/// - ✗ `figures.css` - Figure captions/containers → Custom paragraph styles for figure captions
/// - ✗ `labels.css` - Figure/table label formatting → Character styles for label prefixes (e.g., "Figure 1:")
/// - ✗ `lists.css` - Ordered/unordered lists → DOCX numbering definitions (w:numPr, w:numbering.xml)
/// - ✗ `pages.css` - Page layout/printing → Section properties (w:sectPr for margins, columns, etc.)
/// - ✗ `references.css` - Cross-reference styling → Custom character styles for internal references
///
/// **No Mapping Applicable** (not relevant for static DOCX):
/// - ✗ `articles.css` - Document container layout (layout-only, not applicable to DOCX body styles)
/// - ✗ `datatables.css` - Interactive data table UI (dynamic features not in static DOCX)
/// - ✗ `diagrams.css` - Diagram rendering (diagrams embedded as images, not styled)
/// - ✗ `images.css` - Image display properties (images are objects in DOCX, not styled elements)
/// - ✗ `plots.css` - Plot/chart rendering (plots embedded as images, not styled)
/// - ✗ `math.css` - Mathematical notation rendering (DOCX uses OMML equation format, not CSS)
///
/// # Arguments
///
/// * `variables` - Pre-computed theme variables with fonts already resolved
pub(crate) fn theme_to_styles(variables: &BTreeMap<String, Value>) -> String {
    // Pre-allocate capacity (estimate ~20KB for full styles.xml)
    let mut xml = String::with_capacity(20_000);

    // XML declaration and root element
    xml.push_str(
        r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<w:styles xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"
    xmlns:w14="http://schemas.microsoft.com/office/word/2010/wordml"
    xmlns:mc="http://schemas.openxmlformats.org/markup-compatibility/2006"
    mc:Ignorable="w14">"#,
    );

    // Build document structure
    xml.push_str(&build_doc_defaults(variables));
    xml.push_str(&build_latent_styles());
    xml.push_str(&build_normal_style(variables));

    // Headings 1-9
    for level in 1..=9 {
        xml.push_str(&build_heading_style(variables, level));
    }

    // Character styles
    xml.push_str(&build_heading_char_styles(variables));
    xml.push_str(&build_default_paragraph_font());
    xml.push_str(&build_body_text_char_style());
    xml.push_str(&build_verbatim_char_style(variables));
    xml.push_str(&build_hyperlink_style(variables));

    // Paragraph styles
    xml.push_str(&build_body_text_style());
    xml.push_str(&build_block_text_style(variables));

    // Table styles
    xml.push_str(&build_table_style(variables));

    xml.push_str("</w:styles>");
    xml
}

// ============================================================================
// Constants for common XML fragments
// ============================================================================

const KEEP_NEXT: &str = r#"<w:keepNext w:val="true"/>"#;
const KEEP_LINES: &str = r#"<w:keepLines/>"#;
const WIDOW_CONTROL: &str = r#"<w:widowControl/>"#;

// ============================================================================
// Helper functions for value extraction
// ============================================================================

/// Get a string value from computed theme variables
fn get_var(vars: &BTreeMap<String, Value>, name: &str) -> Option<String> {
    vars.get(name).and_then(|v| match v {
        Value::String(s) => Some(s.clone()),
        Value::Number(n) => Some(n.to_string()),
        _ => None,
    })
}

/// Get a hex color value (strips # prefix for DOCX compatibility)
fn get_color_hex(vars: &BTreeMap<String, Value>, name: &str) -> Option<String> {
    get_var(vars, name).map(|color| color.trim_start_matches('#').to_string())
}

/// Convert twips to half-points (DOCX font size unit: 1pt = 2 half-points, 1pt = 20 twips)
fn twips_to_half_points(twips: f64) -> String {
    (twips / 10.0).round().to_string()
}

/// Get font size in half-points from a twips variable
fn get_font_size_half_points(vars: &BTreeMap<String, Value>, name: &str) -> Option<String> {
    vars.get(name)
        .and_then(|v| v.as_f64().map(twips_to_half_points))
}

/// Get font-variant XML element based on CSS font-variant value
fn get_font_variant_element(vars: &BTreeMap<String, Value>, name: &str) -> String {
    get_var(vars, name)
        .and_then(|variant| {
            let variant = variant.trim();
            match variant {
                "small-caps" => Some(r#"<w:smallCaps/>"#.to_string()),
                "all-small-caps" | "all-caps" => Some(r#"<w:caps/>"#.to_string()),
                _ => None,
            }
        })
        .unwrap_or_default()
}

/// Get spacing value in twips as a string
fn get_twips(vars: &BTreeMap<String, Value>, name: &str) -> Option<String> {
    vars.get(name)
        .and_then(|v| v.as_f64().map(|twips| twips.round().to_string()))
}

/// Get text alignment value for DOCX (w:jc)
///
/// Converts CSS text-align values to DOCX justification values.
/// Returns "left" as default if the token is not found or has an unsupported value.
fn get_text_align(vars: &BTreeMap<String, Value>, name: &str) -> String {
    get_var(vars, name)
        .and_then(|align| {
            let align = align.trim();
            match align {
                "left" | "start" => Some("left".to_string()),
                "right" | "end" => Some("right".to_string()),
                "center" => Some("center".to_string()),
                "justify" => Some("both".to_string()),
                _ => None,
            }
        })
        .unwrap_or_else(|| "left".to_string())
}

/// Check if font weight indicates bold (>= 600)
fn is_bold(vars: &BTreeMap<String, Value>, name: &str) -> bool {
    vars.get(name)
        .and_then(|v| v.as_f64())
        .map(|weight| weight >= 600.0)
        .unwrap_or(false)
}

/// Check if font style is italic
fn is_italic(vars: &BTreeMap<String, Value>, name: &str) -> bool {
    get_var(vars, name)
        .map(|style| style.trim() == "italic")
        .unwrap_or(false)
}

// ============================================================================
// XML building helpers
// ============================================================================

/// Build <w:rFonts> element from font-family variable
///
/// The font family value should already be a resolved font name (not a CSS stack)
/// since fonts are resolved earlier in the `theme_to_styles` function.
fn build_font_element(vars: &BTreeMap<String, Value>, var_name: &str) -> String {
    get_var(vars, var_name)
        .map(|family| {
            format!(r#"<w:rFonts w:ascii="{family}" w:hAnsi="{family}" w:eastAsia="{family}" w:cs="" />"#)
        })
        .unwrap_or_default()
}

/// Build <w:sz> and <w:szCs> elements for font size
fn build_size_elements(half_points: &str) -> String {
    format!(r#"<w:sz w:val="{half_points}"/><w:szCs w:val="{half_points}"/>"#)
}

/// Build <w:color> element from hex color (without # prefix)
fn build_color_element(hex: &str) -> String {
    format!(r#"<w:color w:val="{hex}"/>"#)
}

/// Build <w:spacing> element with before/after attributes
fn build_spacing_element(before_twips: Option<&str>, after_twips: Option<&str>) -> String {
    let mut attrs = String::new();

    if let Some(before) = before_twips {
        attrs.push_str(&format!(r#" w:before="{before}""#));
    }
    if let Some(after) = after_twips {
        attrs.push_str(&format!(r#" w:after="{after}""#));
    }

    if attrs.is_empty() {
        String::new()
    } else {
        format!("<w:spacing{attrs}/>")
    }
}

// ============================================================================
// Style builders
// ============================================================================

/// Build document defaults (<w:docDefaults>)
///
/// **CSS Tokens Source**: `web/src/themes/base/tokens-semantic.css`
///
/// **Tokens Applied**:
/// - `text-font-family` → w:rFonts (default font for all runs)
/// - `text-font-size` → w:sz/w:szCs (default font size)
/// - `text-letter-spacing` → w:spacing w:val (character spacing in twips)
///
/// **Tokens NOT Yet Applied**:
/// - `text-color-primary` - Applied in Normal style instead
/// - `text-line-height` - DOCX uses automatic line height at document level
fn build_doc_defaults(vars: &BTreeMap<String, Value>) -> String {
    let mut xml = String::with_capacity(512);

    xml.push_str("<w:docDefaults><w:rPrDefault><w:rPr>");
    xml.push_str(&build_font_element(vars, "text-font-family"));

    if let Some(size) = get_font_size_half_points(vars, "text-font-size") {
        xml.push_str(&build_size_elements(&size));
    }

    // Letter spacing (character spacing in twips)
    if let Some(spacing) = get_twips(vars, "text-letter-spacing") {
        xml.push_str(&format!(r#"<w:spacing w:val="{spacing}"/>"#));
    }

    xml.push_str(
        r#"<w:lang w:val="en-US" w:eastAsia="en-US" w:bidi="ar-SA"/>
</w:rPr></w:rPrDefault>
<w:pPrDefault><w:pPr><w:suppressAutoHyphens w:val="true"/></w:pPr></w:pPrDefault>
</w:docDefaults>"#,
    );

    xml
}

/// Build latent styles element (boilerplate)
fn build_latent_styles() -> String {
    r#"<w:latentStyles w:defLockedState="0" w:defUIPriority="0" w:defSemiHidden="0" w:defUnhideWhenUsed="0" w:defQFormat="0" w:count="276"></w:latentStyles>"#.to_string()
}

/// Build Normal paragraph style
///
/// **CSS Tokens Source**: `web/src/themes/base/tokens-semantic.css`, `web/src/themes/base/paragraphs.css`
///
/// **Tokens Applied**:
/// - `text-font-family` → w:rFonts
/// - `text-color-primary` → w:color
/// - `text-font-size` → w:sz/w:szCs
/// - `text-letter-spacing` → w:spacing w:val (character spacing in twips)
/// - `paragraph-spacing` → w:spacing w:after (paragraph spacing after in twips, defaults to 200)
/// - `paragraph-text-align` → w:jc (text justification, defaults to "left")
/// - `paragraph-text-indent` → w:ind w:firstLine (first line indentation in twips)
/// - Fixed spacing before (0 twips) → w:spacing w:before
///
/// **Tokens NOT Yet Applied**:
/// - `text-line-height` / `paragraph-line-height` - DOCX uses automatic line height
/// - `paragraph-orphans` / `paragraph-widows` - Would map to w:widowControl w:val (complex conversion)
fn build_normal_style(vars: &BTreeMap<String, Value>) -> String {
    let mut xml = String::with_capacity(512);

    xml.push_str(
        r#"<w:style w:type="paragraph" w:styleId="Normal" w:default="1">
<w:name w:val="Normal"/><w:qFormat/>
<w:pPr>"#,
    );
    xml.push_str(WIDOW_CONTROL);
    xml.push_str(r#"<w:suppressAutoHyphens w:val="true"/><w:bidi w:val="0"/>"#);

    // Spacing (before and after paragraph)
    let after_spacing = get_twips(vars, "paragraph-spacing").unwrap_or_else(|| "200".to_string());
    xml.push_str(&build_spacing_element(Some("0"), Some(&after_spacing)));

    // Text indent (first line indentation)
    if let Some(indent) = get_twips(vars, "paragraph-text-indent")
        && indent != "0"
    {
        xml.push_str(&format!(r#"<w:ind w:firstLine="{indent}"/>"#));
    }

    // Text alignment
    let alignment = get_text_align(vars, "paragraph-text-align");
    xml.push_str(&format!(r#"<w:jc w:val="{alignment}"/></w:pPr><w:rPr>"#));

    xml.push_str(&build_font_element(vars, "text-font-family"));

    if let Some(color) = get_color_hex(vars, "text-color-primary") {
        xml.push_str(&build_color_element(&color));
    }

    xml.push_str(r#"<w:kern w:val="0"/>"#);

    if let Some(size) = get_font_size_half_points(vars, "text-font-size") {
        xml.push_str(&build_size_elements(&size));
    }

    // Letter spacing (character spacing in twips)
    if let Some(spacing) = get_twips(vars, "text-letter-spacing") {
        xml.push_str(&format!(r#"<w:spacing w:val="{spacing}"/>"#));
    }

    xml.push_str(
        r#"<w:lang w:val="en-US" w:eastAsia="en-US" w:bidi="ar-SA"/>
</w:rPr></w:style>"#,
    );

    xml
}

/// Build a single heading style (Heading1-Heading9)
///
/// Note: The generic `Heading` base style is intentionally skipped.
/// All Heading1-9 styles inherit directly from `Normal`, which provides all necessary
/// functionality without adding an extra layer of indirection.
///
/// **CSS Tokens Source**: `web/src/themes/base/headings.css`
///
/// **Tokens Applied**:
/// - `heading-font-family` → w:rFonts
/// - `heading-color` → w:color
/// - `heading-font-size` with `heading-font-size-ratio` → w:sz (exponential scaling)
/// - `heading-h{N}-font-weight` → w:b/w:bCs (if >= 600)
/// - `heading-h{N}-font-style` → w:i/w:iCs (if "italic")
/// - `heading-h{N}-font-variant` → w:smallCaps (if "small-caps") or w:caps (if "all-caps")
/// - `heading-h{N}-letter-spacing` → w:spacing w:val (character spacing in twips)
/// - `heading-spacing-top-{N}` → w:spacing w:before
/// - `heading-spacing-bottom` → w:spacing w:after
///
/// **Tokens NOT Yet Applied**:
/// - `heading-line-height` - DOCX uses automatic line height; complex conversion for unitless values
/// - `heading-color-opacity-decrement` - Would require blending color with background
/// - `heading-font-weight-decrement` - Already effectively applied via per-level font-weight tokens
/// - Page break properties - Applied via KEEP_NEXT/KEEP_LINES constants instead
fn build_heading_style(vars: &BTreeMap<String, Value>, level: u8) -> String {
    let mut xml = String::with_capacity(768);

    let style_id = format!("Heading{level}");
    let style_name = format!("heading {level}");
    let char_link = format!("Heading{level}Char");

    xml.push_str(&format!(
        r#"<w:style w:type="paragraph" w:styleId="{style_id}">
<w:name w:val="{style_name}"/>
<w:basedOn w:val="Normal"/>
<w:next w:val="BodyText"/>
<w:link w:val="{char_link}"/>
<w:uiPriority w:val="9"/>"#
    ));

    if level > 1 {
        xml.push_str(r#"<w:semiHidden/><w:unhideWhenUsed/>"#);
    }

    xml.push_str(r#"<w:qFormat/><w:rsid w:val="00a10fd9"/><w:pPr>"#);
    xml.push_str(KEEP_NEXT);
    xml.push_str(KEEP_LINES);

    // Spacing
    let spacing_var = format!("heading-spacing-top-{level}");
    let before = get_twips(vars, &spacing_var);
    let after = get_twips(vars, "heading-spacing-bottom");
    xml.push_str(&build_spacing_element(before.as_deref(), after.as_deref()));

    // Outline level (0-indexed)
    xml.push_str(&format!(
        r#"<w:outlineLvl w:val="{}"/></w:pPr><w:rPr>"#,
        level - 1
    ));

    xml.push_str(&build_font_element(vars, "heading-font-family"));

    // Color
    if let Some(color) = get_color_hex(vars, "heading-color") {
        xml.push_str(&build_color_element(&color));
    }

    // Calculate font size: base × ratio^(level-1)
    if let (Some(base_size), Some(ratio)) = (
        vars.get("heading-font-size").and_then(|v| v.as_f64()),
        vars.get("heading-font-size-ratio").and_then(|v| v.as_f64()),
    ) {
        let size_twips = base_size * ratio.powi((level - 1) as i32);
        let half_points = twips_to_half_points(size_twips);
        xml.push_str(&build_size_elements(&half_points));
    }

    // Bold/italic from per-level variables
    let weight_var = format!("heading-h{level}-font-weight");
    if is_bold(vars, &weight_var) {
        xml.push_str(r#"<w:b/><w:bCs/>"#);
    }

    let style_var = format!("heading-h{level}-font-style");
    if is_italic(vars, &style_var) {
        xml.push_str(r#"<w:i/><w:iCs/>"#);
    }

    // Font variant (small-caps, all-caps)
    let variant_var = format!("heading-h{level}-font-variant");
    xml.push_str(&get_font_variant_element(vars, &variant_var));

    // Letter spacing (character spacing in twips)
    let spacing_var = format!("heading-h{level}-letter-spacing");
    if let Some(spacing) = get_twips(vars, &spacing_var) {
        xml.push_str(&format!(r#"<w:spacing w:val="{spacing}"/>"#));
    }

    xml.push_str("</w:rPr></w:style>");
    xml
}

/// Build heading character styles (Heading1Char-Heading9Char)
///
/// **CSS Tokens Source**: `web/src/themes/base/headings.css`
///
/// Same tokens as `build_heading_style()` but applied as character-level formatting.
/// These linked character styles allow heading formatting to be applied to text runs
/// within paragraphs without changing the paragraph style itself.
fn build_heading_char_styles(vars: &BTreeMap<String, Value>) -> String {
    let mut xml = String::with_capacity(6_000);

    for level in 1..=9 {
        let style_id = format!("Heading{level}Char");
        let style_name = format!("Heading {level} Char");
        let para_link = format!("Heading{level}");

        xml.push_str(&format!(
            r#"<w:style w:type="character" w:styleId="{style_id}" w:customStyle="1">
<w:name w:val="{style_name}"/>
<w:basedOn w:val="DefaultParagraphFont"/>
<w:link w:val="{para_link}"/>
<w:uiPriority w:val="9"/>"#
        ));

        if level > 1 {
            xml.push_str(r#"<w:semiHidden/>"#);
        }

        xml.push_str(r#"<w:qFormat/><w:rsid w:val="00a10fd9"/><w:rPr>"#);

        xml.push_str(&build_font_element(vars, "heading-font-family"));

        if let Some(color) = get_color_hex(vars, "heading-color") {
            xml.push_str(&build_color_element(&color));
        }

        if let (Some(base_size), Some(ratio)) = (
            vars.get("heading-font-size").and_then(|v| v.as_f64()),
            vars.get("heading-font-size-ratio").and_then(|v| v.as_f64()),
        ) {
            let size_twips = base_size * ratio.powi(level - 1);
            let half_points = twips_to_half_points(size_twips);
            xml.push_str(&build_size_elements(&half_points));
        }

        let weight_var = format!("heading-h{level}-font-weight");
        if is_bold(vars, &weight_var) {
            xml.push_str(r#"<w:b/><w:bCs/>"#);
        }

        let style_var = format!("heading-h{level}-font-style");
        if is_italic(vars, &style_var) {
            xml.push_str(r#"<w:i/><w:iCs/>"#);
        }

        // Font variant (small-caps, all-caps)
        let variant_var = format!("heading-h{level}-font-variant");
        xml.push_str(&get_font_variant_element(vars, &variant_var));

        // Letter spacing (character spacing in twips)
        let spacing_var = format!("heading-h{level}-letter-spacing");
        if let Some(spacing) = get_twips(vars, &spacing_var) {
            xml.push_str(&format!(r#"<w:spacing w:val="{spacing}"/>"#));
        }

        xml.push_str("</w:rPr></w:style>");
    }

    xml
}

/// Build VerbatimChar character style
///
/// **CSS Tokens Source**: `web/src/themes/base/code.css`
///
/// **Tokens Applied**:
/// - `code-font-family` → w:rFonts
/// - `code-color` → w:color
/// - `code-font-size-inline` → w:sz/w:szCs
///
/// **Tokens NOT Yet Applied**:
/// - `code-background-inline` - DOCX doesn't support background on character styles (only paragraph shading)
/// - `code-border-radius` / `code-border-width` - Not applicable to DOCX
/// - `code-padding-inline` - Not directly mappable to DOCX character style
/// - `code-line-height` - DOCX uses automatic line height
fn build_verbatim_char_style(vars: &BTreeMap<String, Value>) -> String {
    let mut xml = String::with_capacity(512);

    xml.push_str(
        r#"<w:style w:type="character" w:styleId="VerbatimChar" w:customStyle="1">
<w:name w:val="Verbatim Char"/>
<w:basedOn w:val="BodyTextChar"/><w:qFormat/><w:rPr>"#,
    );

    xml.push_str(&build_font_element(vars, "code-font-family"));

    if let Some(color) = get_color_hex(vars, "code-color") {
        xml.push_str(&build_color_element(&color));
    }

    if let Some(size) = get_font_size_half_points(vars, "code-font-size-inline") {
        xml.push_str(&build_size_elements(&size));
    }

    xml.push_str("</w:rPr></w:style>");
    xml
}

/// Build Hyperlink character style
///
/// **CSS Tokens Source**: `web/src/themes/base/links.css`
///
/// **Tokens Applied**:
/// - `link-color` → w:color
///
/// **Tokens NOT Yet Applied**:
/// - `link-decoration` - Would map to w:u (underline), but omitted as DOCX adds underline automatically for hyperlinks
/// - `link-color-visited` / `link-color-hover` - DOCX doesn't support pseudo-states in styles
/// - `link-focus-ring-*` - Not applicable to static documents
fn build_hyperlink_style(vars: &BTreeMap<String, Value>) -> String {
    let mut xml = String::with_capacity(256);

    xml.push_str(
        r#"<w:style w:type="character" w:styleId="Hyperlink">
<w:name w:val="Hyperlink"/>
<w:basedOn w:val="BodyTextChar"/><w:rPr>"#,
    );

    if let Some(color) = get_color_hex(vars, "link-color") {
        xml.push_str(&build_color_element(&color));
    }

    xml.push_str("</w:rPr></w:style>");
    xml
}

/// Build BodyText paragraph style
///
/// **CSS Tokens Source**: `web/src/themes/base/paragraphs.css`
///
/// **Tokens Applied**:
/// - None - BodyText inherits all formatting from Normal
///
/// **Tokens NOT Yet Applied**:
/// - `paragraph-spacing` - Inherited from Normal (w:spacing w:before="0" w:after="paragraph-spacing")
/// - `paragraph-line-height` - DOCX uses automatic line height
/// - `paragraph-text-align` - Inherited from Normal
/// - `paragraph-text-indent` - Inherited from Normal
/// - `paragraph-lead-*` - Lead paragraph enhancement not applicable to DOCX styles
///
/// **Design Note**:
/// BodyText is a linked paragraph style that provides semantic distinction from Normal
/// while maintaining identical formatting. It's used as the default style after headings
/// (see w:next in heading styles) and can be customized independently if needed.
/// Previously set paragraph-spacing on both before/after which caused double-spacing;
/// now relies on inheritance from Normal to maintain consistent spacing behavior.
fn build_body_text_style() -> String {
    r#"<w:style w:type="paragraph" w:styleId="BodyText">
<w:name w:val="Body Text"/>
<w:basedOn w:val="Normal"/>
<w:link w:val="BodyTextChar"/><w:qFormat/><w:pPr></w:pPr><w:rPr></w:rPr></w:style>"#
        .to_string()
}

/// Build DefaultParagraphFont character style
fn build_default_paragraph_font() -> String {
    r#"<w:style w:type="character" w:styleId="DefaultParagraphFont" w:default="1">
<w:name w:val="Default Paragraph Font"/><w:semiHidden/><w:unhideWhenUsed/><w:qFormat/><w:rPr></w:rPr>
</w:style>"#.to_string()
}

/// Build BodyTextChar character style
///
/// **CSS Tokens Source**: `web/src/themes/base/paragraphs.css`
///
/// **Tokens Applied**:
/// - None - BodyTextChar inherits all formatting from DefaultParagraphFont
///
/// **Design Note**:
/// BodyTextChar is a linked character style for BodyText (see w:link in build_body_text_style).
/// It allows BodyText paragraph formatting to be applied at the character level within other
/// paragraph styles. Since there are no body-text-specific character tokens defined in the theme,
/// this style simply provides the linking mechanism without additional formatting.
fn build_body_text_char_style() -> String {
    r#"<w:style w:type="character" w:styleId="BodyTextChar" w:customStyle="1">
<w:name w:val="Body Text Char"/><w:basedOn w:val="DefaultParagraphFont"/><w:qFormat/><w:rPr></w:rPr>
</w:style>"#
        .to_string()
}

/// Build BlockText paragraph style (for quotes)
///
/// **CSS Tokens Source**: `web/src/themes/base/quotes.css`
///
/// **Tokens Applied**:
/// - `quote-font-style` → w:i/w:iCs (if "italic")
/// - `quote-padding` → w:ind w:left and w:right (horizontal indentation)
/// - Fixed spacing (100 twips before/after) → w:spacing
///
/// **Tokens NOT Yet Applied**:
/// - `quote-background` - Would map to w:shd (paragraph shading)
/// - `quote-border-width` / `quote-border-color` - Would map to w:pBdr (paragraph borders)
/// - `quote-border-radius` - Not supported in DOCX
/// - `quote-font-size` - Would map to w:sz/w:szCs
/// - `quote-spacing` - Using fixed values instead
fn build_block_text_style(vars: &BTreeMap<String, Value>) -> String {
    let mut xml = String::with_capacity(512);

    xml.push_str(
        r#"<w:style w:type="paragraph" w:styleId="BlockText">
<w:name w:val="Block Text"/>
<w:basedOn w:val="BodyText"/>
<w:next w:val="BodyText"/>
<w:uiPriority w:val="9"/>
<w:unhideWhenUsed/><w:qFormat/><w:pPr>"#,
    );

    // Use fixed spacing values for quotes
    xml.push_str(&build_spacing_element(Some("100"), Some("100")));

    // Add indentation (left and right)
    if let Some(padding) = get_twips(vars, "quote-padding") {
        xml.push_str(&format!(
            r#"<w:ind w:hanging="0" w:left="{padding}" w:right="{padding}"/>"#
        ));
    }

    xml.push_str("</w:pPr><w:rPr>");

    // Italic if quote-font-style is italic
    if is_italic(vars, "quote-font-style") {
        xml.push_str(r#"<w:i/><w:iCs/>"#);
    }

    xml.push_str("</w:rPr></w:style>");
    xml
}

/// Build Table style
///
/// **CSS Tokens Source**: `web/src/themes/base/tables.css`
///
/// **Tokens Applied**:
/// - `table-cell-padding` → w:tblCellMar (cell margins in twips)
/// - `table-header-font-weight` → w:b/w:bCs in firstRow style (if >= 600)
///
/// **Tokens NOT Yet Applied**:
/// - `table-border-*` - Would map to w:tblBorders (outer table borders)
/// - `table-row-border-*` - Would map to w:tcBorders (cell borders)
/// - `table-header-border-bottom-*` - Would map to w:tcBorders bottom in thead
/// - `table-column-border-*` - Would map to w:tcBorders left/right
/// - `table-header-background` - Would map to w:shd in firstRow
/// - `table-row-hover` / `table-row-striped` - Not supported in static DOCX
/// - `table-cell-font-*` - Would map to w:rPr in table style
/// - `table-border-radius` - Not supported in DOCX
/// - `table-spacing-top` / `table-spacing-bottom` - Margins not in table style
fn build_table_style(vars: &BTreeMap<String, Value>) -> String {
    let mut xml = String::with_capacity(1024);

    xml.push_str(
        r#"<w:style w:type="table" w:default="1" w:styleId="Table">
<w:name w:val="Table"/>
<w:basedOn w:val="TableNormal"/>
<w:semiHidden/><w:unhideWhenUsed/><w:qFormat/>
<w:tblPr><w:tblCellMar>"#,
    );

    // Cell margins (padding)
    let padding = get_twips(vars, "table-cell-padding").unwrap_or_else(|| "108".to_string());
    xml.push_str(&format!(
        r#"<w:top w:w="0" w:type="dxa"/><w:left w:w="{padding}" w:type="dxa"/><w:bottom w:w="0" w:type="dxa"/><w:right w:w="{padding}" w:type="dxa"/>"#
    ));

    xml.push_str(
        r#"</w:tblCellMar></w:tblPr>
<w:tblStylePr w:type="firstRow"><w:tblPr/><w:tcPr>
<w:tcBorders><w:bottom w:val="single"/></w:tcBorders>
<w:vAlign w:val="bottom"/></w:tcPr>"#,
    );

    if is_bold(vars, "table-header-font-weight") {
        xml.push_str(r#"<w:rPr><w:b/><w:bCs/></w:rPr>"#);
    }

    xml.push_str(r#"</w:tblStylePr></w:style>"#);

    xml
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_theme_to_styles_generates_valid_xml() {
        use serde_json::json;

        // Create mock variables (simulating resolved fonts)
        let mut variables = BTreeMap::new();
        variables.insert("text-font-family".to_string(), json!("Noto Serif"));
        variables.insert("text-font-size".to_string(), json!(320.0)); // 16pt in twips
        variables.insert("text-color-primary".to_string(), json!("#000000"));
        variables.insert("heading-font-family".to_string(), json!("Aptos"));
        variables.insert("heading-font-size".to_string(), json!(640.0)); // 32pt in twips
        variables.insert("heading-font-size-ratio".to_string(), json!(0.85));
        variables.insert("heading-color".to_string(), json!("#0F4761"));
        variables.insert("code-font-family".to_string(), json!("Noto Sans Mono"));
        variables.insert("code-color".to_string(), json!("#333333"));
        variables.insert("link-color".to_string(), json!("#4F81BD"));

        // Generate styles
        let styles_xml = theme_to_styles(&variables);

        // Verify basic XML structure
        assert!(
            styles_xml.starts_with(r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>"#)
        );
        assert!(styles_xml.contains(r#"<w:styles"#));
        assert!(styles_xml.ends_with(r#"</w:styles>"#));

        // Verify doc defaults exist
        assert!(styles_xml.contains("<w:docDefaults>"));

        // Verify Normal style exists
        assert!(styles_xml.contains(r#"w:styleId="Normal""#));

        // Verify all 9 heading styles exist
        for level in 1..=9 {
            let style_id = format!(r#"w:styleId="Heading{level}""#);
            assert!(
                styles_xml.contains(&style_id),
                "Missing heading style: {}",
                level
            );
        }

        // Verify character styles exist
        assert!(styles_xml.contains(r#"w:styleId="VerbatimChar""#));
        assert!(styles_xml.contains(r#"w:styleId="Hyperlink""#));

        // Verify paragraph styles exist
        assert!(styles_xml.contains(r#"w:styleId="BodyText""#));
        assert!(styles_xml.contains(r#"w:styleId="BlockText""#));

        // Verify table style exists
        assert!(styles_xml.contains(r#"w:styleId="Table""#));
    }

    #[test]
    fn test_helper_functions() {
        use serde_json::json;
        let mut vars = BTreeMap::new();
        vars.insert("test-color".to_string(), json!("#FF0000"));
        vars.insert("test-size".to_string(), json!(240.0)); // 240 twips = 12pt = 24 half-points
        vars.insert("test-weight".to_string(), json!(700.0));
        vars.insert("test-style".to_string(), json!("italic"));

        // Test color extraction
        assert_eq!(
            get_color_hex(&vars, "test-color"),
            Some("FF0000".to_string())
        );

        // Test font size conversion
        assert_eq!(
            get_font_size_half_points(&vars, "test-size"),
            Some("24".to_string())
        );

        // Test bold check
        assert!(is_bold(&vars, "test-weight"));

        // Test italic check
        assert!(is_italic(&vars, "test-style"));
    }
}
