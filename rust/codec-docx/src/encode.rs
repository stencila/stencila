use std::{
    fs::{write, File},
    io::{Cursor, Write},
    path::Path,
};

use codec::common::eyre::Result;
use zip::{write::SimpleFileOptions, ZipArchive, ZipWriter};

pub(super) fn embeddings_and_custom_properties(
    path: &Path,
    embeddings: &[(&str, &str)],
    properties: &[(&str, String)],
) -> Result<()> {
    if embeddings.is_empty() && properties.is_empty() {
        return Ok(());
    }

    let mut src = ZipArchive::new(File::open(&path)?)?;

    let mut buf = Cursor::new(Vec::new());
    let mut dst = ZipWriter::new(&mut buf);

    // Copy over all existing files except for those that will be modified
    let mut skip = vec![
        // TODO
        //"[Content_Types].xml",
        //"word/_rels/document.xml.rels",
    ];
    if !properties.is_empty() {
        skip.push("docProps/custom.xml");
    }

    for index in 0..src.len() {
        let file = src.by_index(index)?;
        let name = file.name().to_string();
        if skip.contains(&name.as_str()) {
            continue;
        }
        dst.raw_copy_file(file)?;
    }

    // Add each of the embeddings
    for (name, content) in embeddings {
        dst.start_file(
            ["word/embeddings/", name].concat(),
            SimpleFileOptions::default(),
        )?;
        dst.write_all(content.as_bytes())?;
    }

    if !properties.is_empty() {
        // Encode properties. See https://learn.microsoft.com/en-us/office/open-xml/word/how-to-set-a-custom-property-in-a-word-processing-document
        let mut xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<Properties
    xmlns="http://schemas.openxmlformats.org/officeDocument/2006/custom-properties"
    xmlns:vt="http://schemas.openxmlformats.org/officeDocument/2006/docPropsVTypes"
>"#
        .to_string();

        let mut pid = 2; // Note: intentionally starts at 2
        for (name, value) in properties {
            xml.push_str(&format!(r#"
                <property fmtid="{{D5CDD505-2E9C-101B-9397-08002B2CF9AE}}" pid="{pid}" name="{name}">
                    <vt:lpwstr>{value}</vt:lpwstr>
                </property>
            "#, name = escape_xml(name), value = escape_xml(value)));
            pid += 1;
        }

        xml.push_str("</Properties>");

        dst.start_file("docProps/custom.xml", SimpleFileOptions::default())?;
        dst.write_all(xml.as_bytes())?;
    }

    // Finish the zip and overwrite the path
    dst.finish()?;
    write(path, buf.into_inner())?;

    Ok(())
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
