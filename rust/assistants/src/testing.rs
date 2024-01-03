use std::{fs::File, io::Write, path::Path};

use assistant::{
    common::{
        eyre::{eyre, Result},
        futures::future::try_join_all,
        serde_json, serde_yaml, tracing,
    },
    GenerateDetails, GenerateOptions, Instruction,
};
use sea_orm::{
    ActiveValue, ConnectOptions, ConnectionTrait, Database, DatabaseBackend, EntityTrait, Statement,
};

use super::testing_db::{prelude::*, *};

/// Add a new assistant testing trial
pub async fn insert_trial(
    user_instruction: &str,
    assistant_response: &str,
    details: GenerateDetails,
) -> Result<()> {
    let trial = trials::ActiveModel {
        user_instruction: ActiveValue::Set(user_instruction.to_string()),
        assistant_response: ActiveValue::Set(assistant_response.to_string()),
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
pub async fn test_example(path: &Path, instruction_name: &str, reps: u16) -> Result<()> {
    // Read instruction
    let instruction_file = File::open(path.join(format!("{instruction_name}.yaml")))
        .map_err(|error| eyre!("unable to read {instruction_name}.yaml: {error}"))?;
    let instruction: Instruction = serde_yaml::from_reader(instruction_file)?;

    // Read document
    let document = path.join("document.md");
    let document = if document.exists() {
        Some(codecs::from_path(&document, None).await?)
    } else {
        None
    };

    // Run repetitions in parallel
    let tasks = (0..reps).map(|_| {
        let instruction = instruction.clone();
        let document = document.clone();
        async {
            crate::perform_instruction(instruction, document, &GenerateOptions::default()).await
        }
    });
    let results = try_join_all(tasks).await?;

    // Create output file
    let mut file = File::create(path.join(format!("{instruction_name}.md")))?;
    for (index, (output, details)) in results.iter().enumerate() {
        // Write details as YAML header for the first rep, otherwise a separator for other reps
        file.write_all(
            if index == 0 {
                format!("---\n{}\n---\n\n", serde_yaml::to_string(details)?)
            } else {
                "\n\n---\n\n".to_string()
            }
            .as_bytes(),
        )?;

        // Write the output as Markdown
        file.write_all(output.display().as_bytes())?;
    }

    Ok(())
}
