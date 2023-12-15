CREATE TABLE IF NOT EXISTS trials (
    id INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
    agent_name TEXT NOT NULL,
    provider_name TEXT NOT NULL,
    model_name TEXT NOT NULL,
    prompt_name TEXT NOT NULL,
    prompt_content TEXT NOT NULL,
    options TEXT NOT NULL,
    current_timestamp TIMESTAMP DEFAULT CURRENT_TIMESTAMP NOT NULL,
    user_instruction TEXT NOT NULL,
    agent_response TEXT NOT NULL
);
