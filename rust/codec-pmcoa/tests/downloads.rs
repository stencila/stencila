use codec::{
    common::{eyre::Result, tokio},
    Codec,
};
use common_dev::insta::{assert_json_snapshot, assert_yaml_snapshot};

use codec_pmcoa::PmcOaCodec;

/// Download and decode from PMCID
#[tokio::test]
async fn downloads() -> Result<()> {
    for pmcid in ["PMC11518443"] {
        let (article, info) = PmcOaCodec.from_str(pmcid, None).await?;

        assert_json_snapshot!(format!("{pmcid}.json"), article);
        assert_yaml_snapshot!(format!("{pmcid}.decode.losses"), info.losses);
    }

    Ok(())
}
