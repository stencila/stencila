mod collect;
mod embed;
mod extract;
mod naming;
mod reference;

pub use collect::{collect_media, collect_media_with_paths};
pub use embed::{
    embed_audio, embed_image, embed_image_with, embed_media, embed_media_with, embed_video,
};
pub use extract::{extract_media, extract_media_with_paths};
pub use reference::reference_media_with_paths;
pub use stencila_images::ImageResizeOptions;
