use std::sync::Arc;

use agent::{
    common::{eyre::Result, serde_json},
    Agent, GenerateOptions, Prompt,
};
use sea_orm::{
    ActiveValue, ConnectOptions, ConnectionTrait, Database, DatabaseBackend, EntityTrait, Statement,
};

use super::testing_db::{prelude::*, *};

/// Add a new agent testing trial
pub async fn insert_trial(
    agent: Arc<dyn Agent>,
    user_instruction: &str,
    agent_response: &str,
    options: &GenerateOptions,
) -> Result<()> {
    let prompt_name = options
        .prompt_name
        .as_ref()
        .map_or_else(|| agent.prompt(), |name| name.clone());

    let prompt_content = Prompt::load(&prompt_name)?.content()?;

    let options = serde_json::to_string(options)?;

    let trial = trials::ActiveModel {
        agent_name: ActiveValue::Set(agent.name()),
        provider_name: ActiveValue::Set(agent.provider()),
        model_name: ActiveValue::Set(agent.model()),
        prompt_name: ActiveValue::Set(prompt_name),
        prompt_content: ActiveValue::Set(prompt_content),
        options: ActiveValue::Set(options),
        user_instruction: ActiveValue::Set(user_instruction.to_string()),
        agent_response: ActiveValue::Set(agent_response.to_string()),
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
