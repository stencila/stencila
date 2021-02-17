use strum::{Display, EnumString, EnumVariantNames};

#[derive(Debug, Display, EnumString, EnumVariantNames, PartialEq)]
#[strum(serialize_all = "lowercase")]
pub enum Protocol {
    #[cfg(any(feature = "delegate-stdio", feature = "serve-stdio"))]
    Stdio,
    #[cfg(any(feature = "delegate-http", feature = "serve-http"))]
    Http,
    #[cfg(any(feature = "delegate-ws", feature = "serve-ws"))]
    Ws,
}
