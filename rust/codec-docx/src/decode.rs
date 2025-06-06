use std::{fs::File, io::Read, path::Path};

use roxmltree::Document;
use zip::ZipArchive;

use codec::{
    common::{eyre::Result, indexmap::IndexMap, serde_json},
    schema::Primitive,
};

/// Read custom properties from a DOCX into a map of [`Primitive`] nodes
/// for the `extra`, and other properties, of an `Article`
///
/// At the time of writing Pandoc does not support reading custom properties
/// (only "standard" ones) so this implements that.
/// See https://github.com/jgm/pandoc/issues/3034#issuecomment-460381664
pub(super) fn custom_properties(path: &Path) -> Result<IndexMap<String, Primitive>> {
    // Open the DOCX as a ZIP archive
    let file = File::open(path)?;
    let mut archive = ZipArchive::new(file)?;

    // Attempt to read the custom properties XML file. If it does not exist, then
    // there are no custom properties
    let Ok(mut file) = archive.by_name("docProps/custom.xml") else {
        return Ok(IndexMap::new());
    };
    let mut xml_content = String::new();
    file.read_to_string(&mut xml_content)?;

    // Parse the XML
    let doc = Document::parse(&xml_content)?;

    // Iterate over <property> elements
    let mut properties = IndexMap::new();
    for prop in doc.descendants().filter(|n| {
        n.has_tag_name((
            "http://schemas.openxmlformats.org/officeDocument/2006/custom-properties",
            "property",
        ))
    }) {
        if let Some(name) = prop.attribute("name") {
            // Find the first child in the vt namespace and get its text
            let value = prop
                .children()
                .find(|child| child.is_element() && child.tag_name().namespace() == Some("http://schemas.openxmlformats.org/officeDocument/2006/docPropsVTypes"))
                .and_then(|n| n.text())
                .unwrap_or("");

            // Deserialize as a primitive node, falling back to a string
            let value: Primitive = match serde_json::from_str(value) {
                Ok(primitive) => primitive,
                _ => Primitive::String(value.into()),
            };

            properties.insert(name.into(), value);
        }
    }

    Ok(properties)
}
