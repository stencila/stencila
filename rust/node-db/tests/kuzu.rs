//! Tests related to our use of Kuzu

use kuzu::{Connection, Database, SystemConfig};

use common::eyre::Result;

#[test]
fn create_fts_index() -> Result<()> {
    let database = Database::in_memory(SystemConfig::default())?;
    let connection = Connection::new(&database)?;

    connection.query(
        "
        INSTALL FTS;
        ",
    )?;

    connection.query(
        "
        LOAD EXTENSION FTS;
        ",
    )?;

    connection.query(
        "
        CREATE NODE TABLE IF NOT EXISTS `CodeBlock` (
            `docId` STRING,
            `nodeId` STRING PRIMARY KEY,
            `code` STRING,
            `programmingLanguage` STRING
        );
        ",
    )?;

    connection.query(
        "
        CREATE (:CodeBlock {
            docId: 'doc_1',
            nodeId: 'cbk_1',
            code: 'Hello world',
            programmingLanguage: 'python'
        });
        ",
    )?;

    connection.query(
        "
        CALL CREATE_FTS_INDEX('CodeBlock', 'code', ['code']);
        ",
    )?;

    Ok(())
}
