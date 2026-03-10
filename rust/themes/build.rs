//! Generate a builtin theme token registry at build time.
//!
//! Rationale:
//! - production builds should not depend on runtime access to `web/src/themes/base`
//!   files from a repository checkout
//! - production web asset bundles may not include individual `themes/base/*.css`
//!   module files, so runtime discovery via embedded web assets is not reliable
//! - the token list should still derive from implementation CSS, not from a
//!   hand-maintained Rust table or documentation
//!
//! This build script parses the source-of-truth base theme CSS modules in
//! `web/src/themes/base/*.css`, extracts top-level `:root` custom properties,
//! infers scope and family from module filenames, and writes a generated Rust
//! module into `OUT_DIR`. The runtime crate then includes that generated module,
//! making builtin token discovery fast, deterministic, and independent of the
//! runtime environment.

use std::{collections::BTreeMap, env, fmt::Write, fs, path::Path};

use lightningcss::{
    printer::Printer,
    properties::{Property, custom::CustomPropertyName},
    rules::CssRule,
    stylesheet::{ParserOptions, PrinterOptions, StyleSheet},
    traits::ToCss,
};

fn main() {
    let manifest_dir = env::var("CARGO_MANIFEST_DIR").expect("missing CARGO_MANIFEST_DIR");
    let base_dir = Path::new(&manifest_dir).join("../../web/src/themes/base");

    println!("cargo:rerun-if-changed={}", base_dir.display());

    let tokens = build_token_registry(&base_dir);
    let out_dir = env::var("OUT_DIR").expect("missing OUT_DIR");
    let output_path = Path::new(&out_dir).join("builtin_tokens.rs");
    fs::write(&output_path, render_registry(&tokens))
        .expect("failed to write builtin token registry");
}

#[derive(Clone)]
struct TokenRecord {
    name: String,
    scope: &'static str,
    family: &'static str,
    default_value: String,
}

fn build_token_registry(base_dir: &Path) -> Vec<TokenRecord> {
    let mut tokens = Vec::<TokenRecord>::new();

    let entries = fs::read_dir(base_dir).expect("failed to read web/src/themes/base");
    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().and_then(|ext| ext.to_str()) != Some("css") {
            continue;
        }

        let filename = path
            .file_name()
            .and_then(|name| name.to_str())
            .expect("invalid CSS filename");

        let Some((scope, family)) = infer_token_scope_family(filename) else {
            continue;
        };

        let css = fs::read_to_string(&path).expect("failed to read CSS file");
        let mut last_seen = BTreeMap::<String, usize>::new();
        for (name, default_value) in parse_css_variables_in_order(&css) {
            if let Some(index) = last_seen.get(&name).copied() {
                tokens[index].default_value = default_value;
                continue;
            }

            let index = tokens.len();
            tokens.push(TokenRecord {
                name,
                scope,
                family,
                default_value,
            });
            last_seen.insert(tokens[index].name.clone(), index);
        }
    }

    tokens.sort_by(|a, b| {
        a.scope
            .cmp(b.scope)
            .then_with(|| a.family.cmp(b.family))
            .then_with(|| a.name.cmp(&b.name))
    });

    tokens
}

fn parse_css_variables_in_order(css: &str) -> Vec<(String, String)> {
    let mut vars = Vec::new();

    let Ok(sheet) = StyleSheet::parse(css, ParserOptions::default()) else {
        return vars;
    };

    for rule in &sheet.rules.0 {
        if let CssRule::Style(style) = rule {
            let Ok(selector_list) = style.selectors.to_css_string(PrinterOptions::default()) else {
                continue;
            };

            if !selector_list.split(',').any(|s| s.trim() == ":root") {
                continue;
            }

            for (property, _) in style.declarations.iter() {
                if let Property::Custom(custom_prop) = property
                    && let CustomPropertyName::Custom(dashed_ident) = &custom_prop.name
                {
                    let name = dashed_ident.0.as_ref().trim_start_matches("--").to_string();

                    let mut prop_string = String::new();
                    if property
                        .to_css(
                            &mut Printer::new(&mut prop_string, PrinterOptions::default()),
                            false,
                        )
                        .is_ok()
                        && let Some(colon_pos) = prop_string.find(':')
                    {
                        let value = prop_string[colon_pos + 1..].trim().to_string();
                        vars.push((name, value));
                    }
                }
            }
        }
    }

    vars
}

fn infer_token_scope_family(file: &str) -> Option<(&'static str, &'static str)> {
    match file {
        "tokens-primitive.css" => Some(("Semantic", "primitive")),
        "tokens-semantic.css" => Some(("Semantic", "semantic")),
        "pages.css" => Some(("Print", "page")),
        "plots.css" => Some(("Plot", "plot")),
        "layout.css" => Some(("Site", "layout")),
        "breadcrumbs.css" => Some(("Site", "breadcrumbs")),
        "copy-markdown.css" => Some(("Site", "copy-markdown")),
        "copyright.css" => Some(("Site", "copyright")),
        "edit-on.css" => Some(("Site", "edit-on")),
        "edit-source.css" => Some(("Site", "edit-source")),
        "logo.css" => Some(("Site", "logo")),
        "nav-access.css" => Some(("Site", "nav-access")),
        "nav-groups.css" => Some(("Site", "nav-groups")),
        "nav-menu.css" => Some(("Site", "nav-menu")),
        "nav-tree.css" => Some(("Site", "nav-tree")),
        "prev-next.css" => Some(("Site", "prev-next")),
        "site-action.css" => Some(("Site", "site-action")),
        "site-actions.css" => Some(("Site", "site-actions")),
        "site-remote.css" => Some(("Site", "site-remote")),
        "site-review.css" => Some(("Site", "site-review")),
        "site-search.css" => Some(("Site", "site-search")),
        "site-upload.css" => Some(("Site", "site-upload")),
        "social-links.css" => Some(("Site", "social-links")),
        "title.css" => Some(("Site", "title")),
        "toc-tree.css" => Some(("Site", "toc-tree")),
        "admonitions.css" => Some(("Node", "admonition")),
        "breaks.css" => Some(("Node", "break")),
        "captions.css" => Some(("Node", "caption")),
        "citations.css" => Some(("Node", "citation")),
        "code.css" => Some(("Node", "code")),
        "datatables.css" => Some(("Node", "datatable")),
        "diagrams.css" => Some(("Node", "diagram")),
        "figures.css" => Some(("Node", "figure")),
        "headings.css" => Some(("Node", "heading")),
        "images.css" => Some(("Node", "image")),
        "labels.css" => Some(("Node", "label")),
        "links.css" => Some(("Node", "link")),
        "lists.css" => Some(("Node", "list")),
        "math.css" => Some(("Node", "math")),
        "paragraphs.css" => Some(("Node", "paragraph")),
        "quotes.css" => Some(("Node", "quote")),
        "references.css" => Some(("Node", "reference")),
        "styleds.css" => Some(("Node", "styled")),
        "tables.css" => Some(("Node", "table")),
        "works.css" => Some(("Node", "work")),
        "browsers.css" | "color-mode.css" | "icons.css" | "root.css" => {
            Some(("Semantic", "semantic"))
        }
        _ => None,
    }
}

fn render_registry(tokens: &[TokenRecord]) -> String {
    let mut output = String::new();
    output.push_str("// Generated by rust/themes/build.rs. Do not edit by hand.\n");
    output.push_str("vec![\n");

    for token in tokens {
        let _ = writeln!(
            output,
            "    ThemeToken {{ name: {:?}.into(), scope: TokenScope::{}, family: {:?}.into(), default_value: {:?}.into() }},",
            token.name, token.scope, token.family, token.default_value,
        );
    }

    output.push_str("]\n");
    output
}
