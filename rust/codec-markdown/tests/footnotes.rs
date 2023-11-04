use pulldown_cmark::{html, Options, Parser};

use common_dev::pretty_assertions::assert_eq;

/// Test that the version of `pulldown_cmark` being used supports complex
/// footnote definitions (older versions do not)
#[test]
fn footnotes() {
    let md = "
This example has three footnotes [^1], a [^2], a [^3].

[^1]: A paragraph

[^2]: First paragraph.

    Second paragraph.

[^3]: A paragraph.

    ```
    A code block
    ```

    > A quote block
    ";

    let mut options = Options::empty();
    options.insert(Options::ENABLE_FOOTNOTES);

    let parser = Parser::new_ext(md, options);

    let mut html_output = String::new();
    html::push_html(&mut html_output, parser);

    assert_eq!(
        &html_output,
        r##"<p>This example has three footnotes <sup class="footnote-reference"><a href="#1">1</a></sup>, a <sup class="footnote-reference"><a href="#2">2</a></sup>, a <sup class="footnote-reference"><a href="#3">3</a></sup>.</p>
<div class="footnote-definition" id="1"><sup class="footnote-definition-label">1</sup>
<p>A paragraph</p>
</div>
<div class="footnote-definition" id="2"><sup class="footnote-definition-label">2</sup>
<p>First paragraph.</p>
<p>Second paragraph.</p>
</div>
<div class="footnote-definition" id="3"><sup class="footnote-definition-label">3</sup>
<p>A paragraph.</p>
<pre><code>A code block
</code></pre>
<blockquote>
<p>A quote block</p>
</blockquote>
</div>
"##
    );
}
