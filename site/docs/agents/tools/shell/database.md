---
title: "Database"
description: "Guards against destructive MySQL operations. Guards against destructive PostgreSQL operations. Guards against destructive SQLite operations"
---

This page lists the safe and destructive patterns in the **MySQL**, **PostgreSQL**, and **SQLite** shell guard packs. See [Shell Tool](/docs/agents/tools/shell#guard-pipeline) for how these patterns are evaluated.

## MySQL

**Pack ID:** `database.mysql`

Guards against destructive MySQL operations

### Destructive patterns

| Rule ID | Reason | Suggestion | Confidence |
|---------|--------|------------|:----------:|
| `database.mysql.drop_database` | Permanently destroys the entire database | Use `mysqldump` to backup first | High |
| `database.mysql.drop_table` | Permanently destroys a table and all its data | Backup the table first with `mysqldump` | High |
| `database.mysql.truncate` | Removes all rows without logging | Use `DELETE FROM` with a `WHERE` clause | Medium |
| `database.mysql.delete_no_where` | Deletes all rows from a table | Add a `WHERE` clause. If your query already has a WHERE clause on a separate line, combine onto one line | Medium |

## PostgreSQL

**Pack ID:** `database.postgresql`

Guards against destructive PostgreSQL operations

### Destructive patterns

| Rule ID | Reason | Suggestion | Confidence |
|---------|--------|------------|:----------:|
| `database.postgresql.drop_database` | Permanently destroys the entire database | Use `pg_dump` to backup first | High |
| `database.postgresql.drop_table` | Permanently destroys a table and all its data | Use `pg_dump -t <table>` to backup first; use a transaction with `BEGIN`/`ROLLBACK` for safety | High |
| `database.postgresql.truncate` | Removes all rows without logging individual deletions | Use `DELETE FROM` with a `WHERE` clause for selective deletion | Medium |
| `database.postgresql.delete_no_where` | Deletes all rows from a table | Add a `WHERE` clause to limit deletion scope. If your query already has a WHERE clause on a separate line, combine them onto one line (e.g., `DELETE FROM users WHERE active = false`) | Medium |

## SQLite

**Pack ID:** `database.sqlite`

Guards against destructive SQLite operations

### Destructive patterns

| Rule ID | Reason | Suggestion | Confidence |
|---------|--------|------------|:----------:|
| `database.sqlite.drop_table` | Permanently destroys a table | Backup the database file first | High |
| `database.sqlite.delete_no_where` | Deletes all rows from a table | Add a `WHERE` clause. If your query already has a WHERE clause on a separate line, combine onto one line | Medium |

---

This documentation was generated from [`rust/agents/src/tool_guard/shell/packs/database.rs`](https://github.com/stencila/stencila/blob/main/rust/agents/src/tool_guard/shell/packs/database.rs).
