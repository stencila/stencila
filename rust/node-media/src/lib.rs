mod collect;
mod embed;
mod extract;

pub use collect::collect_media;
pub use embed::{embed_audio, embed_image, embed_media, embed_video};
pub use extract::extract_media;
