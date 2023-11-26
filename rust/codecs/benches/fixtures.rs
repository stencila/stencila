use codec::schema::{shortcuts::*, AdmonitionType, ClaimType, Node, NoteType};

/// Creates an `Article` node with (roughly) one of each node type
#[allow(unused)]
pub(crate) fn one_of_each() -> Node {
    art([
        // Heading
        h1([t("heading")]),
        // Paragraph
        p([
            // Text
            t("text"),
            // Marks
            em([t("emphasis")]),
            qi([t("q")]),
            stg([t("strong")]),
            stk([t("strike")]),
            sub([t("subscript")]),
            sup([t("superscript")]),
            u([t("underline")]),
            // Media objects
            aud("url"),
            img("url"),
            vid("url"),
            // Other
            btn("name", "code"),
            ce("code", Some("lang")),
            ci("code"),
            ct("target"),
            ctg(["target1", "target2"]),
            dei([t("text")]),
            isi([t("text")]),
            lnk([t("link")], "url"),
            mi("math", Some("lang")),
            nte(NoteType::default(), [p([t("text")])]),
            par("name"),
            sti("style", [t("styled")]),
        ]),
        // Quote, math and code blocks
        cb("code block", Some("lang")),
        cc("code chunk", Some("lang")),
        mb("math block", Some("lang")),
        qb([p([t("quote block")])]),
        // List
        ol([li([t("text")])]),
        ul([li([t("text")])]),
        // Table
        tbl([tr([th([t("col")])]), tr([td([t("cell")])])]),
        // Other
        adm(AdmonitionType::default(), Some("title"), [p([t("text")])]),
        clb("source", [arg("name", "code")]),
        clm(ClaimType::default(), [p([t("text")])]),
        stb("code", [p([t("text")])]),
        fig([p([img("url")])]),
        ifb([ibc("code", Some("lang"), [p([t("text")])])]),
        inb("source"),
        frb("symbol", "code", [p([t("text")])]),
        sec([p([t("text")])]),
        tb(),
    ])
}

/// Include a file from the `article-ark` example as a `str`
#[macro_export]
macro_rules! include_ark_str {
    ($extension:literal) => {
        include_str!(concat!(
            "../../../examples/nodes/article-ark/article-ark.",
            $extension
        ))
    };
}

/// Include a file from the `article-ark` example as bytes
#[macro_export]
macro_rules! include_ark_bytes {
    ($extension:literal) => {
        include_bytes!(concat!(
            "../../../examples/nodes/article-ark/article-ark.",
            $extension
        ))
    };
}
