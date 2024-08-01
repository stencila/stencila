use std::{
    fs::read_to_string,
    io::Cursor,
    path::{Path, PathBuf},
};

use base64::{engine::general_purpose::STANDARD, Engine as _};
use common::eyre::{bail, Result};
use image::{ImageFormat, ImageReader};
use mime_guess::from_path;

/**
 * Covert an image URL to a HTTP or data URI
 */
pub fn ensure_http_or_data_uri(url: &str) -> Result<String> {
    if url.starts_with("http://") || url.starts_with("https://") || url.starts_with("data:image/") {
        return Ok(url.into());
    }

    let path = url.strip_prefix("file://").unwrap_or(url);
    let path = PathBuf::from(path);

    path_to_data_uri(&path)
}

/**
 * Convert an image path into a data URI
 */
pub fn path_to_data_uri(path: &Path) -> Result<String> {
    let mime_type = from_path(path).first_or_octet_stream();

    if mime_type.type_() != mime::IMAGE {
        bail!("Path is not an image: {}", path.display())
    }

    let encoded = if mime_type.subtype() == mime::SVG {
        // Plain text images
        STANDARD.encode(read_to_string(path)?)
    } else {
        // Binary images
        let image = ImageReader::open(path)?.decode()?;
        let mut bytes: Vec<u8> = Vec::new();
        image.write_to(&mut Cursor::new(&mut bytes), ImageFormat::Png)?;
        STANDARD.encode(&bytes)
    };

    Ok(format!("data:{};base64,{}", mime_type, encoded))
}
