# Database Schema Snapshots

This directory contains the current database schema (`current.cypher`) and schema snapshots for different Stencila versions.

The `current.cypher` file is used when creating a new database.

The snapshots are used to generate database migrations in `../migrations/`.

## File Naming

The current database schema is represented as Cypher as `current.cypher`.

Schema snapshots are stored as JSON files with the naming convention `v{VERSION}.json` (e.g. `v2.6.0.json`).

Between releases (which is when new version numbers are minted) a schema snapshot named `v99.99.99.json` is generated if there are changes in the schema. When a new release is made this file is renamed using the freshly minted version number.

## File Format

See `../../schema-gen/src/kuzu_types.rs` for the schema representation that is serialized to snapshot files.

## Generation

The current schema (`current.cypher`) and current snapshot (`v99.99.99.json`) are automatically created during the schema generation:

```sh
cargo run -p schema-gen
```
