/// A single schema migration for a domain.
///
/// Migrations are identified by `(domain, version)` pairs. The `sql` field
/// contains the DDL/DML statements to execute. Migrations must be additive
/// and idempotent where possible (use `CREATE TABLE IF NOT EXISTS`, etc.).
pub struct Migration {
    /// Monotonically increasing version number within the domain.
    pub version: i32,
    /// Human-readable name for this migration (e.g. `"initial"`, `"add_embeddings"`).
    pub name: &'static str,
    /// SQL statements to execute.
    pub sql: &'static str,
}
