# Database Migrations

This directory contains Kuzu database migration files that track schema changes across Stencila versions. These are usually generated from the schema snapshots in the `../snapshots/` directory (see the `README.md` there).

## Migration File Format

Migration files must follow the naming convention:

```
v{VERSION}-{DESCRIPTION}.cypher
```

Where:

- `{VERSION}` is a semantic version (e.g., `2.1.0`, `2.1.1`)
- `{DESCRIPTION}` is a kebab-case description of the migration (e.g., `add-reference-appearance-index`)

For example,

- `v2.1.0-add-reference-appearance-index.cypher`
- `v2.2.0-remove-deprecated-columns.cypher`
- `v2.19.3-restructure-relationship-tables.cypher`

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

## Supported Operations

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

## Execution

Migrations are automatically discovered and executed by the migration system when:

1. A new database is created
2. An existing database is opened with a newer version of Stencila
3. Migrations are manually triggered via CLI commands

The system tracks applied migrations in the `_migrations` table to ensure each migration is only run once.
