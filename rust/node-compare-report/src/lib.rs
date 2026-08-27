//! Human-readable reports over a `stencila-node-compare` comparison
//!
//! Presentation only: nothing here decides what an alignment or a comparison *is*, and
//! no projection, matching or difference logic is duplicated from the comparison crate.
//! It lives outside `stencila-node-compare` because that crate is deliberately free of
//! presentation, and because rendering needs to read one-sided content back out of the
//! source documents, which pulls in the text codec trait.
//!
//! Two renderings are offered, both deterministic human presentation rather than stable
//! interchange formats: use the serialized `Comparison` for that.

use std::{fmt::Write as _, time::Duration};

use eyre::Result;
use similar::{Algorithm, ChangeTag, TextDiff};

use stencila_codec_dom_trait::{DomCodec as _, DomEncodeContext};
use stencila_codec_text_trait::to_text;
use stencila_node_compare::{
    Alignment, Comparison, Correspondence, Difference, DifferenceFilter, NodeRef, PropertyPresence,
    ScalarValue, Side, UnmatchedReason, ValueState,
};
use stencila_node_path::NodePath;
use stencila_node_type::NodeProperty;
use stencila_schema::{Node, NodeSet};

/// One side of a comparison: what was compared, and what to call it
///
/// The reports need both: a label to name the side with, and the node to read one-sided
/// content back out of. The label is deliberately free text rather than a path, because a
/// caller may be comparing nodes that never came from a file.
#[derive(Clone, Copy)]
pub struct Snapshot<'document> {
    /// The node that side was compared as
    pub node: &'document Node,

    /// What to call that side in the report, such as a file path
    pub label: &'document str,
}

/// Render the human-readable report for a comparison
///
/// Deterministic human presentation, not a stable interchange format: use `--to json`
/// or `--to yaml` for that.
pub fn text_report(
    comparison: &Comparison,
    left: Snapshot,
    right: Snapshot,
    summary: bool,
) -> Result<String> {
    let differences = comparison.differences();
    let one_sided =
        OneSidedRoots::collect(comparison.alignment(), comparison.filter(), left, right);
    let counts = Counts::collect(comparison);

    let mut report = String::new();

    writeln!(
        report,
        "{}",
        if comparison.is_equal() {
            "equal"
        } else {
            "different"
        }
    )?;
    writeln!(report, "left:  {}", left.label)?;
    writeln!(report, "right: {}", right.label)?;
    writeln!(report)?;

    writeln!(
        report,
        "correspondences: {paired} paired, {left_only} left-only ({left_roots}), {right_only} right-only ({right_roots})",
        paired = counts.paired,
        left_only = counts.left_only,
        left_roots = plural(one_sided.left.len(), "root"),
        right_only = counts.right_only,
        right_roots = plural(one_sided.right.len(), "root"),
    )?;
    writeln!(report, "differences: {}", differences.len())?;
    writeln!(
        report,
        "  node type: {}  presence: {}  value: {}  parent: {}  reordered: {}",
        counts.node_type, counts.presence, counts.value, counts.parent, counts.reordered
    )?;
    if let Some((selectors, suppressed)) = filter_description(comparison) {
        writeln!(report, "filter:     {selectors}")?;
        writeln!(report, "suppressed: {suppressed}")?;
    }

    if summary
        || (one_sided.left.is_empty() && one_sided.right.is_empty() && differences.is_empty())
    {
        return Ok(report);
    }

    writeln!(report)?;

    for root in one_sided.left.iter().chain(one_sided.right.iter()) {
        write_one_sided(&mut report, root)?;
    }
    for difference in differences {
        write_difference(&mut report, difference)?;
    }

    Ok(report)
}

/// Render the HTML view for a comparison
///
/// This entry point preserves the differences-only report used by existing callers.
/// Use [`html_report_with_overlay`] to include a merged-document overlay.
pub fn html_report(
    comparison: &Comparison,
    left: Snapshot,
    right: Snapshot,
    summary: bool,
) -> Result<String> {
    html_report_inner(comparison, left, right, summary, None)
}

/// Render the HTML view for a comparison with a merged-document overlay
pub fn html_report_with_overlay(
    comparison: &Comparison,
    left: Snapshot,
    right: Snapshot,
    summary: bool,
    overlay: &Node,
) -> Result<String> {
    html_report_inner(comparison, left, right, summary, Some(overlay))
}

/// Render the HTML view for a comparison
///
/// Two readings of the same comparison, as tabs of one page. The differences tab lists
/// what changed, occurrence by occurrence; the overlay tab shows the left document
/// with the right one's changes marked up in place, which is what makes a change
/// legible in the prose around it.
///
/// The overlay is an ordinary Stencila document carrying `Suggestion` and `Comment`
/// nodes, built by `stencila-node-merge` and passed in rather than derived here, so
/// that this crate keeps deciding only how a comparison is *presented*. Without one,
/// the page has the differences tab alone.
///
/// A self-contained page, with no external assets and no scripts, so that it can be
/// written to a temporary file and opened directly by a browser. The tabs are CSS
/// only for that reason. Like the text report, this is human presentation rather than
/// an interchange format.
fn html_report_inner(
    comparison: &Comparison,
    left: Snapshot,
    right: Snapshot,
    summary: bool,
    overlay: Option<&Node>,
) -> Result<String> {
    let alignment = comparison.alignment();
    let differences = comparison.differences();
    let one_sided = OneSidedRoots::collect(alignment, comparison.filter(), left, right);
    let counts = Counts::collect(comparison);

    let (status, status_class) = if comparison.is_equal() {
        ("equal", "equal")
    } else {
        ("different", "different")
    };

    // Rows are read down the page, so they are put into the reading order of the
    // documents rather than grouped by kind
    let anchors = LeftAnchors::collect(alignment);
    let mut rows = Vec::new();
    for root in one_sided.left.iter().chain(one_sided.right.iter()) {
        rows.push(ViewRow::one_sided(root, &anchors));
    }
    for difference in differences {
        rows.push(ViewRow::difference(difference));
    }
    // A stable sort, so that several rows about the same occurrence keep the
    // canonical order that the comparison put them in
    rows.sort_by(|first, second| first.order.cmp(&second.order));

    let mut html = String::new();

    writeln!(
        html,
        r#"<!DOCTYPE html>
<html lang="en">
<head>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width, initial-scale=1">
<title>Comparison of {left_title} and {right_title}</title>
<style>{CSS}</style>
</head>
<body>
<header>
  <h1>Comparison <span class="status {status_class}">{status}</span></h1>
  <div class="paths">
    <div class="side"><span class="label">left</span> <span class="path">{left_path}</span></div>
    <div class="side"><span class="label">right</span> <span class="path">{right_path}</span></div>
  </div>
  <ul class="counts">
    <li><span>paired</span> {paired}</li>
    <li><span>left-only</span> {left_only} <em>({left_roots})</em></li>
    <li><span>right-only</span> {right_only} <em>({right_roots})</em></li>
    <li><span>differences</span> {differences_count}</li>
    <li><span>node type</span> {node_type}</li>
    <li><span>presence</span> {presence}</li>
    <li><span>value</span> {value}</li>
    <li><span>parent</span> {parent}</li>
    <li><span>reordered</span> {reordered}</li>
  </ul>
  {filter}
</header>"#,
        left_title = escape(left.label),
        right_title = escape(right.label),
        left_path = escape(left.label),
        right_path = escape(right.label),
        paired = counts.paired,
        left_only = counts.left_only,
        left_roots = escape(&plural(one_sided.left.len(), "root")),
        right_only = counts.right_only,
        right_roots = escape(&plural(one_sided.right.len(), "root")),
        differences_count = differences.len(),
        node_type = counts.node_type,
        presence = counts.presence,
        value = counts.value,
        parent = counts.parent,
        reordered = counts.reordered,
        filter = match filter_description(comparison) {
            Some((selectors, suppressed)) => format!(
                "<div class=\"filter\"><span class=\"label\">filter</span> \
                 <code>{selectors}</code> <em>suppressed {suppressed}</em></div>",
                selectors = escape(&selectors),
                suppressed = escape(&suppressed),
            ),
            None => String::new(),
        },
    )?;

    // The radio inputs precede both the labels and the panes, so that a `:checked`
    // sibling selector can reach either
    writeln!(
        html,
        r#"<div class="tabs">
<input type="radio" name="tab" id="tab-differences" checked>
<input type="radio" name="tab" id="tab-overlay"{overlay_disabled}>
<div class="tablist">
<label for="tab-differences">Differences</label>
<label for="tab-overlay"{overlay_label_class}>Overlay</label>
</div>
<section class="pane differences">"#,
        overlay_disabled = match overlay {
            Some(..) => "",
            None => " disabled",
        },
        overlay_label_class = match overlay {
            Some(..) => "",
            None => r#" class="unavailable""#,
        },
    )?;

    if summary || rows.is_empty() {
        let note = if rows.is_empty() {
            "The documents are equal."
        } else {
            "Only counts are reported because <code>--summary</code> was used."
        };
        writeln!(html, "<p class=\"note\">{note}</p>")?;
    } else {
        writeln!(
            html,
            r#"<table>
<thead><tr><th class="kind">&nbsp;</th><th>left</th><th>right</th></tr></thead>
<tbody>"#
        )?;
        for row in &rows {
            write_view_row(&mut html, row)?;
        }
        writeln!(html, "</tbody>\n</table>")?;
    }

    writeln!(html, "</section>\n<section class=\"pane overlay\">")?;

    match overlay {
        Some(node) => writeln!(html, "{}", overlay_html(node))?,
        None => writeln!(
            html,
            "<p class=\"note\">No overlay was built for this comparison.</p>"
        )?,
    }

    writeln!(html, "</section>\n</div>")?;

    writeln!(html, "</body>\n</html>")?;

    Ok(html)
}

/// Render the overlay document as HTML
///
/// Uses the DOM encoding, which is the canonical HTML form of a Stencila document, but
/// only its markup: the web bundle that would otherwise animate it is deliberately not
/// referenced, so the page stays self-contained and works from a temporary file. The
/// custom elements it emits carry real semantic HTML inside them — a paragraph is a
/// `<p slot="content">` within a `<stencila-paragraph>` — so the styles below have
/// only to give the wrappers a layout and the suggestions their colours.
///
/// The static view is asked for because the merged document is read, not edited: it
/// leaves out the node identifiers that only matter to a live view.
fn overlay_html(node: &Node) -> String {
    let mut context = DomEncodeContext::new(Some("static"), Some(false));
    node.to_dom(&mut context);
    let mut html = context.content();

    html.push_str(&annotations_html(node));

    html
}

/// Render the comments of the overlay document onto the nodes they are about
///
/// A comment names its subject by identifier, and the DOM encoding writes that
/// identifier onto the element as `_id`, so the two can be joined with an attribute
/// selector. That is enough to mark the node and to show what the comment says on
/// hovering it, without a script and without touching the markup the encoding
/// produced.
///
/// A list of every comment underneath the document would be the same information, but
/// it is not the same thing to read: a note that says a heading's level changed is
/// worth having *at the heading*, and worth nothing three hundred entries down a page.
/// Only the comments with nothing to attach to are listed.
fn annotations_html(node: &Node) -> String {
    let Node::Article(article) = node else {
        return String::new();
    };

    let Some(comments) = &article.options.comments else {
        return String::new();
    };

    let mut anchored: Vec<(String, String)> = Vec::new();
    let mut unanchored: Vec<String> = Vec::new();

    for comment in comments {
        let message = to_text(&comment.content).trim().to_string();
        match comment
            .options
            .start_location
            .as_deref()
            .and_then(|location| location.strip_prefix('#'))
        {
            Some(id) if !id.is_empty() => anchored.push((id.to_string(), message)),
            _ => unanchored.push(message),
        }
    }

    let mut html = String::new();
    html.push_str(&anchored_styles(&anchored));
    html.push_str(&unanchored_html(&unanchored));

    html
}

/// The styles that mark the commented nodes and reveal what the comments say
///
/// Generated rather than static because each rule has to name the identifiers it
/// applies to. The marking is one rule for every commented node at once; the text is
/// one rule per distinct message, since the same observation is usually made about
/// many nodes in a document and repeating it once per node would be most of the page.
fn anchored_styles(anchored: &[(String, String)]) -> String {
    if anchored.is_empty() {
        return String::new();
    }

    let selector = |ids: &[&str], suffix: &str| -> String {
        ids.iter()
            .map(|id| format!(".pane.overlay [_id=\"{}\"]{suffix}", css_escape(id)))
            .collect::<Vec<_>>()
            .join(",\n")
    };

    let all: Vec<&str> = anchored.iter().map(|(id, ..)| id.as_str()).collect();

    // A rule down the left rather than a fill. A comment is very often about a whole
    // section — its identifier changed, it moved — and filling one floods the page with
    // colour to say something about the container rather than about anything in it. A
    // marker reads the same on a section and on a word, and leaves the text legible
    // underneath, which is what the overlay is for.
    let mut css = String::from("\n<style>\n");
    let _ = write!(
        css,
        "{}{{\n  box-shadow: inset 3px 0 0 var(--comment);\n  \
         position: relative;\n  cursor: help;\n}}\n",
        selector(&all, "")
    );

    // Grouped by message, in first-seen order so the output is stable
    let mut messages: Vec<&str> = Vec::new();
    let mut grouped: Vec<Vec<&str>> = Vec::new();
    for (id, message) in anchored {
        match messages.iter().position(|seen| *seen == message) {
            Some(index) => grouped[index].push(id),
            None => {
                messages.push(message);
                grouped.push(vec![id]);
            }
        }
    }

    for (message, ids) in messages.iter().zip(grouped.iter()) {
        let _ = write!(
            css,
            "{}{{\n  content: \"{}\";\n}}\n",
            selector(ids, ":hover::after"),
            css_escape(message)
        );
    }

    css.push_str("</style>\n");

    css
}

/// The comments that have nothing in the document to attach to
///
/// A change under `authors` or `references` has no block or inline ancestor to mark, so
/// there is nowhere to put it but a list.
fn unanchored_html(unanchored: &[String]) -> String {
    if unanchored.is_empty() {
        return String::new();
    }

    let mut html =
        String::from(r#"<section class="annotations"><h2>Elsewhere in the document</h2><ol>"#);
    for message in unanchored {
        let _ = write!(html, "<li>{}</li>", escape(message));
    }
    html.push_str("</ol></section>");

    html
}

/// Escape a string for use inside a CSS string or attribute selector value
fn css_escape(text: &str) -> String {
    let mut escaped = String::with_capacity(text.len());
    for character in text.chars() {
        match character {
            '\\' => escaped.push_str("\\\\"),
            '"' => escaped.push_str("\\\""),
            // A literal `<` can terminate the surrounding HTML `style` element even
            // when it occurs inside a valid CSS string. Use a CSS hexadecimal escape
            // so the browser displays the character without seeing HTML markup.
            '<' => escaped.push_str("\\3c "),
            '\n' | '\r' => escaped.push(' '),
            _ => escaped.push(character),
        }
    }
    escaped
}

/// The styles for the HTML view
const CSS: &str = r#"
:root {
  --background: #ffffff;
  --foreground: #16181d;
  --muted: #5b6472;
  --border: #dfe3e9;
  --surface: #f6f7f9;
  --left: #b4442e;
  --right: #1f7a4d;
  --changed: #8a5a00;
  --removed-background: #ffd7d1;
  --removed-foreground: #7a2617;
  --added-background: #c8f0d8;
  --added-foreground: #10502f;
  --removed-tint: #fdeeec;
  --added-tint: #edfaf1;
  --comment: #c26a00;
  --comment-background: #ffe9cc;
  --comment-foreground: #4a2a00;
}
@media (prefers-color-scheme: dark) {
  :root {
    --background: #14161a;
    --foreground: #e7e9ee;
    --muted: #98a1b0;
    --border: #2b2f37;
    --surface: #1b1e24;
    --left: #f08f78;
    --right: #74d3a2;
    --changed: #e0b355;
    --removed-background: #5e2018;
    --removed-foreground: #ffd7d1;
    --added-background: #14472e;
    --added-foreground: #c8f0d8;
    --removed-tint: #2a1512;
    --added-tint: #12261b;
    --comment: #e08a2e;
    --comment-background: #4a2f10;
    --comment-foreground: #ffe9cc;
  }
}
* { box-sizing: border-box; }
body {
  margin: 0;
  padding: 1.5rem;
  background: var(--background);
  color: var(--foreground);
  font-family: system-ui, -apple-system, "Segoe UI", sans-serif;
  font-size: 15px;
  line-height: 1.5;
}
h1 { font-size: 1.25rem; margin: 0 0 0.75rem; }
.status {
  font-size: 0.75rem;
  text-transform: uppercase;
  letter-spacing: 0.06em;
  padding: 0.15rem 0.5rem;
  border-radius: 999px;
  border: 1px solid var(--border);
  vertical-align: middle;
}
.status.equal { color: var(--right); }
.status.different { color: var(--changed); }
.paths { display: flex; flex-wrap: wrap; gap: 1.5rem; margin-bottom: 0.75rem; }
.side .label {
  font-size: 0.75rem;
  text-transform: uppercase;
  letter-spacing: 0.06em;
  color: var(--muted);
}
.path, code, .subject, .detail { font-family: ui-monospace, SFMono-Regular, Menlo, monospace; }
.counts { display: flex; flex-wrap: wrap; gap: 0.5rem; list-style: none; margin: 0; padding: 0; }
.counts li {
  background: var(--surface);
  border: 1px solid var(--border);
  border-radius: 0.375rem;
  padding: 0.15rem 0.5rem;
  font-size: 0.85rem;
}
.counts span { color: var(--muted); margin-right: 0.35rem; }
.counts em { color: var(--muted); font-style: normal; }
.note { color: var(--muted); margin-top: 1.25rem; }
.filter { margin-top: 0.5rem; font-size: 0.85rem; }
.filter em { color: var(--muted); font-style: normal; }
table { width: 100%; border-collapse: collapse; margin-top: 1.25rem; table-layout: fixed; }
th, td {
  text-align: left;
  vertical-align: top;
  padding: 0.5rem 0.65rem;
  border-bottom: 1px solid var(--border);
  overflow-wrap: anywhere;
}
thead th {
  font-size: 0.75rem;
  text-transform: uppercase;
  letter-spacing: 0.06em;
  color: var(--muted);
  font-weight: 600;
}
th.kind, td.kind { width: 9.5rem; }
.marker { margin-right: 0.4rem; font-family: ui-monospace, SFMono-Regular, Menlo, monospace; }
/* Tabs, driven by the checked state of two radio inputs so that the page needs no
   script and stays self-contained. The inputs stay focusable, and so keyboard
   reachable, rather than being hidden with `display: none`. */
.tabs > input[type="radio"] {
  position: absolute;
  opacity: 0;
  width: 0;
  height: 0;
  pointer-events: none;
}
.tablist {
  display: flex;
  gap: 0.25rem;
  margin-top: 1.25rem;
  border-bottom: 1px solid var(--border);
}
.tablist label {
  padding: 0.4rem 0.8rem;
  cursor: pointer;
  color: var(--muted);
  border-bottom: 2px solid transparent;
  margin-bottom: -1px;
  font-size: 0.9rem;
}
.tablist label.unavailable { cursor: default; opacity: 0.5; }
#tab-differences:checked ~ .tablist label[for="tab-differences"],
#tab-overlay:checked ~ .tablist label[for="tab-overlay"] {
  color: var(--foreground);
  border-bottom-color: var(--foreground);
}
#tab-differences:focus-visible ~ .tablist label[for="tab-differences"],
#tab-overlay:focus-visible ~ .tablist label[for="tab-overlay"] {
  outline: 2px solid var(--changed);
  outline-offset: -2px;
}
.pane { display: none; }
#tab-differences:checked ~ .pane.differences,
#tab-overlay:checked ~ .pane.overlay { display: block; }
.pane table { margin-top: 0.75rem; }

/* The overlay: the DOM encoding of the merged document. Its custom elements are
   inline by default, so the ones that stand for a block need a layout given to them. */
.pane.overlay {
  max-width: 46rem;
  margin-top: 1.5rem;
  line-height: 1.6;
}
.pane.overlay stencila-article,
.pane.overlay stencila-section,
.pane.overlay stencila-paragraph,
.pane.overlay stencila-heading,
.pane.overlay stencila-list,
.pane.overlay stencila-list-item,
.pane.overlay stencila-table,
.pane.overlay stencila-figure,
.pane.overlay stencila-quote-block,
.pane.overlay stencila-code-block,
.pane.overlay stencila-code-chunk,
.pane.overlay stencila-math-block,
.pane.overlay stencila-admonition,
.pane.overlay stencila-thematic-break,
.pane.overlay stencila-suggestion-block { display: block; }
/* A guard, in case the DOM encoding ever does emit comments: they are rendered as
   notes below instead, and showing both would say everything twice. */
.pane.overlay [slot="comments"] { display: none; }
/* A commented node is marked in the document rather than footnoted below it, and says
   what the comment says on hover. The rules that name which nodes those are, and what
   each says, are generated per document and emitted with the overlay. */
.pane.overlay [_id]:hover::after {
  position: absolute;
  left: 0;
  top: calc(100% + 0.25rem);
  z-index: 10;
  width: max-content;
  max-width: 28rem;
  padding: 0.4rem 0.6rem;
  border-radius: 0.375rem;
  border: 1px solid var(--comment);
  background: var(--comment-background);
  color: var(--comment-foreground);
  font-family: system-ui, -apple-system, "Segoe UI", sans-serif;
  font-size: 0.8rem;
  font-weight: 400;
  font-style: normal;
  line-height: 1.4;
  text-align: left;
  text-decoration: none;
  white-space: normal;
  pointer-events: none;
}
.annotations {
  margin-top: 2rem;
  padding-top: 1rem;
  border-top: 1px solid var(--border);
}
.annotations h2 {
  font-size: 0.75rem;
  text-transform: uppercase;
  letter-spacing: 0.06em;
  color: var(--muted);
  font-weight: 600;
  margin: 0 0 0.5rem;
}
.annotations ol { margin: 0; padding-left: 1.25rem; }
.annotations li { margin-bottom: 0.35rem; font-size: 0.9rem; }
.annotations code { color: var(--muted); font-size: 0.85em; }
.pane.overlay pre { overflow-x: auto; }
.pane.overlay table { table-layout: auto; }

/* Suggestions. The DOM encoding writes the enum variant verbatim, so the attribute
   values are the schema's own names.

   Inline and block suggestions are marked differently on purpose. A few changed words
   read best filled in, the way a proofreader would strike and insert them. A changed
   *region* does not: filling several paragraphs solid states loudly and repeatedly
   what one mark in the margin states once, and buries the text it is supposed to be
   showing. Blocks therefore get a margin rule and a wash faint enough to read
   through. */
.pane.overlay stencila-suggestion-inline { border-radius: 0.2rem; }
.pane.overlay stencila-suggestion-inline[suggestion-type="Delete"] [slot="content"],
.pane.overlay stencila-suggestion-inline[suggestion-type="Replace"] [slot="original"] {
  text-decoration: line-through;
  background: var(--removed-background);
  color: var(--removed-foreground);
}
.pane.overlay stencila-suggestion-inline[suggestion-type="Insert"] [slot="content"],
.pane.overlay stencila-suggestion-inline[suggestion-type="Replace"] [slot="content"] {
  background: var(--added-background);
  color: var(--added-foreground);
}

/* An inline replacement reads as an edit rather than as two alternatives when the
   words being replaced come first, which is not the order the encoding writes them */
.pane.overlay stencila-suggestion-inline[suggestion-type="Replace"] {
  display: inline-flex;
  flex-wrap: wrap;
  gap: 0.15rem;
  vertical-align: baseline;
}
.pane.overlay stencila-suggestion-inline[suggestion-type="Replace"] [slot="original"] {
  order: 0;
}
.pane.overlay stencila-suggestion-inline[suggestion-type="Replace"] [slot="content"] {
  order: 1;
}

.pane.overlay stencila-suggestion-block > [slot] {
  display: block;
  margin: 0.35rem 0;
  padding: 0.15rem 0 0.15rem 0.75rem;
}
.pane.overlay stencila-suggestion-block[suggestion-type="Delete"] > [slot="content"],
.pane.overlay stencila-suggestion-block[suggestion-type="Replace"] > [slot="original"] {
  border-left: 3px solid var(--left);
  background: var(--removed-tint);
}
.pane.overlay stencila-suggestion-block[suggestion-type="Insert"] > [slot="content"],
.pane.overlay stencila-suggestion-block[suggestion-type="Replace"] > [slot="content"] {
  border-left: 3px solid var(--right);
  background: var(--added-tint);
}
.pane.overlay stencila-suggestion-block[suggestion-type="Delete"] > [slot="content"] {
  text-decoration: line-through;
  text-decoration-color: var(--left);
}

tr.left-only .marker, tr.left-only .kind-label { color: var(--left); }
tr.right-only .marker, tr.right-only .kind-label { color: var(--right); }
tr.changed .marker, tr.changed .kind-label { color: var(--changed); }
.kind-label { font-size: 0.85rem; }
.note-inline { display: block; color: var(--muted); font-size: 0.8rem; }
.detail { display: block; color: var(--muted); font-size: 0.85rem; margin-top: 0.15rem; }
.detail .removed, .detail .added { border-radius: 0.15rem; padding: 0.05rem 0.1rem; }
.detail .removed { background: var(--removed-background); color: var(--removed-foreground); }
.detail .added { background: var(--added-background); color: var(--added-foreground); }
.absent { color: var(--muted); }
"#;

/// Where a row belongs in the reading order of the view
///
/// Correspondences and differences are both in canonical order, which is left path
/// order, so rows that have a left occurrence order themselves. Right-only rows have
/// no left path at all, which is why the comparison groups them ahead of everything
/// else, so they are instead anchored to the left path of the paired occurrence that
/// precedes them on the right (see `LeftAnchors`).
#[derive(PartialEq, Eq, PartialOrd, Ord)]
struct RowOrder {
    /// The left path the row sits at, or is anchored to
    anchor: Option<NodePath>,

    /// Whether the row belongs at that left path, or just after it
    ///
    /// A right-only row goes after the occurrence it is anchored to, but before that
    /// occurrence's descendants, which is where it was inserted on the right.
    after_anchor: bool,

    /// The right path, which orders right-only rows sharing an anchor
    right: Option<NodePath>,
}

/// The left path that each right path belongs after
///
/// Built from the paired occurrences, which are the only ones whose position is known
/// on both sides.
struct LeftAnchors<'comparison> {
    /// The paired occurrences as (right path, left path), ordered by right path
    pairs: Vec<(&'comparison NodePath, &'comparison NodePath)>,
}

impl<'comparison> LeftAnchors<'comparison> {
    fn collect(alignment: &'comparison Alignment) -> Self {
        let mut pairs: Vec<_> = alignment
            .pairs()
            .map(|(left, right, ..)| (&right.path, &left.path))
            .collect();
        pairs.sort();

        Self { pairs }
    }

    /// The left path that a right path is positioned after, if any
    ///
    /// The nearest paired occurrence preceding it on the right, which is `None` when
    /// nothing on the right precedes it, and the row belongs at the top.
    fn anchor(&self, right: &NodePath) -> Option<NodePath> {
        let following = self.pairs.partition_point(|(path, ..)| *path < right);
        following
            .checked_sub(1)
            .map(|preceding| self.pairs[preceding].1.clone())
    }
}

/// One side of a row of the side-by-side view
struct ViewCell {
    /// What the row is about on this side, usually a path
    subject: String,

    /// The state of that subject on this side, in runs to be shaded
    ///
    /// Empty when the row has no state to show, and a single unshaded run when it
    /// has one that is not worth comparing within.
    detail: Vec<Segment>,
}

impl ViewCell {
    /// A cell whose state is shown as it is, without shading
    fn new(subject: String, detail: Option<String>) -> Self {
        Self {
            subject,
            detail: detail.into_iter().map(Segment::unchanged).collect(),
        }
    }
}

/// A run of a rendered state, and whether the other side also has it
struct Segment {
    text: String,

    /// Whether this run is missing from the other side, and so is shaded
    changed: bool,
}

impl Segment {
    fn unchanged(text: String) -> Self {
        Self {
            text,
            changed: false,
        }
    }
}

/// The longest rendered states that are compared within
///
/// Beyond this the shading is more noise than signal, and the comparison is not worth
/// the time, so the states are shown plainly instead.
const MAX_SHADED_CHARACTERS: usize = 10_000;

/// How long to spend comparing within a pair of rendered states
const SHADING_TIMEOUT: Duration = Duration::from_millis(250);

/// Split two rendered states into runs, marking what the other side does not have
///
/// Compares by Unicode word, rather than by character, so that shading falls on whole
/// words instead of fragmenting inside them. The rendered states are compared, rather
/// than the raw strings, so that the type prefix and quoting stay in place and remain
/// unshaded when only the value within them changed.
fn shade(left: &str, right: &str) -> (Vec<Segment>, Vec<Segment>) {
    if left.len() > MAX_SHADED_CHARACTERS || right.len() > MAX_SHADED_CHARACTERS {
        return (
            vec![Segment::unchanged(left.to_string())],
            vec![Segment::unchanged(right.to_string())],
        );
    }

    let diff = TextDiff::configure()
        .algorithm(Algorithm::Patience)
        .timeout(SHADING_TIMEOUT)
        .diff_unicode_words(left, right);

    let mut left_segments: Vec<Segment> = Vec::new();
    let mut right_segments: Vec<Segment> = Vec::new();

    for change in diff.iter_all_changes() {
        let text = change.value();
        match change.tag() {
            ChangeTag::Equal => {
                push_segment(&mut left_segments, text, false);
                push_segment(&mut right_segments, text, false);
            }
            ChangeTag::Delete => push_segment(&mut left_segments, text, true),
            ChangeTag::Insert => push_segment(&mut right_segments, text, true),
        }
    }

    (
        merge_across_whitespace(left_segments),
        merge_across_whitespace(right_segments),
    )
}

/// Absorb the space between two changed words into the shading
///
/// Comparing by word reports the space between two changed words as equal to the
/// other side, which would otherwise break what reads as one change into two shaded
/// runs. Trailing space before unchanged text is left alone, because runs of the same
/// kind are already merged, so a whitespace-only run is only ever between two changes.
fn merge_across_whitespace(segments: Vec<Segment>) -> Vec<Segment> {
    let count = segments.len();
    let mut merged: Vec<Segment> = Vec::new();

    for (index, segment) in segments.into_iter().enumerate() {
        let bridges = !segment.changed
            && index + 1 < count
            && segment.text.chars().all(char::is_whitespace)
            && merged.last().is_some_and(|last| last.changed);

        push_segment(&mut merged, &segment.text, segment.changed || bridges);
    }

    merged
}

/// Add a run to a side, merging it into the previous run when they are alike
///
/// Merging keeps shading contiguous across adjacent changed words, rather than
/// breaking it at every word boundary.
fn push_segment(segments: &mut Vec<Segment>, text: &str, changed: bool) {
    match segments.last_mut() {
        Some(last) if last.changed == changed => last.text.push_str(text),
        _ => segments.push(Segment {
            text: text.to_string(),
            changed,
        }),
    }
}

/// One row of the side-by-side view
struct ViewRow {
    /// The kind of correspondence or difference
    kind: &'static str,

    /// The same marker character that the text report uses
    marker: &'static str,

    /// Which of the three row colors to use
    class: &'static str,

    /// The left side, or `None` when the row is right-only
    left: Option<ViewCell>,

    /// The right side, or `None` when the row is left-only
    right: Option<ViewCell>,

    /// Additional explanation of the row, such as why a subtree is one-sided
    note: Option<String>,

    /// Where the row belongs in the reading order of the view
    order: RowOrder,
}

impl ViewRow {
    /// Build a row for a one-sided subtree root
    fn one_sided(root: &OneSidedRoot, anchors: &LeftAnchors) -> Self {
        // The whole subtree is on one side, so all of its content is shaded, the same
        // way a value that only one side has is
        let cell = ViewCell {
            subject: occurrence(root.node),
            detail: root
                .content
                .iter()
                .map(|content| Segment {
                    text: content.clone(),
                    changed: true,
                })
                .collect(),
        };

        let reason = match root.reason {
            UnmatchedReason::NoCompatibleCandidate => "no compatible candidate",
            UnmatchedReason::GapCheaperThanPair => "gap cheaper than pair",
        };
        let note = Some(if root.occurrences > 1 {
            format!("{} occurrences; {reason}", root.occurrences)
        } else {
            reason.to_string()
        });

        match root.side {
            Side::Left => Self {
                kind: "left-only",
                marker: "-",
                class: "left-only",
                left: Some(cell),
                right: None,
                note,
                order: RowOrder {
                    anchor: Some(root.node.path.clone()),
                    after_anchor: false,
                    right: None,
                },
            },
            Side::Right => Self {
                kind: "right-only",
                marker: "+",
                class: "right-only",
                left: None,
                right: Some(cell),
                note,
                order: RowOrder {
                    anchor: anchors.anchor(&root.node.path),
                    after_anchor: true,
                    right: Some(root.node.path.clone()),
                },
            },
        }
    }

    /// Build a row for a difference
    fn difference(difference: &Difference) -> Self {
        let (kind, marker, left, right) = match difference {
            Difference::NodeTypeChanged { left, right } => (
                "node type",
                "≠",
                ViewCell::new(occurrence(left), Some(left.node_type.to_string())),
                ViewCell::new(occurrence(right), Some(right.node_type.to_string())),
            ),

            Difference::PropertyPresenceChanged {
                left,
                right,
                property,
                left_presence,
                right_presence,
            } => (
                "presence",
                "±",
                ViewCell::new(
                    value_subject(left, Some(property), None),
                    Some(presence(*left_presence).to_string()),
                ),
                ViewCell::new(
                    value_subject(right, Some(property), None),
                    Some(presence(*right_presence).to_string()),
                ),
            ),

            Difference::ValueChanged {
                location,
                left,
                right,
            } => {
                // The only difference where both sides carry content that is worth
                // comparing within, rather than a single term
                let (left_detail, right_detail) = shade(&value_state(left), &value_state(right));
                (
                    "value",
                    "~",
                    ViewCell {
                        subject: value_subject(
                            &location.left,
                            location.property.as_ref(),
                            location.left_index,
                        ),
                        detail: left_detail,
                    },
                    ViewCell {
                        subject: value_subject(
                            &location.right,
                            location.property.as_ref(),
                            location.right_index,
                        ),
                        detail: right_detail,
                    },
                )
            }

            Difference::ParentChanged {
                left,
                right,
                left_parent,
                right_parent,
                left_property,
                right_property,
            } => (
                "parent",
                "→",
                ViewCell::new(
                    occurrence(left),
                    Some(parent(left_parent.as_ref(), left_property.as_ref())),
                ),
                ViewCell::new(
                    occurrence(right),
                    Some(parent(right_parent.as_ref(), right_property.as_ref())),
                ),
            ),

            Difference::Reordered { left, right, .. } => (
                "reordered",
                "↕",
                ViewCell::new(occurrence(left), None),
                ViewCell::new(occurrence(right), None),
            ),
        };

        Self {
            kind,
            marker,
            class: "changed",
            left: Some(left),
            right: Some(right),
            note: None,
            order: RowOrder {
                anchor: Some(difference.left().path.clone()),
                after_anchor: false,
                right: Some(difference.right().path.clone()),
            },
        }
    }
}

/// Write a row of the side-by-side view
fn write_view_row(html: &mut String, row: &ViewRow) -> Result<()> {
    writeln!(
        html,
        r#"<tr class="{class}">
  <td class="kind"><span class="marker">{marker}</span><span class="kind-label">{kind}</span></td>
  {left}
  {right}
</tr>"#,
        class = row.class,
        marker = escape(row.marker),
        kind = escape(row.kind),
        left = view_cell(row.left.as_ref(), row.note.as_deref(), "removed"),
        right = view_cell(row.right.as_ref(), row.note.as_deref(), "added"),
    )?;

    Ok(())
}

/// Render one side of a row of the side-by-side view
///
/// `shading` is the class for the runs that the other side does not have, which is
/// what makes the same run read as removed on the left and added on the right.
fn view_cell(cell: Option<&ViewCell>, note: Option<&str>, shading: &str) -> String {
    let Some(cell) = cell else {
        return "<td class=\"absent\">—</td>".to_string();
    };

    let mut rendered = format!(
        "<td><span class=\"subject\">{}</span>",
        escape(&cell.subject)
    );
    if !cell.detail.is_empty() {
        rendered.push_str("<span class=\"detail\">");
        for segment in &cell.detail {
            let text = escape(&segment.text);
            if segment.changed {
                rendered.push_str(&format!("<span class=\"{shading}\">{text}</span>"));
            } else {
                rendered.push_str(&text);
            }
        }
        rendered.push_str("</span>");
    }
    if let Some(note) = note {
        rendered.push_str(&format!(
            "<span class=\"note-inline\">{}</span>",
            escape(note)
        ));
    }
    rendered.push_str("</td>");

    rendered
}

/// Escape text for inclusion in HTML
fn escape(text: &str) -> String {
    let mut escaped = String::with_capacity(text.len());
    for character in text.chars() {
        match character {
            '&' => escaped.push_str("&amp;"),
            '<' => escaped.push_str("&lt;"),
            '>' => escaped.push_str("&gt;"),
            '"' => escaped.push_str("&quot;"),
            '\'' => escaped.push_str("&#39;"),
            _ => escaped.push(character),
        }
    }
    escaped
}

/// How many correspondences and differences of each kind there are
#[derive(Default)]
struct Counts {
    paired: usize,
    left_only: usize,
    right_only: usize,
    node_type: usize,
    presence: usize,
    value: usize,
    parent: usize,
    reordered: usize,
}

impl Counts {
    /// Count what the comparison reports
    ///
    /// One-sided counts come from the comparison's tally rather than from the raw
    /// correspondences, so that a filtered subtree is counted out whole rather than
    /// leaving its descendants behind.
    fn collect(comparison: &Comparison) -> Self {
        let mut counts = Self::default();

        for correspondence in comparison.alignment().correspondences() {
            if matches!(correspondence, Correspondence::Paired { .. }) {
                counts.paired += 1;
            }
        }

        let tally = comparison.one_sided_tally();
        counts.left_only = tally.left_only();
        counts.right_only = tally.right_only();

        for difference in comparison.differences() {
            match difference {
                Difference::NodeTypeChanged { .. } => counts.node_type += 1,
                Difference::PropertyPresenceChanged { .. } => counts.presence += 1,
                Difference::ValueChanged { .. } => counts.value += 1,
                Difference::ParentChanged { .. } => counts.parent += 1,
                Difference::Reordered { .. } => counts.reordered += 1,
            }
        }

        counts
    }
}

/// The root of a maximal one-sided subtree, and how many occurrences it covers
struct OneSidedRoot<'comparison> {
    side: Side,
    node: &'comparison NodeRef,
    reason: UnmatchedReason,
    occurrences: usize,

    /// What the subtree says, read back out of the document it is only in
    content: Option<String>,
}

/// The maximal one-sided subtree roots of each side
#[derive(Default)]
struct OneSidedRoots<'comparison> {
    left: Vec<OneSidedRoot<'comparison>>,
    right: Vec<OneSidedRoot<'comparison>>,
}

impl<'comparison> OneSidedRoots<'comparison> {
    /// Collapse the one-sided correspondences onto their maximal roots
    ///
    /// Every structured descendant of a one-sided occurrence is itself one-sided, and
    /// correspondences are in canonical path order, so on each side the descendants
    /// of a root immediately follow it.
    fn collect(
        alignment: &'comparison Alignment,
        filter: &DifferenceFilter,
        left_snapshot: Snapshot,
        right_snapshot: Snapshot,
    ) -> Self {
        let mut roots = Self::default();

        for correspondence in alignment.correspondences() {
            let (side, node, reason, ancestor) = match correspondence {
                Correspondence::Paired { .. } => continue,
                Correspondence::LeftOnly {
                    left,
                    reason,
                    nearest_one_sided_ancestor,
                } => (Side::Left, left, reason, nearest_one_sided_ancestor),
                Correspondence::RightOnly {
                    right,
                    reason,
                    nearest_one_sided_ancestor,
                } => (Side::Right, right, reason, nearest_one_sided_ancestor),
            };

            let side_roots = match side {
                Side::Left => &mut roots.left,
                Side::Right => &mut roots.right,
            };

            match (ancestor, side_roots.last_mut()) {
                (Some(..), Some(root)) => root.occurrences += 1,
                _ => {
                    let snapshot = match side {
                        Side::Left => left_snapshot,
                        Side::Right => right_snapshot,
                    };
                    side_roots.push(OneSidedRoot {
                        side,
                        node,
                        reason: *reason,
                        occurrences: 1,
                        // Read only for a root: a descendant's content is already part
                        // of the root's, so rendering it again would repeat it
                        content: one_sided_content(snapshot.node, &node.path),
                    })
                }
            }
        }

        // Applied to the collapsed roots, not to each correspondence, so that excluding
        // a node type hides its whole subtree rather than just its root. Descendants
        // are already folded into the root's occurrence count by this point.
        roots.left.retain(|root| filter.allows_node(root.node));
        roots.right.retain(|root| filter.allows_node(root.node));

        roots
    }
}

/// Write a one-sided subtree root
fn write_one_sided(report: &mut String, root: &OneSidedRoot) -> Result<()> {
    let (marker, label) = match root.side {
        Side::Left => ("-", "left-only"),
        Side::Right => ("+", "right-only"),
    };

    let reason = match root.reason {
        UnmatchedReason::NoCompatibleCandidate => "no compatible candidate",
        UnmatchedReason::GapCheaperThanPair => "gap cheaper than pair",
    };

    let detail = if root.occurrences > 1 {
        format!("{} occurrences; {reason}", root.occurrences)
    } else {
        reason.to_string()
    };

    writeln!(
        report,
        "{marker} {label:<10} {subject} ({detail})",
        subject = occurrence(root.node)
    )?;
    if let Some(content) = &root.content {
        writeln!(
            report,
            "  {side}{content}",
            side = match root.side {
                Side::Left => "left:  ",
                Side::Right => "right: ",
            }
        )?;
    }

    Ok(())
}

/// Write a difference
fn write_difference(report: &mut String, difference: &Difference) -> Result<()> {
    match difference {
        Difference::NodeTypeChanged { left, right } => {
            writeln!(
                report,
                "≠ {:<10} {} ↔ {}",
                "node type",
                occurrence(left),
                occurrence(right)
            )?;
        }

        Difference::PropertyPresenceChanged {
            left,
            right,
            property,
            left_presence,
            right_presence,
        } => {
            let left_subject = value_subject(left, Some(property), None);
            let right_subject = value_subject(right, Some(property), None);
            writeln!(
                report,
                "± {:<10} {}",
                "presence",
                sides(&left_subject, &right_subject)
            )?;
            writeln!(report, "  left:  {}", presence(*left_presence))?;
            writeln!(report, "  right: {}", presence(*right_presence))?;
        }

        Difference::ValueChanged {
            location,
            left,
            right,
        } => {
            let left_subject = value_subject(
                &location.left,
                location.property.as_ref(),
                location.left_index,
            );
            let right_subject = value_subject(
                &location.right,
                location.property.as_ref(),
                location.right_index,
            );
            writeln!(
                report,
                "~ {:<10} {}",
                "value",
                sides(&left_subject, &right_subject)
            )?;
            writeln!(report, "  left:  {}", value_state(left))?;
            writeln!(report, "  right: {}", value_state(right))?;
        }

        Difference::ParentChanged {
            left,
            right,
            left_parent,
            right_parent,
            left_property,
            right_property,
        } => {
            writeln!(
                report,
                "→ {:<10} {} ↔ {}",
                "parent",
                occurrence(left),
                occurrence(right)
            )?;
            writeln!(
                report,
                "  left:  {}",
                parent(left_parent.as_ref(), left_property.as_ref())
            )?;
            writeln!(
                report,
                "  right: {}",
                parent(right_parent.as_ref(), right_property.as_ref())
            )?;
        }

        Difference::Reordered { left, right, .. } => {
            writeln!(
                report,
                "↕ {:<10} {} ↔ {}",
                "reordered",
                occurrence(left),
                occurrence(right)
            )?;
        }
    }

    Ok(())
}

/// The longest rendering of a one-sided occurrence that is shown
///
/// A one-sided subtree root can be a whole section, and the point of showing it is to
/// recognise it, not to read it. Anything longer is elided.
const MAX_ONE_SIDED_CHARACTERS: usize = 160;

/// Render the content of a one-sided occurrence, from the document it came from
///
/// The comparison deliberately records no values for one-sided occurrences: the
/// alignment says only that a node of some type at some path has no counterpart. So the
/// content is read back out of the original snapshot, by the path the alignment gives.
///
/// `None` when the path does not resolve, or when the occurrence has no text of its own,
/// in which case the row shows just its path and type as before.
fn one_sided_content(node: &Node, path: &NodePath) -> Option<String> {
    let text = match stencila_schema::get(node, path.clone()).ok()? {
        NodeSet::One(node) => to_text(&node),
        NodeSet::Many(nodes) => nodes.iter().map(to_text).collect::<Vec<_>>().join(" "),
    };

    // Collapsed onto one line, because a row shows a single line and a block's own
    // newlines would otherwise decide where it wraps
    let text = text.split_whitespace().collect::<Vec<_>>().join(" ");
    if text.is_empty() {
        return None;
    }

    Some(elide(&text, MAX_ONE_SIDED_CHARACTERS))
}

/// Shorten a rendering to a maximum number of characters
///
/// Counts characters rather than bytes, so that it never splits a multi-byte character.
fn elide(text: &str, maximum: usize) -> String {
    if text.chars().count() <= maximum {
        return text.to_string();
    }

    let mut elided: String = text.chars().take(maximum).collect();
    elided.push('…');
    elided
}

/// Describe the filter a comparison was made under
///
/// Returns the selectors and what they suppressed, or `None` when nothing was filtered.
/// Always reported alongside the counts, so that an `equal` verdict is never read
/// without the filter that produced it.
fn filter_description(comparison: &Comparison) -> Option<(String, String)> {
    if !comparison.is_filtered() {
        return None;
    }

    let filter = comparison.filter();
    let selectors = filter
        .exclude
        .iter()
        .map(|selector| format!("-{selector}"))
        .chain(filter.include.iter().map(|selector| format!("+{selector}")))
        .collect::<Vec<_>>()
        .join(" ");

    let tally = comparison.one_sided_tally();
    let suppressed = format!(
        "{differences} {noun}, {left_only} left-only, {right_only} right-only",
        differences = comparison.suppressed_differences(),
        noun = if comparison.suppressed_differences() == 1 {
            "difference"
        } else {
            "differences"
        },
        left_only = tally.suppressed[0],
        right_only = tally.suppressed[1],
    );

    Some((selectors, suppressed))
}

/// Render a path, using `$` for the root
fn path(path: &NodePath) -> String {
    if path.is_empty() {
        "$".to_string()
    } else {
        format!("$/{path}")
    }
}

/// Render an occurrence as its path and node type
fn occurrence(node: &NodeRef) -> String {
    format!("{} {}", path(&node.path), node.node_type)
}

/// Render the two sides of a subject, collapsing them when they are the same
fn sides(left: &str, right: &str) -> String {
    if left == right {
        left.to_string()
    } else {
        format!("{left} ↔ {right}")
    }
}

/// Render the location of a value within its occurrence
///
/// Named as `$/path NodeType.property[index]`, so that the subject of a value or
/// presence difference reads the same way as every other row's, and a path is never
/// left to be understood on its own.
fn value_subject(node: &NodeRef, property: Option<&NodeProperty>, index: Option<usize>) -> String {
    let mut subject = occurrence(node);
    if let Some(property) = property {
        subject.push('.');
        subject.push_str(&property.to_string());
    }
    if let Some(index) = index {
        subject.push_str(&format!("[{index}]"));
    }
    subject
}

/// Render the parent side of a parent change
fn parent(node: Option<&NodeRef>, property: Option<&NodeProperty>) -> String {
    let mut rendered = match node {
        Some(node) => occurrence(node),
        None => "(none)".to_string(),
    };
    if let Some(property) = property {
        rendered.push_str(&format!(" .{property}"));
    }
    rendered
}

/// Render the presence of a property
fn presence(presence: PropertyPresence) -> &'static str {
    match presence {
        PropertyPresence::Undeclared => "undeclared",
        PropertyPresence::Absent => "absent",
        PropertyPresence::Present => "present",
    }
}

/// Render the complete state of one side of a value change
fn value_state(state: &ValueState) -> String {
    match state {
        ValueState::Absent => "absent".to_string(),
        ValueState::One { value } => scalar(value),
        ValueState::Many { values } => format!(
            "[{}]",
            values.iter().map(scalar).collect::<Vec<_>>().join(", ")
        ),
    }
}

/// Render a typed scalar value
fn scalar(value: &ScalarValue) -> String {
    match value {
        ScalarValue::Null => "null".to_string(),
        ScalarValue::Boolean { value } => format!("boolean {value}"),
        ScalarValue::Integer { value } => format!("integer {value}"),
        ScalarValue::UnsignedInteger { value } => format!("unsigned {value}"),
        ScalarValue::Number { value } => format!("number {value}"),
        ScalarValue::String { value } => format!("string {}", quote(value)),
        ScalarValue::Enum {
            schema_type,
            variant,
        } => format!("enum {schema_type}.{variant}"),
        ScalarValue::Array { items } => format!(
            "array [{}]",
            items.iter().map(scalar).collect::<Vec<_>>().join(", ")
        ),
        ScalarValue::Object { entries } => format!(
            "object {{{}}}",
            entries
                .iter()
                .map(|(key, value)| format!("{}: {}", quote(key), scalar(value)))
                .collect::<Vec<_>>()
                .join(", ")
        ),
    }
}

/// Quote a string using JSON-style escaping
fn quote(value: &str) -> String {
    serde_json::to_string(value).unwrap_or_else(|_| format!("{value:?}"))
}

/// Render a count with a singular or plural noun
fn plural(count: usize, noun: &str) -> String {
    if count == 1 {
        format!("{count} {noun}")
    } else {
        format!("{count} {noun}s")
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use std::str::FromStr;

    use eyre::Result;
    use stencila_node_compare::compare;
    use stencila_schema::{Article, Block, Heading, Node, Paragraph, Section, shortcuts::t};

    use super::*;

    fn article(blocks: Vec<Block>) -> Node {
        Node::Article(Article::new(blocks))
    }

    fn para(text: &str) -> Block {
        Block::Paragraph(Paragraph::new(vec![t(text)]))
    }

    /// The two sides of a comparison, as the reports take them
    fn snapshots<'node>(
        left: &'node Node,
        right: &'node Node,
    ) -> (Snapshot<'node>, Snapshot<'node>) {
        (
            Snapshot {
                node: left,
                label: "left.smd",
            },
            Snapshot {
                node: right,
                label: "right.smd",
            },
        )
    }

    fn report(left: &Node, right: &Node, summary: bool) -> String {
        let comparison = compare(left, right).unwrap();
        let (left, right) = snapshots(left, right);
        text_report(&comparison, left, right, summary).unwrap()
    }

    fn html(left: &Node, right: &Node, summary: bool) -> String {
        let comparison = compare(left, right).unwrap();
        let (left, right) = snapshots(left, right);
        html_report(&comparison, left, right, summary).unwrap()
    }

    /// The page with an overlay, using the left document as a stand-in for a merged
    /// one so that this crate does not depend on the merge to test the rendering
    fn html_with_overlay(left: &Node, right: &Node) -> Result<String> {
        let comparison = compare(left, right)?;
        let overlay = left.clone();
        let (left, right) = snapshots(left, right);
        html_report_with_overlay(&comparison, left, right, false, &overlay)
    }

    #[test]
    fn reports_equality() {
        let node = article(vec![para("One")]);
        let report = report(&node, &node, false);

        assert_eq!(
            report,
            r#"equal
left:  left.smd
right: right.smd

correspondences: 3 paired, 0 left-only (0 roots), 0 right-only (0 roots)
differences: 0
  node type: 0  presence: 0  value: 0  parent: 0  reordered: 0
"#
        );
    }

    /// A one-sided subtree shows what it says, not just where it is
    #[test]
    fn one_sided_rows_show_their_content() {
        let left = article(vec![para("Kept"), para("Only on the left")]);
        let right = article(vec![para("Kept")]);

        let report = report(&left, &right, false);
        assert!(
            report.contains("- left-only  $/content/1 Paragraph"),
            "{report}"
        );
        assert!(
            report.contains("  left:  Only on the left"),
            "the content is read back out of the left document: {report}"
        );

        // In the view it is shaded, the same way a value only the left has is
        let page = html(&left, &right, false);
        assert!(
            page.contains("<span class=\"removed\">Only on the left</span>"),
            "{page}"
        );

        // A right-only subtree is read out of the right document, and shaded as added
        let page = html(&right, &left, false);
        assert!(
            page.contains("<span class=\"added\">Only on the left</span>"),
            "{page}"
        );
    }

    /// A rendering longer than the limit is elided rather than filling the row
    #[test]
    fn long_one_sided_content_is_elided() {
        let long = "word ".repeat(200);
        let left = article(vec![para("Kept"), para(&long)]);
        let right = article(vec![para("Kept")]);

        let report = report(&left, &right, false);
        let line = report
            .lines()
            .find(|line| line.starts_with("  left:  word"))
            .expect("Expected the one-sided content");
        assert!(line.ends_with('…'), "{line}");
        assert!(
            line.chars().count() <= MAX_ONE_SIDED_CHARACTERS + 10,
            "{line}"
        );
    }

    /// Every subject names the node type, so that a path is never read on its own
    #[test]
    fn subjects_name_the_node_type() {
        // `authors` is a structured property, so a change of presence is reported as a
        // presence difference rather than as a value change
        let left = Node::Article(Article {
            authors: Some(vec![]),
            ..Article::new(vec![para("Methods")])
        });
        let right = Node::Article(Article::new(vec![para("Method")]));

        let report = report(&left, &right, false);
        assert!(
            report.contains(
                "± presence   $ Article.authors
"
            ),
            "presence subjects name the type: {report}"
        );
        assert!(
            report.contains(
                "~ value      $/content/0/content/0 Text.value
"
            ),
            "value subjects name the type: {report}"
        );

        // And the same subjects reach the side-by-side view
        let page = html(&left, &right, false);
        assert!(page.contains("$ Article.authors"), "{page}");
        assert!(page.contains("$/content/0/content/0 Text.value"), "{page}");
    }

    #[test]
    fn reports_value_changes_with_typed_states() {
        let left = article(vec![para("Methods")]);
        let right = article(vec![para("Method")]);
        let report = report(&left, &right, false);

        assert!(report.starts_with("different\n"), "{report}");
        assert!(
            report.contains("~ value      $/content/0/content/0 Text.value\n"),
            "{report}"
        );
        assert!(report.contains("  left:  string \"Methods\"\n"), "{report}");
        assert!(report.contains("  right: string \"Method\"\n"), "{report}");
    }

    #[test]
    fn summary_stops_after_the_counts() {
        let left = article(vec![para("Methods")]);
        let right = article(vec![para("Method")]);

        let report = report(&left, &right, true);
        assert_eq!(
            report,
            r#"different
left:  left.smd
right: right.smd

correspondences: 3 paired, 0 left-only (0 roots), 0 right-only (0 roots)
differences: 1
  node type: 0  presence: 0  value: 1  parent: 0  reordered: 0
"#
        );
    }

    #[test]
    fn reports_one_sided_subtrees_collapsed_onto_their_roots() {
        let left = article(vec![para("One")]);
        let right = article(vec![
            para("One"),
            Block::Section(Section {
                content: vec![para("Two"), para("Three")],
                ..Default::default()
            }),
        ]);
        let report = report(&left, &right, false);

        // The section and its four descendants collapse onto one root
        assert!(
            report.contains(
                "correspondences: 3 paired, 0 left-only (0 roots), 5 right-only (1 root)\n"
            ),
            "{report}"
        );

        let one_sided: Vec<&str> = report
            .lines()
            .filter(|line| line.starts_with('+') || line.starts_with('-'))
            .collect();
        assert_eq!(one_sided.len(), 1, "{report}");
        assert!(
            one_sided[0].starts_with("+ right-only $/content/1 Section ("),
            "{report}"
        );
        assert!(one_sided[0].contains("5 occurrences;"), "{report}");
    }

    #[test]
    fn reports_node_type_changes() {
        let left = article(vec![para("Title")]);
        let right = article(vec![Block::Heading(Heading::new(1, vec![t("Title")]))]);
        let report = report(&left, &right, false);

        assert!(
            report.contains("≠ node type  $/content/0 Paragraph ↔ $/content/0 Heading\n"),
            "{report}"
        );
    }

    #[test]
    fn reports_reordering() {
        let left = article(vec![para("One"), para("Two"), para("Three")]);
        let right = article(vec![para("Two"), para("Three"), para("One")]);
        let report = report(&left, &right, false);

        let reordered: Vec<&str> = report
            .lines()
            .filter(|line| line.starts_with('↕'))
            .collect();
        assert!(!reordered.is_empty(), "{report}");
        assert!(reordered[0].contains(" ↔ "), "{report}");
        assert!(reordered[0].contains("Paragraph"), "{report}");
    }

    #[test]
    fn html_view_is_self_contained_and_side_by_side() {
        let left = article(vec![para("Methods")]);
        let right = article(vec![para("Method")]);
        let page = html(&left, &right, false);

        // Nothing to fetch, so that the page works from a temporary file. Document
        // content may legitimately contain a URL — a link target, a citation — so what
        // is asserted is that the page never *fetches*, not that no URL appears in it.
        assert!(page.starts_with("<!DOCTYPE html>"), "{page}");
        assert!(!page.contains("<script"), "{page}");
        assert!(!page.contains("<link"), "{page}");
        assert!(!page.contains("src=\"http"), "{page}");
        assert!(!page.contains("@import"), "{page}");

        assert!(
            page.contains(r#"<span class="status different">different</span>"#),
            "{page}"
        );
        assert!(
            page.contains(r#"<span class="path">left.smd</span>"#),
            "{page}"
        );
        assert!(
            page.contains(r#"<span class="path">right.smd</span>"#),
            "{page}"
        );

        // Both sides of the value change, in their own cells
        assert!(
            page.contains(r#"<span class="removed">Methods</span>"#),
            "{page}"
        );
        assert!(
            page.contains(r#"<span class="added">Method</span>"#),
            "{page}"
        );
    }

    #[test]
    fn html_view_leaves_one_sided_cells_empty() {
        let left = article(vec![para("One")]);
        let right = article(vec![
            para("One"),
            Block::Section(Section {
                content: vec![para("Two")],
                ..Default::default()
            }),
        ]);
        let page = html(&left, &right, false);

        assert!(page.contains(r#"<tr class="right-only">"#), "{page}");
        assert!(page.contains(r#"<td class="absent">—</td>"#), "{page}");
        assert!(page.contains("$/content/1 Section"), "{page}");
        assert!(!page.contains(r#"<tr class="left-only">"#), "{page}");
    }

    #[test]
    fn the_overlay_tab_renders_the_merged_document() -> Result<()> {
        let left = article(vec![para("Methods")]);
        let right = article(vec![para("Method")]);
        let page = html_with_overlay(&left, &right)?;

        // Both tabs, switched by the radio inputs rather than by a script
        assert!(page.contains(r#"id="tab-differences""#), "{page}");
        assert!(page.contains(r#"id="tab-overlay""#), "{page}");
        assert!(!page.contains("<script"), "{page}");

        // The overlay pane holds the DOM encoding of the document
        assert!(page.contains(r#"<section class="pane overlay">"#), "{page}");
        assert!(page.contains("<stencila-paragraph"), "{page}");
        assert!(page.contains("Methods"), "{page}");

        Ok(())
    }

    #[test]
    fn comments_are_marked_on_the_nodes_they_are_about() -> Result<()> {
        use stencila_schema::{Comment, CommentOptions, Heading, Inline, Text};

        // An overlay carrying one comment about one heading, as the merge produces
        let mut overlay = article(vec![Block::Heading(Heading {
            id: Some("mgc0".to_string()),
            ..Heading::new(1, vec![Inline::Text(Text::from("Title"))])
        })]);
        if let Node::Article(article) = &mut overlay {
            article.options.comments = Some(vec![Comment {
                content: vec![para("Property `level` changed from `1` to `2`")],
                options: Box::new(CommentOptions {
                    start_location: Some("#mgc0".to_string()),
                    end_location: Some("#mgc0".to_string()),
                    ..Default::default()
                }),
                ..Default::default()
            }]);
        }

        let left = article(vec![para("One")]);
        let right = article(vec![para("Two")]);
        let comparison = compare(&left, &right)?;
        let (left, right) = snapshots(&left, &right);
        let page = html_report_with_overlay(&comparison, left, right, false, &overlay)?;

        // The comment marks the heading in place and says what it says on hover, so
        // the reader meets it where it applies rather than in a list below
        assert!(page.contains(r#"[_id="mgc0"]"#), "{page}");
        assert!(page.contains(r#"[_id="mgc0"]:hover::after"#), "{page}");
        assert!(page.contains("Property `level` changed"), "{page}");

        // Which is instead of, not as well as, an entry in a list
        assert!(!page.contains("Elsewhere in the document"), "{page}");

        // And still nothing to fetch and no script to run
        assert!(!page.contains("<script"), "{page}");

        Ok(())
    }

    #[test]
    fn a_comment_with_nothing_to_mark_is_listed() -> Result<()> {
        use stencila_schema::Comment;

        let mut overlay = article(vec![para("Body")]);
        if let Node::Article(article) = &mut overlay {
            article.options.comments = Some(vec![Comment {
                content: vec![para("An author was removed")],
                ..Default::default()
            }]);
        }

        let left = article(vec![para("One")]);
        let right = article(vec![para("Two")]);
        let comparison = compare(&left, &right)?;
        let (left, right) = snapshots(&left, &right);
        let page = html_report_with_overlay(&comparison, left, right, false, &overlay)?;

        assert!(page.contains("Elsewhere in the document"), "{page}");
        assert!(page.contains("An author was removed"), "{page}");

        Ok(())
    }

    #[test]
    fn anchored_comments_cannot_close_the_style_element() -> Result<()> {
        use stencila_schema::{Comment, CommentOptions, Heading, Inline, Text};

        let mut overlay = article(vec![Block::Heading(Heading {
            id: Some("target".to_string()),
            ..Heading::new(1, vec![Inline::Text(Text::from("Title"))])
        })]);
        if let Node::Article(article) = &mut overlay {
            article.options.comments = Some(vec![Comment {
                content: vec![para("</style><script>alert('unsafe')</script>")],
                options: Box::new(CommentOptions {
                    start_location: Some("#target".to_string()),
                    end_location: Some("#target".to_string()),
                    ..Default::default()
                }),
                ..Default::default()
            }]);
        }

        let left = article(vec![para("One")]);
        let right = article(vec![para("Two")]);
        let comparison = compare(&left, &right)?;
        let (left, right) = snapshots(&left, &right);
        let page = html_report_with_overlay(&comparison, left, right, false, &overlay)?;

        assert!(!page.contains("<script>"), "{page}");
        assert!(page.contains(r"\3c /style>\3c script>"), "{page}");

        Ok(())
    }

    #[test]
    fn the_overlay_tab_is_disabled_without_an_overlay() {
        let left = article(vec![para("One")]);
        let right = article(vec![para("Two")]);
        let page = html(&left, &right, false);

        assert!(page.contains(r#"id="tab-overlay" disabled"#), "{page}");
        assert!(page.contains("No overlay was built"), "{page}");
    }

    #[test]
    fn html_view_rows_are_in_reading_order() {
        let left = article(vec![
            para("Alpha first paragraph"),
            para("Beta second paragraph"),
            para("Gamma third paragraph"),
        ]);
        let right = article(vec![
            para("Alpha first paragraph, revised"),
            Block::Section(Section {
                content: vec![para("An inserted section")],
                ..Default::default()
            }),
            para("Beta second paragraph, revised"),
            para("Gamma third paragraph"),
        ]);
        let page = html(&left, &right, false);

        let row = |needle: &str| {
            page.find(needle)
                .unwrap_or_else(|| panic!("Expected {needle} in: {page}"))
        };

        // The right-only section sits between the occurrences it was inserted
        // between, rather than ahead of every difference
        assert!(row("$/content/0/content/0 Text.value") < row("$/content/1 Section"));
        assert!(row("$/content/1 Section") < row("$/content/1/content/0 Text.value"));
    }

    #[test]
    fn html_view_puts_a_leading_insertion_first() {
        let left = article(vec![para("Alpha first paragraph")]);
        let right = article(vec![
            Block::Section(Section {
                content: vec![para("An inserted section")],
                ..Default::default()
            }),
            para("Alpha first paragraph, revised"),
        ]);
        let page = html(&left, &right, false);

        // Nothing on the right precedes the section, so it anchors to nothing
        assert!(
            page.find("$/content/0 Section")
                .expect("Expected the section")
                < page
                    .find("$/content/0/content/0 Text.value")
                    .expect("Expected the revision"),
            "{page}"
        );
    }

    #[test]
    fn html_view_shades_what_the_other_side_does_not_have() {
        let left = article(vec![para("The quick brown fox jumps over the lazy dog")]);
        let right = article(vec![para("The quick red fox jumps over the dog")]);
        let page = html(&left, &right, false);

        // Removed on the left, added on the right, never the other way round
        assert!(
            page.contains(r#"<span class="removed">brown</span>"#),
            "{page}"
        );
        assert!(page.contains(r#"<span class="added">red</span>"#), "{page}");
        assert!(
            !page.contains(r#"<span class="added">brown</span>"#),
            "{page}"
        );
        assert!(
            page.contains(r#"<span class="removed">lazy </span>"#),
            "{page}"
        );

        // What both sides have is left unshaded, including the rendering of the type
        // and the quoting around the value
        assert!(
            page.contains(r#"<span class="detail">string &quot;The quick <span"#),
            "{page}"
        );
    }

    #[test]
    fn shading_merges_adjacent_changes() {
        let (left, right) = shade("one two three four", "one five six four");

        let rendered = |segments: &[Segment]| {
            segments
                .iter()
                .map(|segment| {
                    if segment.changed {
                        format!("[{}]", segment.text)
                    } else {
                        segment.text.clone()
                    }
                })
                .collect::<String>()
        };

        // Two changed words become one shaded run, not two
        assert_eq!(rendered(&left), "one [two three] four");
        assert_eq!(rendered(&right), "one [five six] four");
    }

    #[test]
    fn shading_of_equal_states_changes_nothing() {
        let (left, right) = shade("string \"same\"", "string \"same\"");

        assert!(left.iter().all(|segment| !segment.changed));
        assert!(right.iter().all(|segment| !segment.changed));
    }

    #[test]
    fn very_long_states_are_not_shaded() {
        let long = "word ".repeat(MAX_SHADED_CHARACTERS);
        let (left, right) = shade(&long, "word");

        // Shown plainly, rather than spending the time to compare within them
        assert_eq!(left.len(), 1);
        assert!(!left[0].changed);
        assert_eq!(right.len(), 1);
        assert!(!right[0].changed);
    }

    #[test]
    fn html_view_escapes_document_content() {
        let left = article(vec![para("A <script>alert('x')</script> in the text")]);
        let right = article(vec![para("A <span>span</span> in the text")]);
        let page = html(&left, &right, false);

        assert!(!page.contains("<script"), "{page}");
        assert!(page.contains("&lt;"), "{page}");
        assert!(page.contains("script"), "{page}");
    }

    #[test]
    fn html_view_summary_stops_after_the_counts() {
        let left = article(vec![para("Methods")]);
        let right = article(vec![para("Method")]);

        let page = html(&left, &right, true);
        assert!(page.contains("<li><span>value</span> 1</li>"), "{page}");
        assert!(!page.contains("<table>"), "{page}");
        assert!(page.contains("--summary"), "{page}");

        // Equal documents have nothing to tabulate either
        let page = html(&left, &left, false);
        assert!(
            page.contains(r#"<span class="status equal">equal</span>"#),
            "{page}"
        );
        assert!(!page.contains("<table>"), "{page}");
        assert!(page.contains("The documents are equal."), "{page}");
    }

    #[test]
    fn renders_root_paths_as_dollar() {
        assert_eq!(path(&NodePath::new()), "$");
        assert_eq!(
            path(&NodePath::from_str("content/0").unwrap()),
            "$/content/0"
        );
    }

    #[test]
    fn renders_typed_scalars() {
        assert_eq!(scalar(&ScalarValue::Null), "null");
        assert_eq!(
            scalar(&ScalarValue::Boolean { value: true }),
            "boolean true"
        );
        assert_eq!(scalar(&ScalarValue::Integer { value: -3 }), "integer -3");
        assert_eq!(
            scalar(&ScalarValue::UnsignedInteger { value: 3 }),
            "unsigned 3"
        );
        assert_eq!(scalar(&ScalarValue::number(1.5)), "number 1.5");
        assert_eq!(
            scalar(&ScalarValue::string("a \"quoted\"\nvalue")),
            r#"string "a \"quoted\"\nvalue""#
        );
        assert_eq!(
            scalar(&ScalarValue::Enum {
                schema_type: "CitationMode".to_string(),
                variant: "Parenthetical".to_string()
            }),
            "enum CitationMode.Parenthetical"
        );
        assert_eq!(
            scalar(&ScalarValue::Array {
                items: vec![ScalarValue::Integer { value: 1 }, ScalarValue::Null]
            }),
            "array [integer 1, null]"
        );
        assert_eq!(
            scalar(
                &ScalarValue::object([
                    ("b".to_string(), ScalarValue::Null),
                    ("a".to_string(), ScalarValue::string("x"))
                ])
                .unwrap()
            ),
            r#"object {"a": string "x", "b": null}"#
        );
    }

    #[test]
    fn renders_value_states() {
        assert_eq!(value_state(&ValueState::Absent), "absent");
        assert_eq!(
            value_state(&ValueState::Many {
                values: vec![ScalarValue::string("a"), ScalarValue::string("b")]
            }),
            r#"[string "a", string "b"]"#
        );
    }
}
