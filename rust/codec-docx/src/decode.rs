use std::{fs::File, io::Read, path::Path};

use roxmltree::Document;
use zip::ZipArchive;

use codec::{
    common::eyre::Result,
    schema::{Object, Primitive},
};

/// Read custom properties from a DOCX into an [`Object`] to use as
/// the `extra` property of an `Article`
///
/// At the time of writing Pandoc does not support reading custom properties
/// (only "standard" ones) so this implements that.
/// See https://github.com/jgm/pandoc/issues/3034#issuecomment-460381664
pub(super) fn custom_properties(path: &Path) -> Result<Option<Object>> {
    // Open the DOCX as a ZIP archive
    let file = File::open(path)?;
    let mut archive = ZipArchive::new(file)?;

    // Attempt to read the custom properties XML file. If it does not exists, then
    // there are no custom properties
    let Ok(mut file) = archive.by_name("docProps/custom.xml") else {
        return Ok(None);
    };
    let mut xml_content = String::new();
    file.read_to_string(&mut xml_content)?;

    // Parse the XML
    let doc = Document::parse(&xml_content)?;

    // Namespaces for elements
    let cp_ns = "http://schemas.openxmlformats.org/officeDocument/2006/custom-properties";
    let vt_ns = "http://schemas.openxmlformats.org/officeDocument/2006/docPropsVTypes";

    // Iterate over <property> elements
    let mut object = Object::default();
    for prop in doc
        .descendants()
        .filter(|n| n.has_tag_name((cp_ns, "property")))
    {
        if let Some(name) = prop.attribute("name") {
            // Find the first child in the vt namespace and get its text
            let value = prop
                .children()
                .find(|child| child.is_element() && child.tag_name().namespace() == Some(vt_ns))
                .and_then(|n| n.text())
                .unwrap_or("");

            object.insert(name.into(), Primitive::String(value.into()));
        }
    }

    Ok((!object.is_empty()).then_some(object))
}
