use std::{
    fs::{File, copy, create_dir_all, read_to_string},
    hash::{Hash, Hasher},
    io::{Cursor, Write},
    path::{Path, PathBuf},
};

use base64::{Engine as _, engine::general_purpose::STANDARD};
use common::{
    eyre::{OptionExt, Result, bail},
    itertools::Itertools,
    once_cell::sync::Lazy,
    regex::{Captures, Regex},
    seahash::SeaHasher,
};
use image::{GenericImage, GenericImageView, ImageBuffer, ImageFormat, ImageReader, Rgba, open};
use mime_guess::from_path;

/// Covert an image URL to a HTTP or data URI
///
/// URL beginning with `http://`, `https://`, or `data:` will be returned unchanged.
/// Other URLs, including those beginning with `file://`, are assumed to be filesystem
/// path and will be converted to a sata URI.
pub fn ensure_http_or_data_uri(url: &str) -> Result<String> {
    if url.starts_with("http://") || url.starts_with("https://") || url.starts_with("data:") {
        return Ok(url.into());
    }

    let path = url.strip_prefix("file://").unwrap_or(url);
    let path = PathBuf::from(path);

    path_to_data_uri(&path)
}

/// Convert a filesystem path to an image into a data URI
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

    Ok(format!("data:{mime_type};base64,{encoded}"))
}

/// Convert a data URI into an image file
///
/// The image will be converted into an image file with a name
/// based on the hash of the URI and an extension based on the
/// type of data URI.
///
/// # Arguments
///
/// - `data_uri`: the data URI
/// - `images_dir`: the destination images directory
///
/// # Returns
///
/// The file name of the image within `images_dir`.
pub fn data_uri_to_file(data_uri: &str, images_dir: &Path) -> Result<String> {
    // Parse the data URI
    let Some((header, data)) = data_uri.split(',').collect_tuple() else {
        bail!("Invalid data URI format");
    };

    // Extract the MIME type
    let mime_type = header
        .split(';')
        .next()
        .and_then(|mime_type| mime_type.strip_prefix("data:"))
        .ok_or_eyre("Invalid data URI header")?;

    // Determine the file extension based on the MIME type
    let extension = match mime_type {
        "image/jpeg" => "jpg",
        "image/png" => "png",
        "image/gif" => "gif",
        "image/webp" => "webp",
        "image/svg+xml" => "svg",
        _ => bail!("Unsupported image format: {}", mime_type),
    };

    // Decode the Base64 data
    let decoded_data = STANDARD.decode(data.as_bytes())?;

    // Generate a hash of the data URI
    let mut hash = SeaHasher::new();
    data_uri.hash(&mut hash);
    let hash = hash.finish();
    let image_name = format!("{hash:x}.{extension}");

    // Ensure the images directory exists
    if !images_dir.exists() {
        create_dir_all(images_dir)?;
    }

    // Write the decoded data to the file
    let mut file = File::create(images_dir.join(&image_name))?;
    file.write_all(&decoded_data)?;

    Ok(image_name)
}

/// Convert a file URI to a filesystem path to an image
///
/// The absolute path of the source image will be resolved
/// from `file_uri` and `src_path` and the image copied to `images_dir`.
///
/// # Arguments
///
/// - `file_uri`: an absolute or relative filesystem path, which may be prefixed with `file://`
/// - `src_path`: the path that any relative paths are relative to
/// - `images_dir`: the destination images directory
///
/// # Returns
///
/// The file name of the image within `images_dir`.
pub fn file_uri_to_file(
    file_uri: &str,
    src_path: Option<&Path>,
    images_dir: &Path,
) -> Result<String> {
    // Handle the file URI, stripping the "file://" prefix if present
    let path_str = file_uri.strip_prefix("file://").unwrap_or(file_uri);
    let path = PathBuf::from(path_str);

    // Resolve the src path
    let src_path = if path.is_absolute() {
        path
    } else {
        match src_path {
            Some(src) => {
                // If the src path is empty then it implies current dir
                let src = if src == PathBuf::new() {
                    std::env::current_dir()?
                } else {
                    src.to_path_buf()
                };

                if src.is_dir() {
                    src.join(path)
                } else {
                    src.parent()
                        .map(|parent| parent.join(path))
                        .unwrap_or_else(|| src.to_path_buf())
                }
            }
            None => std::env::current_dir()?.join(path),
        }
    };

    // Ensure the source file exists
    if !src_path.exists() {
        bail!("Source file does not exist: {:?}", src_path);
    }

    // Generate a unique filename for the image
    let mut hash = SeaHasher::new();
    src_path.hash(&mut hash);
    let hash = hash.finish();
    let ext = src_path
        .extension()
        .ok_or_eyre("Invalid source file name")?;
    let image_name = format!("{:x}.{}", hash, ext.to_string_lossy());

    // Ensure the images directory exists
    if !images_dir.exists() {
        create_dir_all(images_dir)?;
    }

    // Copy the file to the images directory
    copy(&src_path, images_dir.join(&image_name))?;

    Ok(image_name)
}

/// Transform all the <img> `src` attributes in a string, which are not HTTP, to paths
pub fn img_srcs_to_paths(
    html: &str,
    src_path: Option<&Path>,
    dest_path: Option<&Path>,
    images_dir: &Path,
) -> String {
    img_srcs_transform(html, |src| {
        if src.starts_with("http://") || src.starts_with("https://") {
            return src.to_string();
        }

        match file_uri_to_file(src, src_path, images_dir) {
            Ok(image_name) => {
                let image_path = images_dir.join(image_name);
                match dest_path {
                    Some(to_path) => to_path
                        .parent()
                        .and_then(|dir| image_path.strip_prefix(dir).ok())
                        .map(PathBuf::from)
                        .unwrap_or(image_path),
                    None => image_path,
                }
                .to_string_lossy()
                .to_string()
            }
            Err(..) => src.to_string(),
        }
    })
}

/// Replace the `src` attributes of <img> tags using a transformation function
fn img_srcs_transform(html: &str, transform: impl Fn(&str) -> String) -> String {
    static REGEX: Lazy<Regex> =
        Lazy::new(|| Regex::new(r#"(<img[^>]*\s)src=["']([^"']+)["']"#).expect("invalid regex"));

    REGEX
        .replace_all(html, |caps: &Captures| {
            let prefix = &caps[1]; // Everything before the src attribute
            let src = &caps[2]; // The src value
            let new_src = transform(src);
            format!(r#"{prefix}src="{new_src}""#)
        })
        .into_owned()
}

/// Highlight an image by placing a green border around it
///
/// This border width is 0.5% of the maximum of the image's hight or width.
pub fn highlight_image(path: &Path) -> Result<()> {
    let img = open(path)?;

    let border_color = Rgba([0, 255, 0, 255]);

    let (w, h) = img.dimensions();
    let border_width = (w.min(h) / 200).max(2);
    let new_w = w + 2 * border_width;
    let new_h = h + 2 * border_width;

    // Create a new image and fill it with the border color and overlay the original image in the center
    let mut bordered = ImageBuffer::from_pixel(new_w, new_h, border_color);
    bordered.copy_from(&img.to_rgba8(), border_width, border_width)?;

    bordered.save(path)?;

    Ok(())
}

/// Convert image from one format to another
pub fn convert(from: &Path, to: &Path) -> Result<()> {
    let img = open(from)?;
    img.save(to)?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_img_srcs_transform() {
        let html = r#"
            <div>
                <img src="path/to/image.jpg" alt="test">
                <img class="test" src='another/image.png'>
                <p>Some text</p>
                <img>
                <img src="path/with spaces.jpg">
            </div>
        "#;
        let result = img_srcs_transform(html, |src: &str| format!("/converted/{src}"));

        assert!(result.contains(r#"src="/converted/path/to/image.jpg""#));
        assert!(result.contains(r#"src="/converted/another/image.png""#));
        assert!(result.contains(r#"src="/converted/path/with spaces.jpg""#));

        let html = r#"<img class="test" src="path.jpg" alt="test">"#;
        let result = img_srcs_transform(html, |src: &str| format!("/converted/{src}"));

        assert!(result.contains(r#"class="test""#));
        assert!(result.contains(r#"alt="test""#));
    }
}
