# Database Migrations

This directory contains Kuzu database migration files that track schema changes across Stencila versions. These are usually generated from differences between the versioned schema snapshots in `../snapshots/`.

## File Naming

Migration files must follow the naming convention `v{VERSION}.cypher` where `{VERSION}` is the version of the schema that the migration will take the database to. For example, `v2.9.0.cypher` is the migration from the previous version (whatever that may be) to version `2.9.0`.

Between releases (which is when new version numbers are minted) a migration file named `v99.99.99.cypher` is generated if there are changes in the schema. When a new release is made this file is renamed using the freshly minted version number.

## File Format

Each migration file should contain valid Kuzu Cypher DDL statements such as:

```cypher
-- Add a new column to an existing table
ALTER TABLE `Reference` ADD COLUMN `appearanceIndex` UINT64 DEFAULT NULL;

-- Create a new table
CREATE NODE TABLE IF NOT EXISTS `NewTable` (
  `id` STRING PRIMARY KEY,
  `name` STRING
);

-- Add FTS index
CALL CREATE_FTS_INDEX('NewTable', 'fts', ['name']);
```

The migration system supports all Kuzu DDL operations:

### Table Operations

- `CREATE NODE TABLE` - Create new node tables
- `CREATE REL TABLE` - Create new relationship tables
- `DROP TABLE` - Remove existing tables
- `ALTER TABLE RENAME TABLE` - Rename tables

### Column Operations

- `ALTER TABLE ADD COLUMN` - Add new columns with optional defaults
- `ALTER TABLE DROP COLUMN` - Remove columns
- `ALTER TABLE RENAME COLUMN` - Rename columns

### Index Operations

- `CALL CREATE_FTS_INDEX()` - Create full-text search indices
- `CALL CREATE_VECTOR_INDEX()` - Create vector similarity indices
- Index removal (via DROP statements)

### Complex Schema Changes

For complex changes like column type modifications, migrations use a multi-step approach:

1. Add a new column with the desired type
2. Copy and transform data from the old column using Cypher queries
3. Drop the old column
4. Rename the new column to the original name

### Relationship Table Operations

- Create new relationship tables with specified FROM/TO constraints
- Modify relationship cardinality (ONE_ONE, ONE_MANY, MANY_MANY)
- Remove relationship tables

## Generation

Migrations are automatically created during schema generation:

```sh
cargo run -p schema-gen
```

## Execution

Migrations are automatically discovered and executed by the migration system when:

1. A new database is created
2. An existing database is opened with a newer version of Stencila
3. Migrations are manually triggered via CLI commands

The system tracks applied migrations in the `_migrations` table to ensure each migration is only run once.
