use agent::{
    common::{eyre::Result, serde_json},
    GenerateDetails,
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
