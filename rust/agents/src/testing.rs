use std::{fs::File, io::Write, path::Path};

use agent::{
    common::{
        eyre::{eyre, Result},
        futures::future::try_join_all,
        serde_json, serde_yaml, tracing,
    },
    GenerateDetails, GenerateOptions, GenerateTask,
};
use sea_orm::{
    ActiveValue, ConnectOptions, ConnectionTrait, Database, DatabaseBackend, EntityTrait, Statement,
};

use super::testing_db::{prelude::*, *};

/// Add a new agent testing trial
pub async fn insert_trial(
    user_instruction: &str,
    agent_response: &str,
    details: GenerateDetails,
) -> Result<()> {
    let trial = trials::ActiveModel {
        user_instruction: ActiveValue::Set(user_instruction.to_string()),
        agent_response: ActiveValue::Set(agent_response.to_string()),
        generate_detail: ActiveValue::Set(serde_json::to_string(&details)?),
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
#[tracing::instrument]
pub async fn test_example(path: &Path, instruction: &str, reps: u16) -> Result<()> {
    // Read instruction
    let instruct_file = File::open(path.join(format!("{instruction}.yaml")))
        .map_err(|error| eyre!("unable to read {instruction}.yaml: {error}"))?;
    let instruct = serde_yaml::from_reader(instruct_file)?;

    // Read document
    let document = path.join("document.md");
    let document = if document.exists() {
        Some(codecs::from_path(&document, None).await?)
    } else {
        None
    };

    // Create a task from the instruction and document
    let task = GenerateTask::new(instruct, document);

    // Run repetitions in parallel
    let tasks = (0..reps).map(|_| {
        let task = task.clone();
        async { crate::generate_content(task, &GenerateOptions::default()).await }
    });
    let results = try_join_all(tasks).await?;

    // Create output file
    let mut output = File::create(path.join(format!("{instruction}.md")))?;
    for (index, (content, details)) in results.iter().enumerate() {
        output.write_all(
            if index == 0 {
                format!("---\n{}\n---\n\n", serde_yaml::to_string(details)?)
            } else {
                "\n\n---\n\n".to_string()
            }
            .as_bytes(),
        )?;
        output.write_all(content.as_bytes())?;
    }

    Ok(())
}
