CREATE TABLE IF NOT EXISTS trials (
    id INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
    current_timestamp TIMESTAMP DEFAULT CURRENT_TIMESTAMP NOT NULL,

    user_instruction TEXT NOT NULL,
    agent_response TEXT NOT NULL,
    generate_detail TEXT NOT NULL
);
