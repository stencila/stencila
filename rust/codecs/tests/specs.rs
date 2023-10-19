use std::collections::BTreeMap;

use codec::{
    common::{itertools::Itertools, serde_json::json},
    CodecDirection,
};
use common_dev::insta::assert_json_snapshot;

/// Snapshot test of codec specs to test for regressions
#[test]
fn specs() {
    let mut specs = BTreeMap::new();
    let codecs = codecs::list();
    for codec in &codecs {
        let lossy_from_types = codec
            .lossy_types(Some(CodecDirection::Decode))
            .iter()
            .map(|node_type| node_type.to_string())
            .collect_vec();

        let lossy_to_types = codec
            .lossy_types(Some(CodecDirection::Encode))
            .iter()
            .map(|node_type| node_type.to_string())
            .collect_vec();

        specs.insert(
            codec.name(),
            json!({
                "status": codec.status(),
                "supports_from_formats": codec.supports_from_formats(),
                "supports_from_string": codec.supports_from_string(),
                "supports_from_path": codec.supports_from_path(),
                "lossy_from_types": lossy_from_types,
                "supports_to_formats": codec.supports_to_formats(),
                "supports_to_string": codec.supports_to_string(),
                "supports_to_path": codec.supports_to_path(),
                "lossy_to_types": lossy_to_types
            }),
        );
    }
    assert_json_snapshot!(specs)
}
