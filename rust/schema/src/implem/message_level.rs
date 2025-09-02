use stencila_codec_info::MessageLevel as CodecMessageLevel;

use crate::MessageLevel;

impl From<MessageLevel> for CodecMessageLevel {
    fn from(val: MessageLevel) -> Self {
        match val {
            MessageLevel::Trace => CodecMessageLevel::Warning,
            MessageLevel::Debug => CodecMessageLevel::Warning,
            MessageLevel::Info => CodecMessageLevel::Warning,
            MessageLevel::Warning => CodecMessageLevel::Warning,
            MessageLevel::Error | MessageLevel::Exception => CodecMessageLevel::Error,
        }
    }
}
