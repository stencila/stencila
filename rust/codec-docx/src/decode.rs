use std::{fs::File, io::Read, path::Path};

use roxmltree::Document;
use zip::ZipArchive;

use codec::{
    common::{
        eyre::{eyre, Context, Result},
        indexmap::IndexMap,
        serde_json,
    },
    schema::Primitive,
};

/// Read embedded files and custom properties from a DOCX into a map of [`Primitive`] nodes
/// for the `extra`, and other properties, of an `Article`
///
/// At the time of writing Pandoc does not support reading custom properties
/// (only "standard" ones) so this implements that.
/// See https://github.com/jgm/pandoc/issues/3034#issuecomment-460381664
pub(super) fn data_and_properties(
    path: &Path,
) -> Result<(IndexMap<String, String>, IndexMap<String, Primitive>)> {
    let mut docx =
        File::open(path).wrap_err_with(|| eyre!("unable to open: {}", path.display()))?;
    let mut zip = ZipArchive::new(&mut docx)
        .wrap_err_with(|| eyre!("DOCX is not a valid zip: {}", path.display()))?;

    let mut data = IndexMap::new();
    let mut properties = IndexMap::new();

    for index in 0..zip.len() {
        let mut part = zip.by_index(index)?;
        let name = part.name().to_owned();

        let is_custom_props = name == "docProps/custom.xml";
        let is_custom_data =
            name.starts_with("customXml/") && name.ends_with(".xml") && !name.contains("Props");

        // Skip irrelevant files
        if !is_custom_props && !is_custom_data {
            continue;
        }

        // Read the relevant part to a UTF-8 string exactly once.
        let mut xml = String::new();
        part.read_to_string(&mut xml)?;

        if is_custom_props {
            if let Ok(doc) = Document::parse(&xml) {
                for prop in doc.descendants().filter(|node| {
                    node.has_tag_name((
                        "http://schemas.openxmlformats.org/officeDocument/2006/custom-properties",
                        "property",
                    ))
                }) {
                    if let Some(name) = prop.attribute("name") {
                        // Find the first child in the vt namespace and get its text
                        let value = prop
                            .children()
                            .find(|child| child.tag_name().namespace() == Some(
                                "http://schemas.openxmlformats.org/officeDocument/2006/docPropsVTypes"
                            ))
                            .and_then(|child| child.text())
                            .unwrap_or("");

                        // Deserialize as a primitive node, falling back to a string
                        let value: Primitive = match serde_json::from_str(value) {
                            Ok(primitive) => primitive,
                            _ => Primitive::String(value.into()),
                        };

                        properties.insert(name.into(), value);
                    }
                }
            }
            continue;
        }

        // From here on we know it's a customXml data part.
        if let Ok(doc) = Document::parse(&xml) {
            let root = doc.root_element();
            if root.tag_name().name() == "data" {
                if let Some(name) = root.attribute("name") {
                    if let Some(payload) = root.text() {
                        data.insert(name.into(), payload.into());
                    }
                }
            }
        }
    }

    Ok((data, properties))
}
