use stencila_db::migration::Migration;

pub static AGENT_MIGRATIONS: &[Migration] = &[
    Migration {
        version: 1,
        name: "initial",
        sql: include_str!("001_initial.sql"),
    },
    Migration {
        version: 2,
        name: "agent_sessions",
        sql: include_str!("002_agent_sessions.sql"),
    },
];
