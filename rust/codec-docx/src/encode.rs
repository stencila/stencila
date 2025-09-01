use std::{
    collections::BTreeMap,
    fs::File,
    io::{Read, Write},
    path::Path,
};

use regex::Regex;
use uuid::Uuid;
use zip::{ZipArchive, ZipWriter, write::SimpleFileOptions};

use codec::eyre::{Context, OptionExt, Result, eyre};
use codec_utils::move_file;

/// Encode custom data and properties into a DOCX
pub fn data_and_properties(
    path: &Path,
    data: Vec<(String, String)>,
    properties: Vec<(String, String)>,
) -> Result<()> {
    if data.is_empty() && properties.is_empty() {
        return Ok(());
    }

    // Read the DOCX (ZIP) into a BTreeMap for easy lookup and replacement.
    let mut docx =
        File::open(path).wrap_err_with(|| eyre!("unable to open: {}", path.display()))?;
    let mut zip = ZipArchive::new(&mut docx)
        .wrap_err_with(|| eyre!("DOCX is not a valid zip: {}", path.display()))?;
    let mut parts: BTreeMap<String, Vec<u8>> = BTreeMap::new();

    // Read all existing files
    for index in 0..zip.len() {
        let mut file = zip.by_index(index)?;
        let mut buffer = Vec::with_capacity(file.size() as usize);
        file.read_to_end(&mut buffer)?;
        parts.insert(file.name().to_owned(), buffer);
    }

    // Fetch or create [Content_Types].xml.
    const CONTENT_TYPES: &str = "[Content_Types].xml";
    let mut content_types_xml = if let Some(bytes) = parts.get(CONTENT_TYPES) {
        String::from_utf8(bytes.clone())?
    } else {
        r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types"></Types>"#
            .to_string()
    };

    // Fetch or create docProps/custom.xml.
    const CUSTOM_PROPS: &str = "docProps/custom.xml";
    let mut custom_xml = if let Some(bytes) = parts.get(CUSTOM_PROPS) {
        String::from_utf8(bytes.clone())?
    } else {
        r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Properties xmlns="http://schemas.openxmlformats.org/officeDocument/2006/custom-properties" xmlns:vt="http://schemas.openxmlformats.org/officeDocument/2006/docPropsVTypes"></Properties>"#
            .to_string()
    };

    // Fetch or create word/_rels/document.xml.rels.
    const DOCUMENT_RELS: &str = "word/_rels/document.xml.rels";
    let mut rels_xml = if let Some(bytes) = parts.get(DOCUMENT_RELS) {
        String::from_utf8(bytes.clone())?
    } else {
        r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships"></Relationships>"#
            .to_string()
    };

    // Insert each data embedding as /customXml/*.xml plus its property part.
    for (index, (name, payload)) in data.into_iter().enumerate() {
        // It is necessary for the starting id to be "1", at least for round-trip
        // preservation after editing with LibreOffice
        let id = index + 1;
        let data_part = format!("customXml/item{id}.xml");
        let props_part = format!("customXml/itemProps{id}.xml");
        let rel_part = format!("customXml/_rels/item{id}.xml.rels");

        let data_xml = format!(
            r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<data name="{name}"><![CDATA[{payload}]]></data>"#
        );
        parts.insert(data_part.clone(), data_xml.into_bytes());

        let guid = Uuid::new_v4().to_string();
        let props_xml = format!(
            r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<ds:datastoreItem ds:itemID="{{{guid}}}" xmlns:ds="http://schemas.openxmlformats.org/officeDocument/2006/customXml"/>"#
        );
        parts.insert(props_part.clone(), props_xml.into_bytes());

        parts.insert(rel_part, r#"<?xml version='1.0' encoding='UTF-8' standalone='yes'?>
<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">
  <Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/customXmlProps" Target="itemProps1.xml"/>
</Relationships>
"#.into());

        content_types_xml = insert_override(
            &content_types_xml,
            &format!("/{data_part}"),
            "application/xml",
        )?;
        content_types_xml = insert_override(
            &content_types_xml,
            &format!("/{props_part}"),
            "application/vnd.openxmlformats-officedocument.customXmlProperties+xml",
        )?;

        rels_xml = insert_relationship(&rels_xml, &format!("../{data_part}"))?;
    }

    // Append or extend custom document properties.
    custom_xml = add_custom_props(&custom_xml, properties)?;

    // Replace updated XML parts in the HashMap.
    parts.insert(CONTENT_TYPES.to_string(), content_types_xml.into_bytes());
    parts.insert(CUSTOM_PROPS.to_string(), custom_xml.into_bytes());
    parts.insert(DOCUMENT_RELS.to_string(), rels_xml.into_bytes());

    // Re-assemble the DOCX.
    let mut tmp = tempfile::NamedTempFile::new()?;
    let mut writer = ZipWriter::new(&mut tmp);
    let opts = SimpleFileOptions::default();

    for (name, data) in parts {
        writer.start_file(name, opts)?;
        writer.write_all(&data)?;
    }
    writer.finish()?;

    move_file(tmp.path(), path)?;

    Ok(())
}

/// Insert an `<Override …/>` into `[Content_Types].xml` unless one for `part` already exists
pub fn insert_override(xml: &str, part: &str, content_type: &str) -> Result<String> {
    // Return early if an Override for this part is already present
    if xml.contains(&format!(r#"PartName="{part}""#)) {
        return Ok(xml.to_owned());
    }

    // Build the tag we want to inject
    let tag = format!(r#"<Override PartName="{part}" ContentType="{content_type}"/>"#);

    // Find the last </Types> and splice the tag in front of it
    let pos = xml
        .rfind("</Types>")
        .ok_or_eyre("[Content_Types].xml is missing </Types> tag")?;

    let mut out = String::with_capacity(xml.len() + tag.len());
    out.push_str(&xml[..pos]);
    out.push_str(&tag);
    out.push_str(&xml[pos..]);
    Ok(out)
}

/// Append a `<Relationship …/>` to `word/_rels/document.xml.rels` if it isn’t there already.
pub fn insert_relationship(xml: &str, target: &str) -> Result<String> {
    // Skip if a Relationship for this target already exists
    if xml.contains(&format!(r#"Target="{target}""#)) {
        return Ok(xml.to_owned());
    }

    // Build a new relationship tag
    let id = format!("rId{}", Uuid::new_v4().simple());
    let rel_type = "http://schemas.openxmlformats.org/officeDocument/2006/relationships/customXml";
    let rel_tag = format!(r#"<Relationship Id="{id}" Type="{rel_type}" Target="{target}"/>"#);

    // Inject it just before the closing </Relationships>
    let pos = xml
        .rfind("</Relationships>")
        .ok_or_eyre("word/_rels/document.xml.rels is missing </Relationships> tag")?;

    let mut out = String::with_capacity(xml.len() + rel_tag.len());
    out.push_str(&xml[..pos]);
    out.push_str(&rel_tag);
    out.push_str(&xml[pos..]);
    Ok(out)
}

/// Add `new_props` (name‒value pairs) to the XML string from
/// `docProps/custom.xml`, using nothing but string operations.
///
/// * Appends a `<property …>` node for every pair just before
///   `</Properties>`.
/// * Generates sequential `pid`s (one higher than the current max).
/// * Escapes `& < > ' "` in both name and value.
///
/// If `new_props` is empty the original `xml` is returned unchanged.
pub fn add_custom_props(xml: &str, new_props: Vec<(String, String)>) -> Result<String> {
    if new_props.is_empty() {
        return Ok(xml.to_owned());
    }

    // Find highest existing pid="…"
    let pid_rx = Regex::new(r#"pid="(\d+)""#)?;
    let max_pid = pid_rx
        .captures_iter(xml)
        .filter_map(|c| c.get(1).and_then(|m| m.as_str().parse::<u32>().ok()))
        .max()
        .unwrap_or(1);
    let mut next_pid = max_pid + 1;

    // Build the block of new <property/> elements
    let mut block = String::new();
    for (name, value) in new_props {
        block.push_str(&format!(
            r#"<property pid="{}" fmtid="{{D5CDD505-2E9C-101B-9397-08002B2CF9AE}}" name="{}"><vt:lpwstr>{}</vt:lpwstr></property>"#,
            next_pid,
            escape_xml(&name),
            escape_xml(&value)
        ));
        next_pid += 1;
    }

    let mut out = String::with_capacity(xml.len() + block.len());
    if let Some(pos) = xml.rfind("</Properties>") {
        // Splice the block in before existing </Properties>
        out.push_str(&xml[..pos]);
        out.push_str(&block);
        out.push_str(&xml[pos..]);
    } else if let Some(pos) = xml.rfind("/>") {
        // No, existing custom properties, so replace the end of the self-closing tag
        out.push_str(&xml[..pos]);
        out.push('>');
        out.push_str(&block);
        out.push_str("</Properties>");
    } else {
        // Fallback to constructing new XML
        out = format!(
            r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Properties xmlns="http://schemas.openxmlformats.org/officeDocument/2006/custom-properties" xmlns:vt="http://schemas.openxmlformats.org/officeDocument/2006/docPropsVTypes">{block}</Properties>"#
        );
    }

    Ok(out)
}

/// Escape XML.
///
/// Replaces the five XML-sensitive characters with their corresponding
/// entity references.
///
/// | character | entity  |
/// |-----------|---------|
/// | `&`       | `&amp;` |
/// | `<`       | `&lt;`  |
/// | `>`       | `&gt;`  |
/// | `"`       | `&quot;`|
/// | `'`       | `&apos;`|
pub fn escape_xml(input: &str) -> String {
    // Pre-allocate slightly more than the input length to avoid
    // frequent reallocations for typical “few escapables” cases.
    let mut out = String::with_capacity(input.len() + 8);

    for ch in input.chars() {
        match ch {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            '\'' => out.push_str("&apos;"),
            _ => out.push(ch),
        }
    }

    out
}
