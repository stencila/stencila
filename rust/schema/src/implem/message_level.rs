use crate::MessageLevel;

impl Into<codec_info::MessageLevel> for MessageLevel {
    fn into(self) -> codec_info::MessageLevel {
        match self {
            MessageLevel::Trace => codec_info::MessageLevel::Warning,
            MessageLevel::Debug => codec_info::MessageLevel::Warning,
            MessageLevel::Info => codec_info::MessageLevel::Warning,
            MessageLevel::Warning => codec_info::MessageLevel::Warning,
            MessageLevel::Error | MessageLevel::Exception => codec_info::MessageLevel::Error,
        }
    }
}
