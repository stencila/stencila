//! Equivalent JATS and TEI affiliations should have the same content shape.

use serde_json::{Value, json};
use stencila_codec::eyre::Result;

fn assert_affiliation_shape(jats_aff: &str, tei_aff: &str, expected: Value) -> Result<()> {
    let (tei, _) = stencila_codec_tei::decode(
        &format!(
            r#"<TEI><teiHeader><fileDesc><sourceDesc><biblStruct><analytic>
              <author><persName><surname>Smith</surname></persName>
                <affiliation>{tei_aff}</affiliation>
              </author>
            </analytic></biblStruct></sourceDesc></fileDesc></teiHeader></TEI>"#,
        ),
        None,
    )?;
    assert_eq!(
        serde_json::to_value(&tei)?["authors"][0]["affiliations"],
        json!([expected]),
        "TEI affiliation"
    );

    // Affiliations can be inline or referenced from article metadata.
    for referenced in [false, true] {
        let aff = format!(r#"<aff id="aff1">{jats_aff}</aff>"#);
        let inline = if referenced {
            r#"<xref ref-type="aff" rid="aff1"/>"#
        } else {
            &aff
        };
        let standalone = if referenced { aff.as_str() } else { "" };
        let (jats, _) = stencila_codec_jats::decode(
            &format!(
                r#"<article><front><article-meta><contrib-group>
                  <contrib contrib-type="author"><name><surname>Smith</surname></name>{inline}</contrib>
                </contrib-group>{standalone}</article-meta></front></article>"#,
            ),
            None,
        )?;
        assert_eq!(
            serde_json::to_value(&jats)?["authors"][0]["affiliations"],
            json!([expected]),
            "JATS affiliation (referenced={referenced})"
        );
        for (left, right) in [(&jats, &tei), (&tei, &jats)] {
            let comparison = stencila_node_compare::compare(left, right)?;
            assert!(comparison.is_equal(), "{comparison:#?}");
        }
    }
    Ok(())
}

#[test]
fn structured_affiliation_shape_matches_jats() -> Result<()> {
    assert_affiliation_shape(
        r#"<label>1</label><institution>Department of Physics</institution>,
          <institution>University of Auckland</institution>,
          <addr-line>Auckland</addr-line>, <country>New Zealand</country>"#,
        r#"<orgName type="department">Department of Physics</orgName>
          <orgName type="institution">University of Auckland</orgName>
          <address><settlement>Auckland</settlement><country>New Zealand</country></address>"#,
        json!({
            "type": "Organization",
            "name": "Department of Physics, University of Auckland",
            "address": "Auckland, New Zealand"
        }),
    )
}

#[test]
fn nested_affiliation_text_matches_jats() -> Result<()> {
    // eLife uses named-content within addr-line; inline formatting must also
    // survive in names, both directly under aff and within institution-wrap.
    for institution in [
        "<institution>University <italic>of</italic> Auckland</institution>",
        "<institution-wrap><institution>University <italic>of</italic> Auckland</institution></institution-wrap>",
    ] {
        assert_affiliation_shape(
            &format!(
                r#"<named-content content-type="organisation-division">Department of <italic>Physics</italic></named-content>
                  {institution}
                  <addr-line><named-content content-type="city">Auckland</named-content></addr-line>
                  <country>New <bold>Zealand</bold></country>"#,
            ),
            r#"<orgName>Department of <hi rend="italic">Physics</hi></orgName><orgName>University of Auckland</orgName>
              <address><settlement>Auckland</settlement><country>New <hi rend="bold">Zealand</hi></country></address>"#,
            json!({
                "type": "Organization",
                "name": "Department of Physics, University of Auckland",
                "address": "Auckland, New Zealand"
            }),
        )?;
    }
    Ok(())
}

#[test]
fn affiliation_whitespace_and_separators_match_jats() -> Result<()> {
    assert_affiliation_shape(
        "<institution>Department of\n\tPhysics;</institution> ;\n\t
          <institution-wrap><institution>University of Auckland, </institution></institution-wrap>
          Auckland,\n\tNew Zealand;",
        "<orgName>Department of\n\tPhysics;</orgName><orgName>University of Auckland, </orgName>
          <address><addrLine>Auckland,\n\tNew Zealand;</addrLine></address>",
        json!({
            "type": "Organization",
            "name": "Department of Physics, University of Auckland",
            "address": "Auckland, New Zealand"
        }),
    )
}

#[test]
fn affiliation_postal_address_matches_jats() -> Result<()> {
    for region in [
        "<state>Auckland Region</state>",
        r#"<named-content content-type="country-part">Auckland <italic>Region</italic></named-content>"#,
    ] {
        assert_affiliation_shape(
            &format!(
                r#"<institution>University</institution><addr-line>123 Example Street</addr-line>
                  <city>Auckland</city>{region}<postal-code>1010</postal-code><country>New Zealand</country>"#,
            ),
            r#"<orgName>University</orgName><address><addrLine>123 Example Street</addrLine>
              <settlement>Auckland</settlement><region>Auckland Region</region><postCode>1010</postCode><country>New Zealand</country></address>"#,
            json!({
                "type": "Organization",
                "name": "University",
                "address": "123 Example Street, Auckland, Auckland Region, 1010, New Zealand"
            }),
        )?;
    }
    Ok(())
}

#[test]
fn unstructured_affiliation_shape_matches_jats() -> Result<()> {
    // Without a marked institution, both codecs retain the whole affiliation as
    // its name. They cannot infer a reliable institution/address boundary.
    for (jats_aff, tei_aff) in [
        (
            "<label>1</label><addr-line>University of Auckland, Auckland, New Zealand</addr-line>",
            "<address><addrLine>University of Auckland, Auckland, New Zealand</addrLine></address>",
        ),
        (
            "<sup>1</sup>University of Auckland, Auckland, New Zealand.",
            "University of Auckland, Auckland, New Zealand.",
        ),
        (
            "University of <italic>Auckland</italic>, Auckland<!-- city -->, New Zealand",
            r#"<address><addrLine>University of <hi rend="italic">Auckland</hi>, Auckland, New Zealand</addrLine></address>"#,
        ),
    ] {
        assert_affiliation_shape(
            jats_aff,
            tei_aff,
            json!({
                "type": "Organization",
                "name": "University of Auckland, Auckland, New Zealand"
            }),
        )?;
    }
    Ok(())
}

#[test]
fn affiliation_address_text_matches_jats() -> Result<()> {
    for address in [
        "Auckland, New Zealand",
        "Auckland, <country>New Zealand</country>",
        "Auckland<lb/>New Zealand",
    ] {
        assert_affiliation_shape(
            r#"<institution>University</institution><city>Auckland</city><country>New Zealand</country>"#,
            &format!("<orgName>University</orgName><address>{address}</address>"),
            json!({
                "type": "Organization",
                "name": "University",
                "address": "Auckland, New Zealand"
            }),
        )?;
    }
    Ok(())
}

#[test]
fn affiliation_markers_are_not_text() -> Result<()> {
    // Footnotes and reference markers are not part of a name, but superscripts
    // within it are.
    for (institution, name) in [
        (
            r#"<institution>University<xref ref-type="fn" rid="n1">1</xref></institution>"#,
            "University",
        ),
        (
            r#"<institution-wrap><institution>University<xref ref-type="fn" rid="n1">1</xref></institution></institution-wrap>"#,
            "University",
        ),
        (
            "<institution>2<sup>nd</sup> Institute</institution>",
            "2nd Institute",
        ),
    ] {
        assert_affiliation_shape(
            &format!("<sup>1</sup>{institution}"),
            &format!("<orgName>{name}</orgName>"),
            json!({
                "type": "Organization",
                "name": name
            }),
        )?;
    }
    Ok(())
}

#[test]
fn affiliation_address_line_parts_match_jats() -> Result<()> {
    // An address line can contain address fields, and institutions, which are
    // separate parts of the affiliation.
    for (jats_aff, tei_aff) in [
        (
            "<institution>University</institution><addr-line><city>Boston</city><state>MA</state></addr-line>",
            "<orgName>University</orgName><address><settlement>Boston</settlement><region>MA</region></address>",
        ),
        (
            "<addr-line><institution>University</institution>, <city>Boston</city>, MA</addr-line>",
            "<orgName>University</orgName><address><settlement>Boston</settlement>, MA</address>",
        ),
        (
            "<institution>University</institution>, Bos<!-- editorial -->ton, MA",
            "<orgName>University</orgName><address>Bos<!-- editorial -->ton, MA</address>",
        ),
    ] {
        assert_affiliation_shape(
            jats_aff,
            tei_aff,
            json!({
                "type": "Organization",
                "name": "University",
                "address": "Boston, MA"
            }),
        )?;
    }
    Ok(())
}
