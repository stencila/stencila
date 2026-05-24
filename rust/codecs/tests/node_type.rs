use stencila_codecs::{Format, NodeType, node_type_from_format};

#[test]
fn maps_formats_to_schema_resource_node_types() {
    let cases = [
        (Format::Csv, Some(NodeType::Datatable)),
        (Format::Tsv, Some(NodeType::Datatable)),
        (Format::Xlsx, Some(NodeType::Datatable)),
        (Format::Python, Some(NodeType::SoftwareSourceCode)),
        (Format::TypeScript, Some(NodeType::SoftwareSourceCode)),
        (Format::Rust, Some(NodeType::SoftwareSourceCode)),
        (Format::Png, Some(NodeType::ImageObject)),
        (Format::Svg, Some(NodeType::ImageObject)),
        (Format::Mp3, Some(NodeType::AudioObject)),
        (Format::Mp4, Some(NodeType::VideoObject)),
        (Format::Markdown, None),
    ];

    for (format, node_type) in cases {
        assert_eq!(node_type_from_format(&format), node_type, "{format}");
    }
}
