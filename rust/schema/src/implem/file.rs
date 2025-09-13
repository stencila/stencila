use std::{fs, path::Path};

use base64::{Engine as _, engine::general_purpose::STANDARD};

use crate::{File, prelude::*};

impl File {
    /// Read a file from the file system
    pub fn read(path: &Path) -> Result<Self> {
        if !path.exists() {
            bail!("File does not exist: {}", path.display());
        }

        let name = path
            .file_name()
            .map(|name| name.to_string_lossy().to_string())
            .unwrap_or_else(|| "Unnamed".to_string());

        let format = Format::from_path(path);

        let media_type = Some(format.media_type());

        let content = if format.is_binary() {
            // For binary files, read as bytes and base64 encode
            let bytes = fs::read(path)?;
            Some(STANDARD.encode(&bytes))
        } else {
            // For text files, read as string
            let text = fs::read_to_string(path)?;
            Some(text)
        };

        let path = path.to_string_lossy().to_string();

        Ok(File {
            name,
            path,
            media_type,
            content,
            ..Default::default()
        })
    }

    /// Convert file content to a data URI
    pub fn to_data_uri(&self) -> Option<String> {
        let content = self.content.as_ref()?;
        let media_type = self
            .media_type
            .as_deref()
            .unwrap_or("application/octet-stream");

        // Check if the format is binary by checking the media type
        let is_binary = Format::from_media_type(media_type)
            .map(|format| format.is_binary())
            .unwrap_or(false);

        if is_binary {
            // Content is already base64 encoded for binary files
            Some(format!("data:{media_type};base64,{content}"))
        } else {
            // For text files, we need to base64 encode the content
            let encoded = STANDARD.encode(content.as_bytes());
            Some(format!("data:{media_type};base64,{encoded}"))
        }
    }
}
