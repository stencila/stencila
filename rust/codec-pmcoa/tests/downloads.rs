use codec::{
    Codec,
    common::{eyre::Result, tokio},
};
use common_dev::insta::{assert_json_snapshot, assert_yaml_snapshot};

use codec_pmcoa::PmcOaCodec;

/// Download and decode from PMCID Currently skipping this test because request
/// fails with 500 status code (including when using `xh` and Firefox, so not
/// just our client)
#[ignore = "frequently 500s"]
#[tokio::test]
#[allow(clippy::print_stderr)]
async fn downloads() -> Result<()> {
    if std::env::var("CI").is_ok() {
        eprintln!("Skipping network-dependent test on CI");
        return Ok(());
    }

    for pmcid in ["PMC11518443"] {
        let (article, info) = PmcOaCodec.from_str(pmcid, None).await?;

        assert_json_snapshot!(format!("{pmcid}.json"), article);
        assert_yaml_snapshot!(format!("{pmcid}.decode.losses"), info.losses);
    }

    Ok(())
}
