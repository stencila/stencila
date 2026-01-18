use std::{
    fs::{File, copy, create_dir_all, read_to_string},
    hash::{Hash, Hasher},
    io::{Cursor, Write},
    path::{Path, PathBuf},
    sync::LazyLock,
};

use base64::{Engine as _, engine::general_purpose::STANDARD};
use eyre::{OptionExt, Result, bail};
use image::{
    DynamicImage, ExtendedColorType, GenericImageView, ImageEncoder, ImageFormat, ImageReader,
    codecs::png::{CompressionType, FilterType, PngEncoder},
    imageops, open,
};
use itertools::Itertools;
use mime_guess::from_path;
use regex::{Captures, Regex};
use seahash::SeaHasher;

/// Options for image processing (resizing, compression, etc.)
#[derive(Clone, Debug)]
pub struct ImageResizeOptions {
    /// Maximum width in pixels (images wider than this will be scaled down).
    /// None means no limit.
    pub max_width: Option<u32>,

    /// Scale up images smaller than max_width to reach max_width.
    pub upscale: bool,

    /// Use best compression for smallest file size (slower encoding).
    /// When false, uses default/fast compression.
    pub best_compression: bool,

    /// Use JPEG for opaque images (no alpha channel) for smaller file size.
    /// Images with transparency will still use PNG.
    pub jpeg_for_opaque: bool,

    /// JPEG quality (1-100, default 85).
    /// Only used when jpeg_for_opaque is true.
    pub jpeg_quality: u8,
}

impl Default for ImageResizeOptions {
    fn default() -> Self {
        Self {
            max_width: None,
            upscale: false,
            best_compression: false,
            jpeg_for_opaque: false,
            jpeg_quality: 85,
        }
    }
}

impl ImageResizeOptions {
    /// Options optimized for email (600px max width, best PNG compression)
    pub fn for_email() -> Self {
        Self {
            max_width: Some(600),
            upscale: false,
            best_compression: true,
            jpeg_for_opaque: false,
            jpeg_quality: 85,
        }
    }

    /// Options for web viewing (1200px max width, PNG, fast compression)
    pub fn for_web() -> Self {
        Self {
            max_width: Some(1200),
            upscale: false,
            best_compression: false,
            jpeg_for_opaque: false,
            jpeg_quality: 85,
        }
    }

    /// Options optimized for DOCX output (800px max width, JPEG for opaque images)
    pub fn for_docx() -> Self {
        Self {
            max_width: Some(800),
            upscale: false,
            best_compression: true,
            jpeg_for_opaque: true,
            jpeg_quality: 85,
        }
    }

    /// Encode a DynamicImage to bytes using these options
    ///
    /// Returns (bytes, mime_type). Uses JPEG for opaque images (no alpha)
    /// if `jpeg_for_opaque` is true, otherwise PNG.
    pub fn encode_image(&self, img: &DynamicImage) -> Result<(Vec<u8>, &'static str)> {
        let has_alpha = img.color().has_alpha();

        if self.jpeg_for_opaque && !has_alpha {
            // Use JPEG for non-transparent images
            let rgb = img.to_rgb8();
            let mut bytes: Vec<u8> = Vec::new();
            let mut encoder =
                image::codecs::jpeg::JpegEncoder::new_with_quality(&mut bytes, self.jpeg_quality);
            encoder.encode_image(&rgb)?;
            Ok((bytes, "image/jpeg"))
        } else {
            // Use PNG (required for transparency)
            let bytes = self.encode_png(img)?;
            Ok((bytes, "image/png"))
        }
    }

    /// Encode a DynamicImage to PNG bytes using these options
    ///
    /// Detects grayscale images (including RGB images where R=G=B) and
    /// encodes them efficiently as Luma8 for smaller file size.
    fn encode_png(&self, img: &DynamicImage) -> Result<Vec<u8>> {
        let mut bytes: Vec<u8> = Vec::new();
        let (width, height) = img.dimensions();
        let has_alpha = img.color().has_alpha();

        // Check if image is grayscale (native or effective RGB where R=G=B)
        let is_grayscale = matches!(
            img,
            DynamicImage::ImageLuma8(_) | DynamicImage::ImageLuma16(_)
        ) || (!has_alpha && is_effectively_grayscale(img));

        /// Check if an RGB image is effectively grayscale (R=G=B for all pixels)
        fn is_effectively_grayscale(img: &DynamicImage) -> bool {
            let rgb = img.to_rgb8();
            rgb.pixels().all(|p| p.0[0] == p.0[1] && p.0[1] == p.0[2])
        }

        let (image_bytes, color_type): (Vec<u8>, ExtendedColorType) = if is_grayscale && has_alpha {
            (img.to_luma_alpha8().into_raw(), ExtendedColorType::La8)
        } else if is_grayscale {
            (img.to_luma8().into_raw(), ExtendedColorType::L8)
        } else if has_alpha {
            (img.to_rgba8().into_raw(), ExtendedColorType::Rgba8)
        } else {
            (img.to_rgb8().into_raw(), ExtendedColorType::Rgb8)
        };

        if self.best_compression {
            let encoder = PngEncoder::new_with_quality(
                &mut bytes,
                CompressionType::Best,
                FilterType::Adaptive,
            );
            encoder.write_image(&image_bytes, width, height, color_type)?;
        } else {
            let encoder = PngEncoder::new(&mut bytes);
            encoder.write_image(&image_bytes, width, height, color_type)?;
        }

        Ok(bytes)
    }
}

/// Resize a DynamicImage according to the specified options
///
/// Returns the original image unchanged if no resizing is needed.
/// Uses Lanczos3 filter for high-quality downscaling.
/// Preserves the original color type (RGB vs RGBA) to avoid bloating file size.
pub fn resize_image(img: DynamicImage, options: &ImageResizeOptions) -> DynamicImage {
    let Some(max_width) = options.max_width else {
        return img;
    };

    let (width, height) = img.dimensions();
    let needs_resize = width > max_width;
    let needs_upscale = options.upscale && width < max_width;

    if !needs_resize && !needs_upscale {
        return img;
    }

    // Check if the original image has an alpha channel
    let has_alpha = matches!(
        img,
        DynamicImage::ImageRgba8(_)
            | DynamicImage::ImageRgba16(_)
            | DynamicImage::ImageRgba32F(_)
            | DynamicImage::ImageLumaA8(_)
            | DynamicImage::ImageLumaA16(_)
    );

    let aspect_ratio = height as f64 / width as f64;
    let new_height = (max_width as f64 * aspect_ratio).round() as u32;

    // resize() returns RGBA8, convert back to RGB8 if original had no alpha
    let resized = imageops::resize(&img, max_width, new_height, imageops::FilterType::Lanczos3);

    if has_alpha {
        DynamicImage::ImageRgba8(resized)
    } else {
        DynamicImage::ImageRgb8(DynamicImage::ImageRgba8(resized).to_rgb8())
    }
}

/// Resize a data URI image if it exceeds max_width
///
/// This function decodes a base64 data URI, checks the image dimensions,
/// resizes if necessary, and re-encodes as a data URI.
///
/// # Arguments
/// * `data_uri` - A base64 encoded data URI (e.g., "data:image/png;base64,...")
/// * `options` - Image processing options including max_width
///
/// # Returns
/// * `Result<String>` - The (potentially resized) image as a data URI
pub fn resize_data_uri(data_uri: &str, options: &ImageResizeOptions) -> Result<String> {
    // If no max_width specified, return unchanged
    if options.max_width.is_none() {
        return Ok(data_uri.to_string());
    }

    // Parse the data URI header
    let Some((header, data)) = data_uri.split_once(',') else {
        bail!("Invalid data URI format: missing comma separator");
    };

    // Extract MIME type
    let mime_type = header
        .split(';')
        .next()
        .and_then(|s| s.strip_prefix("data:"))
        .ok_or_eyre("Invalid data URI header")?;

    // SVG images don't need raster resizing
    if mime_type == "image/svg+xml" {
        return Ok(data_uri.to_string());
    }

    // Decode base64 to bytes
    let image_bytes = STANDARD.decode(data)?;

    // Load image from bytes
    let img = ImageReader::new(Cursor::new(&image_bytes))
        .with_guessed_format()?
        .decode()?;

    let original_dimensions = img.dimensions();

    // Use shared resize function
    let resized = resize_image(img, options);

    // If dimensions unchanged, return original data URI (avoids re-encoding)
    if resized.dimensions() == original_dimensions {
        return Ok(data_uri.to_string());
    }

    // Encode to bytes - use encode_image which respects prefer_jpeg option
    let (output_bytes, output_mime) = options.encode_image(&resized)?;

    // Re-encode to base64
    let encoded = STANDARD.encode(&output_bytes);

    Ok(format!("data:{output_mime};base64,{encoded}"))
}

/// Load an image from a file path, resize it, and encode as a data URI
///
/// Combines file loading with resizing options for use cases like email encoding
/// where images need to be both embedded and optimized.
///
/// # Arguments
/// * `path` - Path to the image file
/// * `options` - Image processing options including max_width
///
/// # Returns
/// * `Result<String>` - The resized image as a data URI
pub fn resize_file_to_data_uri(path: &Path, options: &ImageResizeOptions) -> Result<String> {
    let mime_type = from_path(path).first_or_octet_stream();

    if mime_type.type_() != mime::IMAGE {
        bail!("Path is not an image: {}", path.display())
    }

    // SVG images don't need raster resizing
    if mime_type.subtype() == mime::SVG {
        let svg_content = read_to_string(path)?;
        let encoded = STANDARD.encode(&svg_content);
        return Ok(format!("data:{mime_type};base64,{encoded}"));
    }

    // Load and resize the image
    let img = ImageReader::open(path)?.decode()?;
    let resized = resize_image(img, options);

    // Encode using options (handles grayscale detection, compression, etc.)
    let (bytes, output_mime) = options.encode_image(&resized)?;
    let encoded = STANDARD.encode(&bytes);

    Ok(format!("data:{output_mime};base64,{encoded}"))
}

/// Resize an image file in place, potentially converting to JPEG if opaque
///
/// Returns the final path (may differ from input if format changed to JPEG)
pub fn resize_and_save_file(path: &Path, options: &ImageResizeOptions) -> Result<PathBuf> {
    let img = ImageReader::open(path)?.decode()?;
    let resized = resize_image(img, options);
    let (bytes, mime_type) = options.encode_image(&resized)?;

    let output_path = if mime_type == "image/jpeg" {
        path.with_extension("jpg")
    } else {
        path.to_path_buf()
    };

    std::fs::write(&output_path, bytes)?;

    // Remove original if different from output
    if output_path != path && path.exists() {
        let _ = std::fs::remove_file(path);
    }

    Ok(output_path)
}

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
    static REGEX: LazyLock<Regex> = LazyLock::new(|| {
        Regex::new(r#"(<img[^>]*\s)src=["']([^"']+)["']"#).expect("invalid regex")
    });

    REGEX
        .replace_all(html, |caps: &Captures| {
            let prefix = &caps[1]; // Everything before the src attribute
            let src = &caps[2]; // The src value
            let new_src = transform(src);
            format!(r#"{prefix}src="{new_src}""#)
        })
        .into_owned()
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
