use std::collections::BTreeMap;

use serde_json::Value;
use stencila_codec::{EncodeOptions, eyre::Result};
use stencila_themes::LengthConversion;

use crate::html_escape;

/// Get theme variables for MJML encoding
pub async fn get_theme_vars(options: &EncodeOptions) -> Result<Option<BTreeMap<String, Value>>> {
    let theme = stencila_themes::get(options.theme.clone(), options.from_path.clone()).await?;
    if let Some(theme) = theme {
        Ok(Some(theme.computed_variables_with_overrides(
            LengthConversion::Pixels,
            BTreeMap::new(),
        )))
    } else {
        Ok(None)
    }
}

/// Get a string value from theme variables
fn get_var(vars: Option<&BTreeMap<String, Value>>, key: &str) -> Option<String> {
    vars.and_then(|v| v.get(key)).and_then(|v| match v {
        Value::String(s) => Some(s.clone()),
        Value::Number(n) => Some(n.to_string()),
        _ => None,
    })
}

/// Encode theme attributes into mj-attributes
pub fn encode_theme_attributes(mjml: &mut String, theme_vars: Option<&BTreeMap<String, Value>>) {
    let font_family =
        get_var(theme_vars, "text-font-family").unwrap_or_else(|| "Arial, sans-serif".to_string());
    let text_color =
        get_var(theme_vars, "text-color-primary").unwrap_or_else(|| "#000000".to_string());
    let font_size = get_var(theme_vars, "text-font-size").unwrap_or_else(|| "16".to_string());

    mjml.push_str(&format!(
        "      <mj-all font-family=\"{}\" color=\"{}\"/>\n",
        html_escape(&font_family),
        html_escape(&text_color)
    ));
    mjml.push_str(&format!(
        "      <mj-text font-size=\"{}px\" line-height=\"1.5\"/>\n",
        font_size
    ));

    // Divider styling - use border-color-default which is the correct semantic token
    if let Some(border_color) = get_var(theme_vars, "border-color-default") {
        mjml.push_str(&format!(
            "      <mj-divider border-color=\"{}\"/>\n",
            html_escape(&border_color)
        ));
    }
}

/// Encode theme styles into mj-style
pub fn encode_theme_styles(mjml: &mut String, theme_vars: Option<&BTreeMap<String, Value>>) {
    // Text colors
    let text_color_secondary =
        get_var(theme_vars, "text-color-secondary").unwrap_or_else(|| "#555555".to_string());
    let text_color_muted =
        get_var(theme_vars, "text-color-muted").unwrap_or_else(|| "#666666".to_string());
    let text_line_height =
        get_var(theme_vars, "text-line-height").unwrap_or_else(|| "1.5".to_string());

    // Heading styles
    let heading_font = get_var(theme_vars, "heading-font-family")
        .unwrap_or_else(|| "Arial, sans-serif".to_string());
    let heading_color =
        get_var(theme_vars, "heading-color").unwrap_or_else(|| "#000000".to_string());
    let heading_line_height =
        get_var(theme_vars, "heading-line-height").unwrap_or_else(|| "1.2".to_string());
    let heading_letter_spacing =
        get_var(theme_vars, "heading-letter-spacing").unwrap_or_else(|| "-0.025em".to_string());
    let heading_font_weight =
        get_var(theme_vars, "heading-font-weight").unwrap_or_else(|| "700".to_string());

    // Heading font sizes - base size and ratio for calculating per-level sizes
    // Default: h1=32px (2.0×16), then each level decreases by 0.85 ratio
    let heading_font_size = get_var(theme_vars, "heading-font-size")
        .and_then(|s| s.parse::<f64>().ok())
        .unwrap_or(32.0);
    let heading_font_size_ratio = get_var(theme_vars, "heading-font-size-ratio")
        .and_then(|s| s.parse::<f64>().ok())
        .unwrap_or(0.85);

    // Calculate per-level font sizes
    let h1_font_size = heading_font_size;
    let h2_font_size = heading_font_size * heading_font_size_ratio;
    let h3_font_size = h2_font_size * heading_font_size_ratio;
    let h4_font_size = h3_font_size * heading_font_size_ratio;
    let h5_font_size = h4_font_size * heading_font_size_ratio;
    let h6_font_size = h5_font_size * heading_font_size_ratio;

    // Link styles
    let link_color = get_var(theme_vars, "link-color").unwrap_or_else(|| "#0066cc".to_string());
    let link_decoration =
        get_var(theme_vars, "link-decoration").unwrap_or_else(|| "none".to_string());
    let link_color_hover =
        get_var(theme_vars, "link-color-hover").unwrap_or_else(|| "#0044aa".to_string());

    // Code styles
    let code_font =
        get_var(theme_vars, "code-font-family").unwrap_or_else(|| "monospace".to_string());
    let code_color = get_var(theme_vars, "code-color").unwrap_or_else(|| "#333333".to_string());
    let code_background =
        get_var(theme_vars, "code-background").unwrap_or_else(|| "#f5f5f5".to_string());
    let code_border_color =
        get_var(theme_vars, "code-border-color").unwrap_or_else(|| "#e0e0e0".to_string());
    let code_border_radius =
        get_var(theme_vars, "code-border-radius").unwrap_or_else(|| "4".to_string());
    let code_line_height =
        get_var(theme_vars, "code-line-height").unwrap_or_else(|| "150%".to_string());

    // Quote styles
    let quote_background =
        get_var(theme_vars, "quote-background").unwrap_or_else(|| "#f9f9f9".to_string());
    let quote_border_width =
        get_var(theme_vars, "quote-border-width").unwrap_or_else(|| "3".to_string());
    let quote_border_color =
        get_var(theme_vars, "quote-border-color").unwrap_or_else(|| "#cccccc".to_string());
    let quote_font_style =
        get_var(theme_vars, "quote-font-style").unwrap_or_else(|| "italic".to_string());
    let quote_padding = get_var(theme_vars, "quote-padding").unwrap_or_else(|| "16".to_string());

    // Table styles
    let table_border_color =
        get_var(theme_vars, "table-border-color").unwrap_or_else(|| "#dddddd".to_string());
    let table_header_background =
        get_var(theme_vars, "table-header-background").unwrap_or_else(|| "#f5f5f5".to_string());
    let table_header_font_weight =
        get_var(theme_vars, "table-header-font-weight").unwrap_or_else(|| "600".to_string());
    let table_cell_padding =
        get_var(theme_vars, "table-cell-padding").unwrap_or_else(|| "8".to_string());

    // Article title styles
    let article_title_font_family =
        get_var(theme_vars, "article-title-font-family").unwrap_or_else(|| heading_font.clone());
    let article_title_font_weight =
        get_var(theme_vars, "article-title-font-weight").unwrap_or_else(|| "700".to_string());
    let article_title_color =
        get_var(theme_vars, "article-title-color").unwrap_or_else(|| heading_color.clone());
    let article_title_text_align =
        get_var(theme_vars, "article-title-text-align").unwrap_or_else(|| "center".to_string());
    let article_title_letter_spacing = get_var(theme_vars, "article-title-letter-spacing")
        .unwrap_or_else(|| "-0.02em".to_string());
    let article_title_line_height =
        get_var(theme_vars, "article-title-line-height").unwrap_or_else(|| "1.1".to_string());

    // Article authors styles
    let article_authors_font_size =
        get_var(theme_vars, "article-authors-font-size").unwrap_or_else(|| "16".to_string());
    let article_authors_color = get_var(theme_vars, "article-authors-color")
        .unwrap_or_else(|| text_color_secondary.clone());
    let article_authors_text_align =
        get_var(theme_vars, "article-authors-text-align").unwrap_or_else(|| "left".to_string());
    let article_authors_margin_bottom =
        get_var(theme_vars, "article-authors-margin-bottom").unwrap_or_else(|| "24".to_string());

    // Abstract styles
    let article_abstract_font_size =
        get_var(theme_vars, "article-abstract-font-size").unwrap_or_else(|| "16".to_string());
    let article_abstract_background =
        get_var(theme_vars, "article-abstract-background").unwrap_or_else(|| "#f9f9f9".to_string());
    let article_abstract_color = get_var(theme_vars, "article-abstract-color")
        .unwrap_or_else(|| text_color_secondary.clone());
    let article_abstract_text_align =
        get_var(theme_vars, "article-abstract-text-align").unwrap_or_else(|| "left".to_string());
    let article_abstract_margin_bottom =
        get_var(theme_vars, "article-abstract-margin-bottom").unwrap_or_else(|| "24".to_string());

    // Border radius
    let border_radius_default =
        get_var(theme_vars, "border-radius-default").unwrap_or_else(|| "4".to_string());

    // List styles
    let list_indent = get_var(theme_vars, "list-indent").unwrap_or_else(|| "24".to_string());
    let list_item_spacing =
        get_var(theme_vars, "list-item-spacing").unwrap_or_else(|| "4".to_string());
    let list_marker_color =
        get_var(theme_vars, "list-marker-color").unwrap_or_else(|| "#666666".to_string());

    // Surface colors
    let surface_background =
        get_var(theme_vars, "surface-background").unwrap_or_else(|| "#ffffff".to_string());
    let border_color_default =
        get_var(theme_vars, "border-color-default").unwrap_or_else(|| "#e0e0e0".to_string());

    mjml.push_str(&format!(
        r#"
      /* Base typography */
      body {{
        background-color: {surface_background};
        line-height: {text_line_height};
      }}
      p {{
        margin: 0 0 1em 0;
        line-height: {text_line_height};
      }}

      /* Headings - base styles */
      h1, h2, h3, h4, h5, h6 {{
        font-family: {heading_font};
        color: {heading_color};
        line-height: {heading_line_height};
        letter-spacing: {heading_letter_spacing};
        font-weight: {heading_font_weight};
        margin: 0 0 0.5em 0;
      }}

      /* Heading sizes - calculated from base size and ratio */
      h1 {{ font-size: {h1_font_size:.1}px; }}
      h2 {{ font-size: {h2_font_size:.1}px; }}
      h3 {{ font-size: {h3_font_size:.1}px; }}
      h4 {{ font-size: {h4_font_size:.1}px; }}
      h5 {{ font-size: {h5_font_size:.1}px; }}
      h6 {{ font-size: {h6_font_size:.1}px; }}

      /* Article title (h1 in title slot) - overrides heading h1 */
      .article-title h1 {{
        font-family: {article_title_font_family};
        font-weight: {article_title_font_weight};
        color: {article_title_color};
        text-align: {article_title_text_align};
        letter-spacing: {article_title_letter_spacing};
        line-height: {article_title_line_height};
      }}

      /* Links */
      a {{
        color: {link_color};
        text-decoration: {link_decoration};
      }}
      a:hover {{
        color: {link_color_hover};
        text-decoration: underline;
      }}

      /* Code */
      code {{
        font-family: {code_font};
        font-size: 0.9em;
        color: {code_color};
        background-color: {code_background};
        border: 1px solid {code_border_color};
        padding: 2px 4px;
        border-radius: {code_border_radius}px;
        line-height: {code_line_height};
      }}
      pre {{
        font-family: {code_font};
        font-size: 0.9em;
        color: {code_color};
        background-color: {code_background};
        border: 1px solid {code_border_color};
        border-radius: {code_border_radius}px;
        padding: 12px;
        overflow-x: auto;
        line-height: {code_line_height};
      }}
      pre code {{
        border: none;
        padding: 0;
        background: none;
      }}

      /* Blockquotes */
      blockquote {{
        border-left: {quote_border_width}px solid {quote_border_color};
        background-color: {quote_background};
        margin: 0;
        padding: {quote_padding}px;
        font-style: {quote_font_style};
      }}
      blockquote p {{
        margin: 0;
      }}

      /* Authors - using article author tokens */
      .authors {{
        font-size: {article_authors_font_size}px;
        color: {article_authors_color};
        text-align: {article_authors_text_align};
        margin-bottom: {article_authors_margin_bottom}px;
      }}

      /* Abstract - using article abstract tokens */
      .abstract {{
        font-size: {article_abstract_font_size}px;
        background-color: {article_abstract_background};
        color: {article_abstract_color};
        text-align: {article_abstract_text_align};
        padding: 16px;
        margin: 0 0 {article_abstract_margin_bottom}px 0;
        border-radius: {border_radius_default}px;
        border-left: 3px solid {border_color_default};
      }}
      .abstract strong {{
        display: block;
        margin-bottom: 0.5em;
      }}
      .abstract p {{
        text-align: {article_abstract_text_align};
      }}

      /* References */
      .references {{
        font-size: 0.875em;
        color: {text_color_muted};
      }}
      .references li {{
        margin-bottom: 0.5em;
      }}

      /* Tables */
      table.content-table {{
        border-collapse: collapse;
        width: 100%;
        border: 1px solid {table_border_color};
      }}
      .content-table th, .content-table td {{
        border: 1px solid {table_border_color};
        padding: {table_cell_padding}px;
        text-align: left;
      }}
      .content-table th {{
        background-color: {table_header_background};
        font-weight: {table_header_font_weight};
      }}

      /* Lists */
      ul, ol {{
        margin: 0 0 1em 0;
        padding-left: {list_indent}px;
      }}
      li {{
        margin-bottom: {list_item_spacing}px;
        line-height: {text_line_height};
      }}
      li::marker {{
        color: {list_marker_color};
      }}
      /* Nested lists */
      li ul, li ol {{
        margin-top: {list_item_spacing}px;
        margin-bottom: 0;
      }}

      /* Horizontal rule / divider */
      hr {{
        border: none;
        border-top: 1px solid {border_color_default};
        margin: 1.5em 0;
      }}

      /* Figure labels and captions */
      .figure-label {{
        font-weight: bold;
        margin-bottom: 0.5em;
        text-align: center;
      }}
      .figure-caption {{
        font-size: 0.9em;
        color: {text_color_secondary};
        text-align: center;
        margin-top: 0.5em;
      }}
      .figure-caption p {{
        margin: 0;
      }}
"#
    ));
}
