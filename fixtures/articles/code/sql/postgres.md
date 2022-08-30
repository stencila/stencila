An example mainly for testing a `SqlKernel` on a Postgres database.

# Configuration

Use a `@global @db postgres://..` tag. To run this example, you'll need to have this database running locally,

```sql exec
-- @global @db postgres://postgres:postgres@localhost:5432/testdb
```

# Column types

See the Postgres [data types documentation](https://www.postgresql.org/docs/current/datatype.html) for a list of all the built-in general-purpose data types.

# Watching tables

Here, we create a table that we'll watch for changes:

```sql exec
-- @watch table_3
create table if not exists table_3(
  id bigserial,
  time timestamp default now(),
  number real
);
```

This code chunk selects from `table_3` and should show new rows when the parameter in `./postgres-inserter.md` is changed,

```sql exec
SELECT * from table_3 order by time desc;
```

Use `@watch @all [schema]` to watch all tables within a given schema (optional, defaulting to `public`) e.g.

```sql exec
-- @watch @all
```

Use `@unwatch <tables...>` or `@unwatch @all` to stop watches.
