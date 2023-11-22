use std::collections::BTreeMap;

use codec::common::serde_json::json;
use common_dev::insta::assert_json_snapshot;

/// Snapshot test of codec specs to test for regressions
#[test]
fn specs() {
    let mut specs = BTreeMap::new();
    let codecs = codecs::list();
    for codec in &codecs {
        specs.insert(
            codec.name(),
            json!({
                "status": codec.status(),

                "supports_from_formats": codec.supports_from_formats(),
                "supports_from_bytes": codec.supports_from_bytes(),
                "supports_from_string": codec.supports_from_string(),
                "supports_from_path": codec.supports_from_path(),

                "supports_to_formats": codec.supports_to_formats(),
                "supports_to_bytes": codec.supports_to_bytes(),
                "supports_to_string": codec.supports_to_string(),
                "supports_to_path": codec.supports_to_path(),
            }),
        );
    }
    assert_json_snapshot!(specs)
}
