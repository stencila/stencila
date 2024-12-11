use roxmltree::Node;

use codec::{
    schema::{
        Article, Block, Primitive, PropertyValue, PropertyValueOrString, Section, SectionType,
        StringOrNumber, ThingType,
    },
    Losses,
};

use super::{
    body::decode_blocks,
    utilities::{extend_path, record_attrs_lost, record_node_lost},
};

/// Decode the `<front>` of an `<article>`
///
/// Recursively descends into the frontmatter, setting or adding to, properties of the
/// Stencila [`Article`]. An easier approach would be to use XPath as we did in Encoda
/// (https://github.com/stencila/encoda/blob/7dd7b143d0edcafa67cab96bf21dc3c077613fcc/src/codecs/jats/index.ts#L377)
/// However, the approach used here has the advantage of allowing us to enumerate tags
/// and attributes that are not handled (via `losses`).
pub(super) fn decode_front(path: &str, node: &Node, article: &mut Article, losses: &mut Losses) {
    for child in node.children() {
        let tag = child.tag_name().name();
        let child_path = extend_path(path, tag);
        match tag {
            "article-meta" => decode_article_meta(&child_path, &child, article, losses),
            "journal-meta" => decode_journal_meta(&child_path, &child, article, losses),
            _ => record_node_lost(&path, &child, losses),
        };
    }
}

/// Decode an `<article-meta>` tag to properties on an [`Article`]
fn decode_article_meta(path: &str, node: &Node, article: &mut Article, losses: &mut Losses) {
    for child in node.children() {
        let tag = child.tag_name().name();
        let child_path = extend_path(path, tag);
        match tag {
            "abstract" => decode_abstract(&child_path, &child, article, losses),
            "article-categories" => decode_article_categories(&child_path, &child, article, losses),
            "article-id" => decode_article_id(&child_path, &child, article, losses),
            "article-version" => decode_article_version(&child_path, &child, article, losses),
            _ => record_node_lost(path, &child, losses),
        };
    }
}

/// Decode an `<abstract>` element by adding it as a [`Section`] to the start of the [`Article`]
fn decode_abstract(path: &str, node: &Node, article: &mut Article, losses: &mut Losses) {
    record_attrs_lost(path, node, [], losses);

    let content = decode_blocks(path, node.children(), losses, 0);

    let section = Block::Section(Section {
        section_type: Some(SectionType::Abstract),
        content,
        ..Default::default()
    });

    article.content.insert(0, section);
}

/// Decode an `<article-categories>` element
fn decode_article_categories(path: &str, node: &Node, article: &mut Article, losses: &mut Losses) {
    record_attrs_lost(path, node, [], losses);

    for child in node.children() {
        let tag = child.tag_name().name();
        let child_path = extend_path(path, tag);
        match tag {
            "subj-group" => decode_subj_group(&child_path, &child, article, losses),
            _ => record_node_lost(path, &child, losses),
        };
    }
}

/// Decode a `<subj-group>` element by adding the text content of the inner `<subject>`
/// to the article's `about` property
fn decode_subj_group(path: &str, node: &Node, article: &mut Article, losses: &mut Losses) {
    let subject_type = node.attribute("subj-group-type").map(String::from);

    record_attrs_lost(path, node, ["subj-group-type"], losses);

    let mut subject = None;
    for child in node.children() {
        if child.tag_name().name() == "subject" {
            if let Some(text) = child.text() {
                subject = Some(text.to_string());
            }
        } else {
            record_node_lost(path, &child, losses);
        }
    }
    let Some(subject) = subject else { return };

    let item = ThingType::PropertyValue(PropertyValue {
        property_id: subject_type,
        value: Primitive::String(subject),
        ..Default::default()
    });

    if let Some(about) = &mut article.options.about {
        about.push(item);
    } else {
        article.options.about = Some(vec![item])
    }
}

/// Decode an `<article-id>` element
fn decode_article_id(path: &str, node: &Node, article: &mut Article, losses: &mut Losses) {
    let property_id = node.attribute("pub-id-type").map(String::from);

    record_attrs_lost(path, node, ["pub-id-type"], losses);

    let Some(id) = node.text() else { return };

    let item = if property_id.is_some() {
        PropertyValueOrString::PropertyValue(PropertyValue {
            property_id,
            value: Primitive::String(id.into()),
            ..Default::default()
        })
    } else {
        PropertyValueOrString::String(id.into())
    };

    if let Some(about) = &mut article.options.identifiers {
        about.push(item);
    } else {
        article.options.identifiers = Some(vec![item])
    }
}

/// Decode an `<article-version>` element
fn decode_article_version(path: &str, node: &Node, article: &mut Article, losses: &mut Losses) {
    record_attrs_lost(path, node, [], losses);

    if let Some(version) = node.text() {
        article.options.version = Some(StringOrNumber::String(version.into()))
    };
}

/// Decode an `<journal-meta>` tag to properties on an [`Article`]
fn decode_journal_meta(path: &str, node: &Node, _article: &mut Article, losses: &mut Losses) {
    for child in node.children() {
        let tag = child.tag_name().name();
        let _child_path = extend_path(path, tag);
        match tag {
            _ => record_node_lost(path, &child, losses),
        };
    }
}
