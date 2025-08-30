# Schema Snapshots

This directory contains schema snapshots for different Stencila versions. These snapshots are used to generate database migrations when the schema changes between versions.

## File Format

Schema snapshots are stored as JSON files with the naming convention `v{VERSION}.json` e.g.

- `v2.1.0.json`
- `v2.2.0.json`
- `v3.0.0.json`

## Contents

Each schema snapshot file contains:

- `version`: The Stencila version this schema corresponds to
- `node_tables`: Definition of all node tables (name, columns, primary key)
- `relationship_tables`: Definition of all relationship tables (name, from/to tables, cardinality)
- `indices`: Definition of all indices (FTS and vector indices)
- `timestamp`: When the snapshot was created

## Usage

Schema snapshots are automatically created during the schema generation process and are used by the migration system to:

1. Compare schemas between versions
2. Generate migration files automatically
3. Track schema evolution over time
4. Validate that migrations are complete and correct

## Migration Generation Process

When a new schema is generated:

1. A new snapshot is created for the current version
2. The system compares it with the previous version's snapshot
3. If differences are found, a migration file is generated
4. The migration is stored in the `../migrations/` directory

This ensures that database schema changes are tracked and can be applied automatically when upgrading Stencila versions.
