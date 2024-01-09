use std::path::{Path, PathBuf};

use assistant::{
    common::{
        eyre::{Report, Result},
        futures::future::try_join_all,
        glob::glob,
        itertools::Itertools,
        tracing,
    },
    GenerateOptions,
};
use sea_orm::{
    ActiveValue, ConnectOptions, ConnectionTrait, Database, DatabaseBackend, EntityTrait, Statement,
};

use super::testing_db::{prelude::*, *};

/// Add a new assistant testing trial
pub async fn insert_trial(user_instruction: &str, assistant_response: &str) -> Result<()> {
    let trial = trials::ActiveModel {
        user_instruction: ActiveValue::Set(user_instruction.to_string()),
        assistant_response: ActiveValue::Set(assistant_response.to_string()),
        ..Default::default()
    };

    let mut options = ConnectOptions::new("sqlite://testing.sqlite3");
    options.sqlx_logging(false);

    let db = Database::connect(options).await?;
    db.execute(Statement::from_string(
        DatabaseBackend::Sqlite,
        include_str!("./testing_db/schema.sql"),
    ))
    .await?;

    Trials::insert(trial).exec(&db).await?;

    Ok(())
}

/// Run an example
///
/// Reads in one or more Markdown documents in a directory, executes the instructions
/// within them, and outputs a `.x.md` and a `.x.json` file of the executed document
/// for each repetition.
#[tracing::instrument]
pub async fn test_example(path: &Path, reps: u16) -> Result<()> {
    let paths = if path.is_dir() {
        glob(&path.join("*.md").to_string_lossy())?
            .flatten()
            .filter(|path| path.to_string_lossy().chars().filter(|&c| c == '.').count() == 1)
            .map(PathBuf::from)
            .collect_vec()
    } else {
        vec![path.to_path_buf()]
    };

    let docs = paths.iter().map(|path| async {
        let doc = codecs::from_path(path, None).await?;

        let reps = (0..reps).map(|rep| {
            let path = path.clone();
            let mut doc = doc.clone();

            async move {
                crate::perform_document(&mut doc, &GenerateOptions::default()).await?;

                let mut md = path.clone();
                md.set_extension(format!("{rep}.md"));
                codecs::to_path(&doc, &md, None).await?;

                let mut json = path.clone();
                json.set_extension(format!("{rep}.json"));
                codecs::to_path(&doc, &json, None).await?;

                Ok::<(), Report>(())
            }
        });

        try_join_all(reps).await?;
        Ok::<(), Report>(())
    });

    try_join_all(docs).await?;
    Ok(())
}
