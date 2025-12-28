mod collect;
mod embed;
mod extract;

pub use collect::collect_media;
pub use embed::{
    embed_audio, embed_image, embed_image_with, embed_media, embed_media_with, embed_video,
};
pub use extract::extract_media;
pub use stencila_images::ImageResizeOptions;
