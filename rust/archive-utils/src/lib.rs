use std::{fs, io, path::Path};

#[allow(unused_imports)]
use common::{
    eyre::{bail, eyre, Result},
    tracing,
};
use path_utils::lexiclean::Lexiclean;

#[cfg(feature = "tar")]
pub use ::tar;

#[cfg(feature = "zip")]
pub use ::zip;

#[cfg(feature = "flate2")]
pub use ::flate2;

#[cfg(feature = "xz2")]
pub use ::xz2;

#[cfg(feature = "zstd")]
pub use ::zstd;

/// Extract an archive to a destination
#[allow(unused_variables)]
pub fn extract(archive: &Path, dest: &Path, strip: usize, subdir: Option<&str>) -> Result<()> {
    tracing::info!("Extracting `{}` to `{}`", archive.display(), dest.display());

    let ext = match archive.extension() {
        Some(ext) => ext.to_string_lossy().to_string(),
        None => {
            bail!(
                "Archive file name `{}` has no extension so I don't know how to extract from it",
                archive.display()
            );
        }
    };

    match ext.as_str() {
        #[cfg(feature = "zip")]
        "zip" => extract_zip(archive, dest, strip, subdir),
        #[cfg(feature = "tar")]
        _ => extract_tar(&ext, archive, dest, strip, subdir),
        #[cfg(not(feature = "tar"))]
        _ => bail!("Unhandled archive extension {}", ext),
    }
}

/// Extract a tar archive
#[cfg(feature = "tar")]
pub fn extract_tar(
    ext: &str,
    archive: &Path,
    dest: &Path,
    strip: usize,
    subdir: Option<&str>,
) -> Result<()> {
    use std::fs::create_dir_all;

    let file = fs::File::open(archive)?;
    let mut archive = tar::Archive::new(match ext {
        "tar" => Box::new(file) as Box<dyn io::Read>,
        #[cfg(feature = "tar-gz")]
        "gz" | "tgz" => Box::new(flate2::read::GzDecoder::new(file)),
        #[cfg(feature = "tar-xz")]
        "xz" => Box::new(xz2::read::XzDecoder::new(file)),
        #[cfg(feature = "tar-zstd")]
        "zst" | "zstd" => Box::new(zstd::stream::Decoder::new(file)),
        _ => bail!("Unhandled archive extension {}", ext),
    });

    let subdir: Vec<String> = match subdir {
        Some(subdir) => subdir.split('/').map(String::from).collect(),
        None => Vec::new(),
    };

    let mut count = 0;
    for mut entry in archive.entries()?.flatten() {
        let path = entry.path()?.display().to_string();
        let mut components: Vec<String> = path.split('/').map(String::from).collect();

        // Strip path components
        if strip > 0 {
            components.drain(0..strip);
        }

        // Only extract entries in specified subdirectories
        let mut extract = true;
        for dir in &subdir {
            if components.first() == Some(dir) {
                components.remove(0);
            } else {
                extract = false;
                break;
            }
        }
        if !extract {
            continue;
        }

        // Do not extract paths that traverse out of destination
        let out_path = dest.join(&components.join("/")).lexiclean();
        if out_path.strip_prefix(dest).is_err() {
            continue;
        }

        // Ensure the parent directories of the output path exist
        if let Some(parent) = out_path.parent() {
            create_dir_all(parent)?;
        }

        // Unpack to destination
        entry.unpack(&out_path)?;
        count += 1;
    }

    tracing::debug!("Extracted {} entries", count);
    Ok(())
}

/// Extract a zip archive
#[cfg(feature = "zip")]
pub fn extract_zip(archive: &Path, dest: &Path, strip: usize, subdir: Option<&str>) -> Result<()> {
    let file = fs::File::open(archive)?;
    let mut archive = zip::ZipArchive::new(file)?;

    let subdir: Vec<String> = match subdir {
        Some(subdir) => subdir.split('/').map(String::from).collect(),
        None => Vec::new(),
    };

    let mut count = 0;
    for index in 0..archive.len() {
        let mut entry = archive.by_index(index)?;
        let path = entry
            .enclosed_name()
            .ok_or_else(|| eyre!("Zip archive entry has no enclosed name"))?
            .display()
            .to_string();
        let mut components: Vec<String> = path.split('/').map(String::from).collect();

        // Strip path components
        if strip > 0 {
            components.drain(0..strip);
        }

        // Only extract entries in specified subdirectories
        let mut extract = true;
        for dir in &subdir {
            if components.first() == Some(dir) {
                components.remove(0);
            } else {
                extract = false;
                break;
            }
        }
        if !extract {
            continue;
        }

        // Do not extract paths that traverse out of destination
        let out_path = dest.join(&components.join("/")).lexiclean();
        if out_path.strip_prefix(dest).is_err() {
            continue;
        }

        // Unpack to destination
        if entry.is_file() {
            if let Some(parent) = out_path.parent() {
                fs::create_dir_all(parent)?;
            }
            let mut out_file = fs::File::create(out_path)?;
            std::io::copy(&mut entry, &mut out_file)?;
            count += 1;
        }
    }

    tracing::debug!("Extracted {} entries", count);
    Ok(())
}
