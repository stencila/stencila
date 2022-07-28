use codec::{utils::vec_string, CodecTrait};
use codec_org::OrgCodec;
use test_props::{article, proptest::prelude::*, Freedom};
use test_utils::{
    assert_json_eq,
    common::{once_cell::sync::Lazy, tokio},
};

static RUNTIME: Lazy<tokio::runtime::Runtime> =
    Lazy::new(|| tokio::runtime::Runtime::new().unwrap());

proptest! {
    #![proptest_config(ProptestConfig::with_cases(30))]

    #[test]
    fn test(input in article(
        Freedom::Min,
        [
            OrgCodec::spec().unsupported_types,
            // The following types must be excluded because of Pandoc's parsing of org
            vec_string![
                // Pandoc does not seem to support headings greater than level 3 (it treats them as list items)
                "Heading",
                // Org mode does not allow inline code without whitespace e.g. `some=code=inline` (as can be generated here)
                "CodeFragment"
            ]
        ].concat(),
        OrgCodec::spec().unsupported_properties,
    )) {
        RUNTIME.block_on(async {
            let org = OrgCodec::to_string_async(&input, None).await.unwrap();
            let output = OrgCodec::from_str_async(&org, None).await.unwrap();
            assert_json_eq!(input, output)
        })
    }
}
