An example mainly for testing a `SqlKernel` on a DuckDB database.

# Configuration

Use a `@global @db` tag so that all SQL and PrQL code chunks in this document use `data.duckdb`:

```sql exec
-- @global @db duckdb://data.duckdb
```

# Setup

Create a table with a couple of rows:

```sql exec
drop table if exists table_1;
create table table_1 (
  col_boolean boolean,
  col_integer integer,
  col_real real,
  col_double double,
  col_text text
);
insert into table_1 values (
  true,
  42,
  1.23,
  1.23,
  'Hello world'
),(
  false,
  24,
  3.21,
  3.21,
  'Good bye'
);
```

This SQL code chunk should select all rows from `table_1` where `col_boolean` is &[par_boolean]{bool}, `col_integer` is &[par_integer]{int def=42}, or `col_text` is &[par_text]{str},

```sql exec
select * from table_1
where col_boolean = $par_boolean
   or col_integer = $par_integer
   or col_text = $par_text;
```

The PrSQL code chunk should do the same,

```prql exec
from table_1
```

# Watching tables

Table watching is not implemented for DuckDB (there is not `NOTIFY` or `TRIGGER` to build on top of). This code chunk should return an error telling you so:

```sql exec
-- @watch @all
```
