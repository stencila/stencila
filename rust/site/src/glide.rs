use stencila_config::GlideConfig;

/// Render a Stencila Glide config as data attributes for the <body>
/// 
/// Only options that are not `None` are rendered as attributes and default
/// behavior is determined by the TypeScript in web/src/site/glide
pub(crate) fn render_glide(config: Option<&GlideConfig>) -> String {
    let mut attrs = String::new();

    if let Some(config) = config {
        attrs.push_str(match config.enabled {
            Some(true) => " data-stencila-glide=\"on\"",
            Some(false) => " data-stencila-glide=\"off\"",
            None => "",
        });

        if let Some(prefetch) = config.prefetch {
            attrs.push_str(&format!(r#" data-stencila-prefetch="{prefetch}""#));
        }

        if let Some(cache) = config.cache {
            attrs.push_str(&format!(r#" data-stencila-cache="{cache}""#));
        }
    }

    attrs
}
