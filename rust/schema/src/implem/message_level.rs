use crate::MessageLevel;

impl From<MessageLevel> for codec_info::MessageLevel {
    fn from(val: MessageLevel) -> Self {
        match val {
            MessageLevel::Trace => codec_info::MessageLevel::Warning,
            MessageLevel::Debug => codec_info::MessageLevel::Warning,
            MessageLevel::Info => codec_info::MessageLevel::Warning,
            MessageLevel::Warning => codec_info::MessageLevel::Warning,
            MessageLevel::Error | MessageLevel::Exception => codec_info::MessageLevel::Error,
        }
    }
}
