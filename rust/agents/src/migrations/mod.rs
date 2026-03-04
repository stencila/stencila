use stencila_db::migration::Migration;

pub static AGENT_MIGRATIONS: &[Migration] = &[Migration {
    version: 1,
    name: "initial",
    sql: include_str!("001_initial.sql"),
}];
