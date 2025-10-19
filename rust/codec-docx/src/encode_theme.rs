use std::collections::BTreeMap;

use serde_json::Value;

use crate::encode_utils::{
    build_cell_borders_element, build_color_element, build_font_element,
    build_paragraph_left_border_element, build_paragraph_shading_element, build_size_elements,
    build_spacing_element, build_table_borders_element, get_color_hex, get_font_size_half_points,
    get_font_variant_element, get_text_align, get_twips, is_bold, is_italic, twips_to_half_points,
};

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
/// - ✓ `List` (paragraph) → [`build_list_style()`] - List items (markers in numbering.xml)
/// - ✓ `Title` (paragraph) → [`build_title_style()`] - Document title
/// - ✓ `Author` (paragraph) → [`build_author_style()`] - Author name
/// - ✓ `Abstract` (paragraph) → [`build_abstract_style()`] - Abstract section
/// - ✓ `AbstractTitle` (paragraph) → [`build_abstract_title_style()`] - Abstract heading (currently uses Heading1 tokens)
/// - ✓ `DefaultParagraphFont` (character) → [`build_default_paragraph_font()`] - Base character style
/// - ✓ `BodyTextChar` (character) → [`build_body_text_char_style()`] - Body text character variant
/// - ✓ `Heading1Char-9Char` (character) → [`build_heading_char_styles()`] - Heading character variants
/// - ✓ `TitleChar` (character) → [`build_title_char_style()`] - Title character variant
/// - ✓ `VerbatimChar` (character) → [`build_verbatim_char_style()`] - Inline code
/// - ✓ `Hyperlink` (character) → [`build_hyperlink_style()`] - Link formatting
/// - ✓ `Table` (table) → [`build_table_style()`] - Default table style
///
/// **Not Yet Implemented** (✗):
///
/// *Paragraph Styles*:
/// - ✗ `Subtitle` - Document subtitle
/// - ✗ `Date` - Document date
/// - ✗ `Caption` - Generic caption style
/// - ✗ `TableCaption` / `ImageCaption` - Specific caption types
/// - ✗ `Figure` / `CaptionedFigure` - Figure containers
/// - ✗ `FirstParagraph` - First paragraph after heading
/// - ✗ `Compact` - Compact paragraph spacing
/// - ✗ `Bibliography` - Bibliography entries
/// - ✗ `FootnoteText` / `FootnoteBlockText` - Footnote content
/// - ✗ `DefinitionTerm` / `Definition` - Definition lists
/// - ✗ `Index` / `IndexHeading` - Index content
/// - ✗ `TOCHeading` - Table of contents heading
///
/// *Character Styles*:
/// - ✗ `SubtitleChar` - Subtitle character variants
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
/// - ✓/✗ `lists.css` → [`build_list_style()`] - Only spacing applied; markers/indent need numbering.xml (see function docs)
/// - ✓ `tables.css` → [`build_table_style()`]
/// - ✓/✗ `articles.css` → [`build_title_style()`], [`build_author_style()`], [`build_abstract_style()`] - Title, Author, Abstract implemented; content-max-width is layout-only
///
/// **Partially Implemented** (some features work, others pending):
/// - ✓/✗ `pages.css` - Page layout → Section properties (w:sectPr), headers/footers (see [`encode_page_layout`], [`encode_headers_footers`])
///   - ✓ Page size, margins, padding
///   - ✓ Header/footer content (left/center/right positioning)
///   - ✓ Header/footer styling (font, size, color)
///   - ✓ Header/footer borders
///   - ✓ First-page-specific headers/footers
///   - ✗ Page numbering (would need field codes)
///   - ✗ Running headers with section titles (would need field codes)
///
/// **Not Yet Mapped** (could be implemented in future):
/// - ✗ `admonitions.css` - Callout/alert boxes → Custom paragraph styles with borders/shading (w:pBdr, w:shd)
/// - ✗ `breaks.css` - Page/section breaks → Page break runs (w:br w:type="page")
/// - ✗ `captions.css` - Figure/table captions → Extended caption paragraph styles (partially in table style)
/// - ✗ `citations.css` - Citation formatting → Custom character/paragraph styles for bibliography
/// - ✗ `figures.css` - Figure captions/containers → Custom paragraph styles for figure captions
/// - ✗ `labels.css` - Figure/table label formatting → Character styles for label prefixes (e.g., "Figure 1:")
/// - ✗ `references.css` - Cross-reference styling → Custom character styles for internal references
///
/// **No Mapping Applicable** (not relevant for static DOCX):
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

    // Article metadata styles
    xml.push_str(&build_title_style(variables));
    xml.push_str(&build_author_style(variables));
    xml.push_str(&build_abstract_style(variables));
    xml.push_str(&build_abstract_title_style(variables));

    // Character styles
    xml.push_str(&build_title_char_style(variables));
    xml.push_str(&build_heading_char_styles(variables));
    xml.push_str(&build_default_paragraph_font());
    xml.push_str(&build_body_text_char_style());
    xml.push_str(&build_verbatim_char_style(variables));
    xml.push_str(&build_hyperlink_style(variables));

    // Paragraph styles
    xml.push_str(&build_body_text_style());
    xml.push_str(&build_block_text_style(variables));
    xml.push_str(&build_list_style(variables));

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
/// - `quote-font-size` → w:sz/w:szCs (text-font-size * 1.125, i.e., 12.5% larger)
/// - `quote-padding` → w:ind w:left and w:right (horizontal indentation)
/// - `quote-spacing` → w:spacing w:before and w:after (vertical spacing)
/// - `quote-background` → w:shd (paragraph shading/background color)
/// - `quote-border-width` + `quote-border-color` → w:pBdr left border
///
/// **Tokens NOT Yet Applied**:
/// - `quote-border-radius` - Not supported in DOCX (paragraph borders are rectangular)
/// - `quote-spacing-horizontal` - Not applicable (handled by document margins, not paragraph style)
///
/// **Design Note**:
/// The CSS applies `border-left` only, so the DOCX style mirrors this with a left-only
/// paragraph border. Background shading provides visual distinction similar to CSS.
fn build_block_text_style(vars: &BTreeMap<String, Value>) -> String {
    let mut xml = String::with_capacity(1024);

    xml.push_str(
        r#"<w:style w:type="paragraph" w:styleId="BlockText">
<w:name w:val="Block Text"/>
<w:basedOn w:val="BodyText"/>
<w:next w:val="BodyText"/>
<w:uiPriority w:val="9"/>
<w:unhideWhenUsed/><w:qFormat/><w:pPr>"#,
    );

    // Spacing (before and after paragraph)
    let before = get_twips(vars, "quote-spacing");
    let after = get_twips(vars, "quote-spacing");
    xml.push_str(&build_spacing_element(before.as_deref(), after.as_deref()));

    // Indentation (left and right padding)
    if let Some(padding) = get_twips(vars, "quote-padding") {
        xml.push_str(&format!(
            r#"<w:ind w:hanging="0" w:left="{padding}" w:right="{padding}"/>"#
        ));
    }

    // Background shading
    if let Some(bg_color) = get_color_hex(vars, "quote-background") {
        xml.push_str(&build_paragraph_shading_element(&bg_color));
    }

    // Left border (matching CSS border-left)
    // Note: border width is already in twips from theme variable computation
    if let (Some(border_width), Some(border_color)) = (
        get_twips(vars, "quote-border-width"),
        get_color_hex(vars, "quote-border-color"),
    ) {
        xml.push_str(&build_paragraph_left_border_element(
            &border_width,
            &border_color,
        ));
    }

    xml.push_str("</w:pPr><w:rPr>");

    // Italic if quote-font-style is italic
    if is_italic(vars, "quote-font-style") {
        xml.push_str(r#"<w:i/><w:iCs/>"#);
    }

    // Font size: quote-font-size = text-font-size * 1.125 (12.5% larger)
    if let Some(base_size) = vars.get("text-font-size").and_then(|v| v.as_f64()) {
        let quote_size = base_size * 1.125;
        let half_points = twips_to_half_points(quote_size);
        xml.push_str(&build_size_elements(&half_points));
    }

    xml.push_str("</w:rPr></w:style>");
    xml
}

/// Build List paragraph style
///
/// **CSS Tokens Source**: `web/src/themes/base/lists.css`
///
/// **Tokens Applied**:
/// - `list-spacing` → w:spacing w:before and w:after (spacing before/after list)
/// - `list-item-spacing` → w:spacing w:after (spacing between list items)
///
/// **Tokens NOT Applied**:
/// - `list-indent` / `list-indent-nested` - Would need to be in numbering.xml
/// - `list-marker-*` - Would need to be in numbering.xml
///
/// **Why List Markers Aren't Generated**:
///
/// In theory, the list-marker-* tokens (marker type, color, etc.) could be applied by
/// generating a `numbering.xml` file with DOCX numbering definitions. However, this approach
/// has a critical limitation:
///
/// **Pandoc's numbering IDs are not predictable.** Each time Pandoc processes a document,
/// it generates different `w:abstractNumId` and `w:numId` values in `numbering.xml`. Without
/// knowing these IDs in advance, we cannot reliably reference them from our generated styles
/// or document content. This makes it impossible to create a working connection between
/// paragraph styles and numbering definitions.
///
/// As a result, this implementation only applies list spacing tokens. List markers and
/// indentation remain controlled by Pandoc's `numbering.xml` generation.
fn build_list_style(vars: &BTreeMap<String, Value>) -> String {
    let mut xml = String::with_capacity(512);

    xml.push_str(
        r#"<w:style w:type="paragraph" w:styleId="List">
<w:name w:val="List"/>
<w:basedOn w:val="BodyText"/>
<w:qFormat/>
<w:pPr>"#,
    );

    // Spacing before/after list
    let before = get_twips(vars, "list-spacing");
    let after = get_twips(vars, "list-spacing");
    xml.push_str(&build_spacing_element(before.as_deref(), after.as_deref()));

    xml.push_str(r#"</w:pPr><w:rPr></w:rPr></w:style>"#);
    xml
}

/// Build Title paragraph style
///
/// **CSS Tokens Source**: `web/src/themes/base/articles.css`
///
/// **Tokens Applied**:
/// - `article-title-font-family` → w:rFonts
/// - `article-title-font-size-print` (with fallback to `article-title-font-size`) → w:sz/w:szCs
/// - `article-title-font-weight` → w:b/w:bCs (if >= 600)
/// - `article-title-color` → w:color
/// - `article-title-text-align` → w:jc
/// - `article-title-letter-spacing` → w:spacing w:val
/// - `article-title-margin-bottom` → w:spacing w:after
///
/// **Tokens NOT Yet Applied**:
/// - `article-title-line-height` - DOCX uses automatic line height
/// - `article-title-max-width` - Layout constraint not applicable to paragraph styles
///
/// **Design Note**:
/// Title is linked to TitleChar character style and uses keep-with-next to prevent
/// the title from being orphaned at the bottom of a page.
fn build_title_style(vars: &BTreeMap<String, Value>) -> String {
    let mut xml = String::with_capacity(768);

    xml.push_str(
        r#"<w:style w:type="paragraph" w:styleId="Title">
<w:name w:val="Title"/>
<w:basedOn w:val="Normal"/>
<w:next w:val="BodyText"/>
<w:link w:val="TitleChar"/>
<w:uiPriority w:val="10"/>
<w:qFormat/>
<w:pPr>"#,
    );

    xml.push_str(KEEP_NEXT);

    // Spacing (margin-bottom)
    let after = get_twips(vars, "article-title-margin-bottom");
    xml.push_str(&build_spacing_element(None, after.as_deref()));

    // Text alignment
    let alignment = get_text_align(vars, "article-title-text-align");
    xml.push_str(&format!(r#"<w:jc w:val="{alignment}"/></w:pPr><w:rPr>"#));

    // Font family
    xml.push_str(&build_font_element(vars, "article-title-font-family"));

    // Color
    if let Some(color) = get_color_hex(vars, "article-title-color") {
        xml.push_str(&build_color_element(&color));
    }

    // Font size - prefer print variant
    let size = get_font_size_half_points(vars, "article-title-font-size-print")
        .or_else(|| get_font_size_half_points(vars, "article-title-font-size"));
    if let Some(size) = size {
        xml.push_str(&build_size_elements(&size));
    }

    // Bold if weight >= 600
    if is_bold(vars, "article-title-font-weight") {
        xml.push_str(r#"<w:b/><w:bCs/>"#);
    }

    // Letter spacing
    if let Some(spacing) = get_twips(vars, "article-title-letter-spacing") {
        xml.push_str(&format!(r#"<w:spacing w:val="{spacing}"/>"#));
    }

    xml.push_str("</w:rPr></w:style>");
    xml
}

/// Build TitleChar character style
///
/// **CSS Tokens Source**: `web/src/themes/base/articles.css`
///
/// Same tokens as `build_title_style()` but applied as character-level formatting.
/// This linked character style allows title formatting to be applied to text runs
/// within paragraphs without changing the paragraph style itself.
fn build_title_char_style(vars: &BTreeMap<String, Value>) -> String {
    let mut xml = String::with_capacity(512);

    xml.push_str(
        r#"<w:style w:type="character" w:styleId="TitleChar" w:customStyle="1">
<w:name w:val="Title Char"/>
<w:basedOn w:val="DefaultParagraphFont"/>
<w:link w:val="Title"/>
<w:uiPriority w:val="10"/>
<w:qFormat/>
<w:rPr>"#,
    );

    // Font family
    xml.push_str(&build_font_element(vars, "article-title-font-family"));

    // Color
    if let Some(color) = get_color_hex(vars, "article-title-color") {
        xml.push_str(&build_color_element(&color));
    }

    // Font size - prefer print variant
    let size = get_font_size_half_points(vars, "article-title-font-size-print")
        .or_else(|| get_font_size_half_points(vars, "article-title-font-size"));
    if let Some(size) = size {
        xml.push_str(&build_size_elements(&size));
    }

    // Bold if weight >= 600
    if is_bold(vars, "article-title-font-weight") {
        xml.push_str(r#"<w:b/><w:bCs/>"#);
    }

    // Letter spacing
    if let Some(spacing) = get_twips(vars, "article-title-letter-spacing") {
        xml.push_str(&format!(r#"<w:spacing w:val="{spacing}"/>"#));
    }

    xml.push_str("</w:rPr></w:style>");
    xml
}

/// Build Author paragraph style
///
/// **CSS Tokens Source**: `web/src/themes/base/articles.css`
///
/// **Tokens Applied**:
/// - `article-authors-font-size` → w:sz/w:szCs
/// - `article-authors-color` → w:color
/// - `article-authors-text-align` → w:jc
/// - `article-authors-margin-bottom` → w:spacing w:after
///
/// **Design Note**:
/// Author style is based on Normal and flows to BodyText for the next paragraph.
fn build_author_style(vars: &BTreeMap<String, Value>) -> String {
    let mut xml = String::with_capacity(512);

    xml.push_str(
        r#"<w:style w:type="paragraph" w:styleId="Author">
<w:name w:val="Author"/>
<w:basedOn w:val="Normal"/>
<w:next w:val="BodyText"/>
<w:uiPriority w:val="10"/>
<w:qFormat/>
<w:pPr>"#,
    );

    // Spacing (margin-bottom)
    let after = get_twips(vars, "article-authors-margin-bottom");
    xml.push_str(&build_spacing_element(None, after.as_deref()));

    // Text alignment
    let alignment = get_text_align(vars, "article-authors-text-align");
    xml.push_str(&format!(r#"<w:jc w:val="{alignment}"/></w:pPr><w:rPr>"#));

    // Color
    if let Some(color) = get_color_hex(vars, "article-authors-color") {
        xml.push_str(&build_color_element(&color));
    }

    // Font size
    if let Some(size) = get_font_size_half_points(vars, "article-authors-font-size") {
        xml.push_str(&build_size_elements(&size));
    }

    xml.push_str("</w:rPr></w:style>");
    xml
}

/// Build Abstract paragraph style
///
/// **CSS Tokens Source**: `web/src/themes/base/articles.css`
///
/// **Tokens Applied**:
/// - `article-abstract-font-size` → w:sz/w:szCs
/// - `article-abstract-color` → w:color
/// - `article-abstract-background` → w:shd (paragraph shading)
/// - `article-abstract-text-align` → w:jc
/// - `article-abstract-margin-bottom` → w:spacing w:after
///
/// **Tokens NOT Yet Applied**:
/// - `article-abstract-max-width` - Layout constraint not applicable to paragraph styles
///
/// **Design Note**:
/// Abstract style is based on Normal and includes optional background shading.
fn build_abstract_style(vars: &BTreeMap<String, Value>) -> String {
    let mut xml = String::with_capacity(768);

    xml.push_str(
        r#"<w:style w:type="paragraph" w:styleId="Abstract">
<w:name w:val="Abstract"/>
<w:basedOn w:val="Normal"/>
<w:next w:val="BodyText"/>
<w:uiPriority w:val="10"/>
<w:qFormat/>
<w:pPr>"#,
    );

    // Spacing (margin-bottom)
    let after = get_twips(vars, "article-abstract-margin-bottom");
    xml.push_str(&build_spacing_element(None, after.as_deref()));

    // Text alignment
    let alignment = get_text_align(vars, "article-abstract-text-align");
    xml.push_str(&format!(r#"<w:jc w:val="{alignment}"/>"#));

    // Background shading
    if let Some(bg_color) = get_color_hex(vars, "article-abstract-background") {
        xml.push_str(&build_paragraph_shading_element(&bg_color));
    }

    xml.push_str("</w:pPr><w:rPr>");

    // Color
    if let Some(color) = get_color_hex(vars, "article-abstract-color") {
        xml.push_str(&build_color_element(&color));
    }

    // Font size
    if let Some(size) = get_font_size_half_points(vars, "article-abstract-font-size") {
        xml.push_str(&build_size_elements(&size));
    }

    xml.push_str("</w:rPr></w:style>");
    xml
}

/// Build AbstractTitle paragraph style
///
/// **CSS Tokens Source**: None currently defined
///
/// **Tokens Applied**:
/// Currently uses the same tokens as Heading1 from `web/src/themes/base/headings.css`:
/// - `heading-font-family` → w:rFonts
/// - `heading-color` → w:color
/// - `heading-font-size` with `heading-font-size-ratio` → w:sz (base size, no exponential scaling)
/// - `heading-h1-font-weight` → w:b/w:bCs (if >= 600)
/// - `heading-h1-font-style` → w:i/w:iCs (if "italic")
/// - `heading-h1-font-variant` → w:smallCaps or w:caps
/// - `heading-h1-letter-spacing` → w:spacing w:val
/// - `heading-spacing-top-1` → w:spacing w:before
/// - `heading-spacing-bottom` → w:spacing w:after
///
/// **Design Note**:
/// This style currently reuses Heading1 formatting as there are no specific
/// `abstract-title-*` design tokens defined in the theme system yet. In the future,
/// if abstract-specific title tokens are added to `articles.css` (e.g.,
/// `article-abstract-title-font-size`, `article-abstract-title-color`, etc.),
/// this function should be updated to use those tokens instead.
///
/// AbstractTitle is used for the "Abstract" heading that appears before abstract content.
/// It is based on Normal and flows to Abstract for the next paragraph.
fn build_abstract_title_style(vars: &BTreeMap<String, Value>) -> String {
    let mut xml = String::with_capacity(768);

    xml.push_str(
        r#"<w:style w:type="paragraph" w:styleId="AbstractTitle">
<w:name w:val="Abstract Title"/>
<w:basedOn w:val="Normal"/>
<w:next w:val="Abstract"/>
<w:uiPriority w:val="10"/>
<w:qFormat/>
<w:pPr>"#,
    );

    xml.push_str(KEEP_NEXT);
    xml.push_str(KEEP_LINES);

    // Spacing (reusing heading-1 spacing)
    let before = get_twips(vars, "heading-spacing-top-1");
    let after = get_twips(vars, "heading-spacing-bottom");
    xml.push_str(&build_spacing_element(before.as_deref(), after.as_deref()));

    xml.push_str(r#"</w:pPr><w:rPr>"#);

    // Font family
    xml.push_str(&build_font_element(vars, "heading-font-family"));

    // Color
    if let Some(color) = get_color_hex(vars, "heading-color") {
        xml.push_str(&build_color_element(&color));
    }

    // Font size: use base heading size without ratio scaling
    if let Some(base_size) = vars.get("heading-font-size").and_then(|v| v.as_f64()) {
        let half_points = twips_to_half_points(base_size);
        xml.push_str(&build_size_elements(&half_points));
    }

    // Bold/italic from h1-specific variables
    if is_bold(vars, "heading-h1-font-weight") {
        xml.push_str(r#"<w:b/><w:bCs/>"#);
    }

    if is_italic(vars, "heading-h1-font-style") {
        xml.push_str(r#"<w:i/><w:iCs/>"#);
    }

    // Font variant (small-caps, all-caps)
    xml.push_str(&get_font_variant_element(vars, "heading-h1-font-variant"));

    // Letter spacing
    if let Some(spacing) = get_twips(vars, "heading-h1-letter-spacing") {
        xml.push_str(&format!(r#"<w:spacing w:val="{spacing}"/>"#));
    }

    xml.push_str("</w:rPr></w:style>");
    xml
}

/// Build Table style
///
/// **CSS Tokens Source**: `web/src/themes/base/tables.css`
///
/// **Tokens Applied**:
/// - ✓ `table-cell-padding` → w:tblCellMar (cell margins in twips)
/// - ✓ `table-cell-font-family` → w:rFonts in wholeTable
/// - ✓ `table-cell-font-size` → w:sz/w:szCs in wholeTable
/// - ✓ `table-border-top/bottom/left/right-width/color/style` → w:tblBorders (outer table borders)
/// - ✓ `table-header-background` → w:shd in firstRow
/// - ✓ `table-header-font-weight` → w:b/w:bCs in firstRow
/// - ✓ `table-header-border-bottom-width/color/style` → w:tcBorders bottom in firstRow
/// - ✓ `table-body-row-border-width/color/style` → w:tcBorders bottom in band1Horz
/// - ✓ `table-column-border-width/color/style` → w:tcBorders left in band1Vert (if width > 0)
///
/// **Tokens NOT Yet Applied**:
/// - ✗ `table-border-radius` - Not supported in DOCX (borders are rectangular)
/// - ✗ `table-row-hover` - Not supported in static DOCX (interactive state)
/// - ✗ `table-row-striped` - Not supported in this implementation (could use band2Horz)
/// - ✗ `table-spacing-top/bottom` - Block-level margins (handled at document level, not style)
/// - ✗ `table-caption-*` / `table-notes-*` - Caption/notes are separate elements, not table style
/// - ✗ `table-cell-line-height` - DOCX uses automatic line height
///
/// **Design Notes**:
/// - Cell padding is applied horizontally (left/right) but not vertically (top/bottom = 0)
///   to match common DOCX table styling conventions
/// - Outer table borders provide the table frame/perimeter
/// - Header border is applied only to bottom edge to separate header from body rows
/// - Body row borders are applied to bottom edge for horizontal row separators
/// - Column borders are only applied if width > 0 (many themes have column-border-width: 0)
///
/// **Compatibility Notes**:
/// - **Google Docs**: Has incomplete support for DOCX table borders defined in table styles
///   (`w:tblBorders` in `w:tblPr`). Outer table borders may not render in Google Docs even
///   though they work correctly in Microsoft Word and LibreOffice. Google Docs appears to
///   require borders defined at the cell level rather than the table style level. This is a
///   known Google Docs limitation, not an issue with our implementation.
///   See: <https://support.google.com/drive/thread/192657375/table-borders-missing-when-uploading-a-word-document>
/// - **LibreOffice**: May have issues rendering fonts from `w:tblStylePr type="wholeTable"`
///   when paragraph styles are present in table cells. Fonts display correctly in Google Docs
///   and Microsoft Word.
fn build_table_style(vars: &BTreeMap<String, Value>) -> String {
    let mut xml = String::with_capacity(2048);

    xml.push_str(
        r#"<w:style w:type="table" w:default="1" w:styleId="Table">
<w:name w:val="Table"/>
<w:basedOn w:val="TableNormal"/>
<w:semiHidden/><w:unhideWhenUsed/><w:qFormat/>
<w:tblPr><w:tblCellMar>"#,
    );

    // Cell margins (padding) - horizontal only, vertical = 0
    // Default to 108 twips (~0.075 inch) if not specified, matching common DOCX defaults
    let padding = get_twips(vars, "table-cell-padding").unwrap_or_else(|| "108".to_string());
    xml.push_str(&format!(
        r#"<w:top w:w="0" w:type="dxa"/><w:left w:w="{padding}" w:type="dxa"/><w:bottom w:w="0" w:type="dxa"/><w:right w:w="{padding}" w:type="dxa"/>"#
    ));

    xml.push_str(r#"</w:tblCellMar>"#);

    // Outer table borders
    let table_borders = build_table_borders_element(
        vars,
        (
            "table-border-top-width",
            "table-border-top-color",
            "table-border-top-style",
        ),
        (
            "table-border-bottom-width",
            "table-border-bottom-color",
            "table-border-bottom-style",
        ),
        (
            "table-border-left-width",
            "table-border-left-color",
            "table-border-left-style",
        ),
        (
            "table-border-right-width",
            "table-border-right-color",
            "table-border-right-style",
        ),
    );
    xml.push_str(&table_borders);

    xml.push_str(r#"</w:tblPr>"#);

    // Default cell formatting (applies to all cells unless overridden)
    // Note: Character formatting must be in w:tblStylePr, not directly under w:style
    xml.push_str(r#"<w:tblStylePr w:type="wholeTable"><w:rPr>"#);

    // Cell font family
    xml.push_str(&build_font_element(vars, "table-cell-font-family"));

    // Cell font size
    if let Some(size) = get_font_size_half_points(vars, "table-cell-font-size") {
        xml.push_str(&build_size_elements(&size));
    }

    xml.push_str(r#"</w:rPr></w:tblStylePr>"#);

    // First row (header) styling
    xml.push_str(r#"<w:tblStylePr w:type="firstRow"><w:tblPr/><w:tcPr>"#);

    // Header background color
    if let Some(bg_color) = get_color_hex(vars, "table-header-background") {
        xml.push_str(&format!(
            r#"<w:shd w:val="clear" w:color="auto" w:fill="{bg_color}"/>"#
        ));
    }

    // Header bottom border
    let header_borders = build_cell_borders_element(
        vars,
        None, // no top border
        Some((
            "table-header-border-bottom-width",
            "table-header-border-bottom-color",
            "table-header-border-bottom-style",
        )),
        None, // no left border (handled by column borders)
        None, // no right border (handled by column borders)
    );
    xml.push_str(&header_borders);

    xml.push_str(r#"<w:vAlign w:val="bottom"/></w:tcPr>"#);

    // Header font weight (bold)
    if is_bold(vars, "table-header-font-weight") {
        xml.push_str(r#"<w:rPr><w:b/><w:bCs/></w:rPr>"#);
    }

    xml.push_str(r#"</w:tblStylePr>"#);

    // Body row borders (horizontal separators between rows)
    // Using band1Horz to apply to all body rows
    xml.push_str(r#"<w:tblStylePr w:type="band1Horz"><w:tcPr>"#);

    let body_borders = build_cell_borders_element(
        vars,
        None, // no top border
        Some((
            "table-body-row-border-width",
            "table-body-row-border-color",
            "table-body-row-border-style",
        )),
        None, // no left border (handled by column borders)
        None, // no right border (handled by column borders)
    );
    xml.push_str(&body_borders);

    xml.push_str(r#"</w:tcPr></w:tblStylePr>"#);

    // Column borders (vertical separators between columns)
    // Only add if column border width > 0
    if let Some(col_border_width) = get_twips(vars, "table-column-border-width")
        && col_border_width.parse::<f64>().unwrap_or(0.0) > 0.0
    {
        xml.push_str(r#"<w:tblStylePr w:type="band1Vert"><w:tcPr>"#);

        let col_borders = build_cell_borders_element(
            vars,
            None, // no top border
            None, // no bottom border
            Some((
                "table-column-border-width",
                "table-column-border-color",
                "table-column-border-style",
            )),
            None, // right border will be the left border of the next cell
        );
        xml.push_str(&col_borders);

        xml.push_str(r#"</w:tcPr></w:tblStylePr>"#);
    }

    xml.push_str(r#"</w:style>"#);

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
        assert!(styles_xml.contains(r#"w:styleId="Title""#));
        assert!(styles_xml.contains(r#"w:styleId="Author""#));
        assert!(styles_xml.contains(r#"w:styleId="Abstract""#));
        assert!(styles_xml.contains(r#"w:styleId="AbstractTitle""#));

        // Verify article-related character styles exist
        assert!(styles_xml.contains(r#"w:styleId="TitleChar""#));

        // Verify table style exists
        assert!(styles_xml.contains(r#"w:styleId="Table""#));
    }
}
