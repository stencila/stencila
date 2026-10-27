use roxmltree::{NS_XML_URI, Node};

use stencila_codec::stencila_schema::{
    Article, ArticleOptions, Block, CreativeWorkVariant, CreativeWorkVariantOrString, DateTime,
    Heading, Object, Organization, OrganizationOptions, Periodical, PersonOrOrganization,
    Primitive, PublicationIssue, PublicationVolume,
};
use stencila_codec_text_trait::to_text;

use super::{
    Context,
    bibl::{Scope, decode_author, decode_bibl_scope, decode_date, decode_idno, push_identifier},
    fragment_ids, non_empty_text, path,
    text::{Divisions, decode_divisions, decode_inlines, normalize},
};

/// Decode the `<teiHeader>` of a `<TEI>` onto an [`Article`]
///
/// The `<encodingDesc>` describes the software that encoded the TEI (for
/// GROBID, its version and the parameters it was run with) rather than the
/// work, so is recorded as lost.
pub(super) fn decode_header(node: &Node, article: &mut Article, context: &mut Context) {
    context.lost_attrs(node, &[]);

    for child in node.children() {
        match child.tag_name().name() {
            "fileDesc" => decode_file_desc(&child, article, context),
            "profileDesc" => decode_profile_desc(&child, article, context),
            _ => context.lost(&child),
        }
    }
}

/// Decode a `<fileDesc>`
fn decode_file_desc(node: &Node, article: &mut Article, context: &mut Context) {
    context.lost_attrs(node, &[]);

    for child in node.children() {
        match child.tag_name().name() {
            "titleStmt" => decode_title_stmt(&child, article, context),
            "publicationStmt" => decode_publication_stmt(&child, article, context),
            "sourceDesc" => {
                context.lost_attrs(&child, &[]);
                for grandchild in child.children() {
                    if grandchild.has_tag_name("biblStruct") {
                        decode_source(&grandchild, article, context)
                    } else {
                        context.lost(&grandchild)
                    }
                }
            }
            _ => context.lost(&child),
        }
    }
}

/// Decode a `<titleStmt>`: the title of the work and its funders
fn decode_title_stmt(node: &Node, article: &mut Article, context: &mut Context) {
    context.lost_attrs(node, &[]);

    for child in node.children() {
        match child.tag_name().name() {
            "title" => decode_title(&child, article, context),
            "funder" => decode_funder(&child, article, context),
            _ => context.lost(&child),
        }
    }
}

/// Decode a `<title>` of the work
///
/// GROBID gives the title in both the `<titleStmt>` and the `<analytic>` of
/// the `<sourceDesc>`, so a title that is the same as the one already decoded
/// is not a loss. A subtitle, or a different title, is.
fn decode_title(node: &Node, article: &mut Article, context: &mut Context) {
    context.lost_attrs(node, &["level", "type"]);

    let title = normalize(decode_inlines(node.children(), context));
    if title.is_empty() {
        return;
    }

    match &article.title {
        None if matches!(node.attribute("type"), None | Some("main")) => {
            article.title = Some(title)
        }
        Some(existing) if to_text(existing) == to_text(&title) => {}
        _ => context.lost(node),
    }
}

/// Decode a `<funder>` to one of the article's funders
///
/// A funder may refer (`ref`) to an `<org>` in the `<listOrg>` of the back
/// matter which has the numbers of the grants that it funded. The reference
/// is kept so that those grants can be attributed to the funder.
fn decode_funder(node: &Node, article: &mut Article, context: &mut Context) {
    context.lost_attrs(node, &["ref"]);

    let mut name = None;
    let mut alternate_names = Vec::new();
    let mut identifiers = None;
    for child in node.children().filter(Node::is_element) {
        match child.tag_name().name() {
            "orgName" => {
                context.lost_attrs(&child, &["type"]);
                let Some(text) = non_empty_text(&child) else {
                    continue;
                };
                match child.attribute("type") {
                    Some("abbreviated") => alternate_names.push(text),
                    _ if name.is_none() => name = Some(text),
                    _ => alternate_names.push(text),
                }
            }
            "idno" => {
                let mut doi = None;
                decode_idno(&child, &mut doi, &mut identifiers, context);
                if let Some(doi) = doi {
                    push_identifier(&mut identifiers, Some("DOI".into()), &doi);
                }
            }
            _ => context.lost(&child),
        }
    }

    if name.is_none() && identifiers.is_none() {
        return;
    }

    let funder = PersonOrOrganization::Organization(Organization {
        name,
        options: Box::new(OrganizationOptions {
            alternate_names: (!alternate_names.is_empty()).then_some(alternate_names),
            identifiers,
            ..Default::default()
        }),
        ..Default::default()
    });

    for id in fragment_ids(node.attribute("ref")) {
        context.funders.push((id, funder.clone()));
    }

    let funders = article.options.funders.get_or_insert_default();
    if !funders.contains(&funder) {
        funders.push(funder);
    }
}

/// Decode a `<publicationStmt>`: the publisher, license and publication date
///
/// GROBID writes empty `<publisher>` and `<licence>` elements when the PDF
/// does not state them, which are not losses.
fn decode_publication_stmt(node: &Node, article: &mut Article, context: &mut Context) {
    context.lost_attrs(node, &[]);

    for child in node.children() {
        match child.tag_name().name() {
            "publisher" => {
                context.lost_attrs(&child, &[]);
                if let Some(name) = non_empty_text(&child) {
                    article.options.publisher =
                        Some(PersonOrOrganization::Organization(Organization {
                            name: Some(name),
                            ..Default::default()
                        }));
                }
            }
            "availability" => {
                context.lost_attrs(&child, &["status"]);
                for grandchild in child.children() {
                    if grandchild.has_tag_name("licence") {
                        decode_licence(&grandchild, article, context);
                    } else {
                        context.lost(&grandchild);
                    }
                }
            }
            "date" => decode_date_published(&child, article, context),
            "idno" => decode_idno(
                &child,
                &mut article.doi,
                &mut article.options.identifiers,
                context,
            ),
            _ => context.lost(&child),
        }
    }
}

/// Decode a `<licence>` to a license of the article
///
/// The `target` is the URL of the license. As when decoding JATS, the prose of
/// a license is not kept as a license itself, so is recorded as lost.
fn decode_licence(node: &Node, article: &mut Article, context: &mut Context) {
    context.lost_attrs(node, &["target"]);

    if let Some(target) = node.attribute("target").map(str::trim)
        && !target.is_empty()
    {
        let license = CreativeWorkVariantOrString::String(target.to_string());
        let licenses = article.options.licenses.get_or_insert_default();
        if !licenses.contains(&license) {
            licenses.push(license);
        }
    }

    if non_empty_text(node).is_some() {
        context.lost_label([&path(node), "/text()"].concat());
    }
}

/// Decode a `<date>` as the date the article was published
fn decode_date_published(node: &Node, article: &mut Article, context: &mut Context) {
    if !matches!(node.attribute("type"), None | Some("published")) {
        context.lost(node);
        return;
    }

    if let Some(date) = decode_date(node, context)
        && article.date_published.is_none()
    {
        article.date_published = Some(DateTime::new(date));
    }
}

/// Decode the `<biblStruct>` of the `<sourceDesc>`: the bibliographic record
/// of the article itself
///
/// Its `<analytic>` has the authors, and its `<monogr>` the journal, volume,
/// issue and pages, of the article. GROBID's `<note type="submission">` (the
/// received and accepted dates as written) is recorded as lost, as are its
/// other notes.
fn decode_source(node: &Node, article: &mut Article, context: &mut Context) {
    context.lost_attrs(node, &["status"]);

    for child in node.children() {
        match child.tag_name().name() {
            "analytic" => {
                context.lost_attrs(&child, &[]);
                for grandchild in child.children() {
                    match grandchild.tag_name().name() {
                        "author" => {
                            if let Some(author) = decode_author(&grandchild, context) {
                                article.authors.get_or_insert_default().push(author);
                            }
                        }
                        "title" => decode_title(&grandchild, article, context),
                        "idno" => decode_idno(
                            &grandchild,
                            &mut article.doi,
                            &mut article.options.identifiers,
                            context,
                        ),
                        _ => context.lost(&grandchild),
                    }
                }
            }
            "monogr" => decode_source_monogr(&child, article, context),
            "idno" => decode_idno(
                &child,
                &mut article.doi,
                &mut article.options.identifiers,
                context,
            ),
            "note" => match child.attribute("type") {
                Some(note_type) => {
                    context.lost_label(format!("{}[@type='{note_type}']", path(&child)))
                }
                None => context.lost(&child),
            },
            _ => context.lost(&child),
        }
    }
}

/// Decode the `<monogr>` of the article's own `<biblStruct>`
///
/// The journal, volume and issue become the article's `isPartOf`, nested in
/// the same way as when decoding JATS (an issue, in a volume, of a periodical).
fn decode_source_monogr(node: &Node, article: &mut Article, context: &mut Context) {
    context.lost_attrs(node, &[]);

    let mut periodical: Option<Periodical> = None;
    let mut scope = Scope::default();

    for child in node.children() {
        match child.tag_name().name() {
            "title" if child.attribute("level") == Some("j") => {
                context.lost_attrs(&child, &["level", "type"]);
                let Some(name) = non_empty_text(&child) else {
                    continue;
                };
                let periodical = periodical.get_or_insert_default();
                if periodical.name.is_none() && child.attribute("type") != Some("abbrev") {
                    periodical.name = Some(name);
                } else if periodical.name.as_ref() != Some(&name) {
                    periodical
                        .options
                        .alternate_names
                        .get_or_insert_default()
                        .push(name);
                }
            }
            "idno"
                if child
                    .attribute("type")
                    .is_some_and(|id_type| id_type.to_lowercase().contains("issn")) =>
            {
                context.lost_attrs(&child, &["type"]);
                if let Some(issn) = non_empty_text(&child) {
                    periodical
                        .get_or_insert_default()
                        .options
                        .issns
                        .get_or_insert_default()
                        .push(issn);
                }
            }
            "idno" => decode_idno(
                &child,
                &mut article.doi,
                &mut article.options.identifiers,
                context,
            ),
            "imprint" => {
                context.lost_attrs(&child, &[]);
                for grandchild in child.children() {
                    match grandchild.tag_name().name() {
                        "biblScope" => decode_bibl_scope(&grandchild, &mut scope, context),
                        "date" => decode_date_published(&grandchild, article, context),
                        "publisher" => {
                            context.lost_attrs(&grandchild, &[]);
                            if article.options.publisher.is_none()
                                && let Some(name) = non_empty_text(&grandchild)
                            {
                                article.options.publisher =
                                    Some(PersonOrOrganization::Organization(Organization {
                                        name: Some(name),
                                        ..Default::default()
                                    }));
                            }
                        }
                        _ => context.lost(&grandchild),
                    }
                }
            }
            _ => context.lost(&child),
        }
    }

    let Scope {
        volume_number,
        issue_number,
        page_start,
        page_end,
    } = scope;
    if page_start.is_some() {
        article.options.page_start = page_start;
    }
    if page_end.is_some() {
        article.options.page_end = page_end;
    }

    let mut work = periodical.map(CreativeWorkVariant::Periodical);
    if volume_number.is_some() {
        work = Some(CreativeWorkVariant::PublicationVolume(PublicationVolume {
            volume_number,
            is_part_of: work.map(Box::new),
            ..Default::default()
        }));
    }
    if issue_number.is_some() {
        work = Some(CreativeWorkVariant::PublicationIssue(PublicationIssue {
            issue_number,
            is_part_of: work.map(Box::new),
            ..Default::default()
        }));
    }
    if work.is_some() {
        article.options.is_part_of = work;
    }
}

/// Decode a `<profileDesc>`: the abstracts and keywords
///
/// Consecutive abstracts with the same language are joined in source order.
/// The first nonempty group becomes the article's abstract. Subsequent groups
/// become article parts, as for additional JATS abstracts, with their language
/// in `inLanguage` metadata (there is no dedicated schema property for it).
///
/// An article has no property for the language of its abstract either, so the
/// language of the first group is lost. Where it is inherited, it is recorded
/// as lost on the ancestor that sets it, otherwise on each of the abstracts.
fn decode_profile_desc(node: &Node, article: &mut Article, context: &mut Context) {
    context.lost_attrs(node, &[]);

    // Indentation and comments do not break a run of consecutive abstracts.
    let mut children = node
        .children()
        .filter(|child| {
            child.is_element()
                || (child.is_text() && child.text().is_some_and(|text| !text.trim().is_empty()))
        })
        .peekable();
    while let Some(child) = children.next() {
        match child.tag_name().name() {
            "abstract" => {
                let language = effective_language(&child);
                let mut group = vec![child];
                while let Some(next) = children.next_if(|next| {
                    next.has_tag_name("abstract") && effective_language(next) == language
                }) {
                    group.push(next);
                }

                let content = group
                    .iter()
                    .flat_map(|element| decode_abstract(element, context))
                    .collect::<Vec<_>>();
                if content.is_empty() {
                    continue;
                }

                if article.r#abstract.is_none() {
                    // A language that differs from the inherited one can only
                    // come from each abstract's own `xml:lang`
                    if language != effective_language(node) {
                        for element in &group {
                            context.lost_attr(element, "lang");
                        }
                    }
                    article.r#abstract = Some(content);
                } else {
                    article.options.parts.get_or_insert_default().push(
                        CreativeWorkVariant::Article(Article {
                            r#abstract: Some(content),
                            options: Box::new(ArticleOptions {
                                extra: language.map(|language| {
                                    Object::from([("inLanguage", Primitive::String(language))])
                                }),
                                ..Default::default()
                            }),
                            ..Default::default()
                        }),
                    );
                }
            }
            "textClass" => {
                context.lost_attrs(&child, &[]);
                for grandchild in child.children() {
                    if grandchild.has_tag_name("keywords") {
                        decode_keywords(&grandchild, article, context);
                    } else {
                        context.lost(&grandchild);
                    }
                }
            }
            _ => context.lost(&child),
        }
    }
}

/// The effective language of an element, including an inherited `xml:lang`.
///
/// An empty `xml:lang` value resets the inherited language. Language tags are
/// compared without regard to case; an unspecified language stays distinct
/// from a specified one.
fn effective_language(node: &Node) -> Option<String> {
    node.ancestors()
        .find_map(|ancestor| ancestor.attribute((NS_XML_URI, "lang")))
        .filter(|language| !language.is_empty())
        .map(str::to_ascii_lowercase)
}

/// Decode an `<abstract>`
///
/// The content is decoded with the headings of any sections at level 2, as
/// when decoding a JATS abstract, and a heading that is just "Abstract" is
/// dropped. GROBID puts such a heading in a `<div>`, so a section headed
/// "Abstract" is replaced by the rest of its content, unless it has an id, in
/// which case only the heading is dropped.
fn decode_abstract(node: &Node, context: &mut Context) -> Vec<Block> {
    // Only `xml:lang` is decoded (by the caller), not other attributes with
    // the same local name, so this does not use `lost_attrs`
    for attribute in node.attributes() {
        if !(attribute.namespace() == Some(NS_XML_URI) && attribute.name() == "lang") {
            context.lost_attr(node, attribute.name());
        }
    }

    let is_abstract_heading = |block: &Block| {
        matches!(block, Block::Heading(Heading { content, .. })
            if to_text(content).trim().to_lowercase() == "abstract")
    };

    decode_divisions(node.children(), 1, Divisions::Sections, context)
        .into_iter()
        .flat_map(|block| match block {
            Block::Section(mut section)
                if section.content.first().is_some_and(is_abstract_heading) =>
            {
                section.content.remove(0);
                if section.id.is_some() {
                    vec![Block::Section(section)]
                } else {
                    section.content
                }
            }
            block if is_abstract_heading(&block) => Vec::new(),
            block => vec![block],
        })
        .collect()
}

/// Decode a `<keywords>`
///
/// Each `<term>` is a keyword. Keywords given as text, without terms, are kept
/// as a single keyword rather than guessing how they are separated.
fn decode_keywords(node: &Node, article: &mut Article, context: &mut Context) {
    context.lost_attrs(node, &[]);

    let mut keywords = Vec::new();
    for child in node.children() {
        if child.has_tag_name("term") {
            context.lost_attrs(&child, &[]);
            keywords.extend(non_empty_text(&child));
        } else if child.is_element() {
            context.lost(&child);
        }
    }
    if keywords.is_empty() {
        keywords.extend(non_empty_text(node));
    }

    if !keywords.is_empty() {
        article
            .options
            .keywords
            .get_or_insert_default()
            .append(&mut keywords);
    }
}
