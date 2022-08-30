An example mainly for testing a `SqlKernel` on a SQLite database.

# Configuration

The default database for SQL code is `sqlite://:memory:` i.e. an in-memory SQLite database. That can be useful for doing on the fly SQL querys (perhaps on data loaded from Python), but most of the time you want to connect to an existing SQLite database file. You can do that by setting the using the `@db` tag in a code chunk comment. Usually you want to use the same database for all SQL code in a document so make it a `@global` e.g.

```sh
# For this example use Bash to remove any existing SQLite file,
# create a new empty one, and specify it as the one to use
rm -f data.sqlite3
touch data.sqlite3
```

```sql exec
-- @global @db sqlite://data.sqlite3
```

Alternatively, you can set the environment variable `DATABASE_URL`. In this case the specified database will be used for all SQL code unless a local `@db` tag is specified for particular code chunks.

# Column types

This code chunk creates a table with the five column types that SQLite supports (see the SQLite [data types documentation](https://www.sqlite.org/datatype3.html)):

```sql exec
drop table if exists table_1;
create table table_1 (
  col_boolean boolean,
  col_integer integer,
  col_real real,
  col_text text
);
insert into table_1 values (
  true,
  42,
  1.23,
  'Hello world'
),(
  false,
  24,
  3.21,
  'Good bye'
);
```

This code chunk then selects all columns from that table, into a `Datatable`:

```sql exec
select * from table_1;
```

This code chunk uses alternative type names for the columns which SQLite should represent as the one of the five core types:

```sql exec
drop table if exists table_2;
create table table_2 (
  col_boolean bool,
  col_integer unsigned big int,
  col_real double,
  col_text varchar(100)
);
insert into table_2 select * from table_1;
```

# Parameters and bindings

This code chunk should select all rows from `table_2` where `col_integer` is &[par_integer]{int def=42} or `col_text` is &[par_text]{str}.

```sql exec
select * from table_2
where col_integer = $par_integer
   or col_text = $par_text;
```

# Watching tables

Here, we create a table that we'll watch for changes:

```sql exec
-- @watch table_3
create table if not exists table_3(
  id integer primary key,
  time timestamp default current_timestamp,
  number real
);
```

This code chunk selects from `table_3` and should show new rows when the parameter in `./sqlite-inserter.md` is changed,

```sql exec
SELECT * from table_3 order by time desc;
```

Use `@watch @all [schema]` to watch all tables within a given schema (optional, defaulting to `main`) e.g.

```sql exec
-- @watch @all
```

Use `@unwatch <tables...>` or `@unwatch @all` to stop watches.
