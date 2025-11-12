mod embed;
mod extract;
mod extract_and_collect;

pub use embed::{embed_audio, embed_image, embed_media, embed_video};
pub use extract::extract_media;
pub use extract_and_collect::{MediaFile, extract_and_collect_media};
