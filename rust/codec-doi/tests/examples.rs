use codec::common::{eyre::Result, tokio};
use common_dev::insta::assert_json_snapshot;

use codec_doi::DoiClient;

/// Decode DOIs into articles
#[tokio::test]
async fn examples() -> Result<()> {
    let client = DoiClient::new()?;

    for doi in ["10.48550/arxiv.2507.09057", "10.48550/arxiv.2507.11353"] {
        let csl = client.get(doi).await?;
        assert_json_snapshot!(format!("{doi}-csl"), csl);

        let article = csl.to_article()?;
        assert_json_snapshot!(format!("{doi}-article"), article);
    }

    Ok(())
}
