Stencila allows you to connect to a DuckDB.

If you spot any inconsistencies with this document please let us know.

Much of the documentation, including executable SQL code is based on, or borrowed from, the [DuckDB documentation](https://duckdb.org/docs/).

# Configuration

```sql exec
-- @global @db duckdb://data-types.duckdb
```

# Data types

## Overview

DuckDB has [several data types](https://duckdb.org/docs/sql/data_types/overview). Conversion to Stencila `DatatableColumn`s is supported for most, but not all, of these:

| DuckDB types                                          | `item_validator` type            | `values` item type |
| ----------------------------------------------------- | -------------------------------- | ------------------ |
| `BOOLEAN`                                             | `BooleanValidator`               | `Boolean`          |
| `ENUM`                                                | `EnumValidator`                  | `String`           |
| `TINYINT`, `SMALLINT`, `INTEGER`, `BIGINT`, `HUGEINT` | `IntegerValidator`               | `Integer`          |
| `UTINYINT`, `USMALLINT`, `UINTEGER`, `UBIGINT`        | `IntegerValidator` with `min: 0` | `Integer`          |
| `REAL`, `DOUBLE`, `DECIMAL`                           | `NumberValidator`                | `Number`           |
| `VARCHAR`                                             | `StringValidator`                | `String`           |
| `DATE`                                                | `DateValidator`                  | `Date`             |
| `TIME`                                                | `TimeValidator`                  | `Time`             |
| `TIMESTAMP`                                           | `TimestampValidator`             | `Timestamp`        |
| `INTERVAL`                                            | `DurationValidator`              | `Duration`         |

Stencila does not yet support conversion from DuckBD's `BLOB`, `LIST`, `STRUCT` and `MAP` composite types. Any columns with theses types will be dropped in the `Datatable` result of queries. It is likely that we will support conversion of DuckDB `LIST`s to Stencila `Array`s and DuckDB `STRUCT`s to Stencila `Object`s in the future. Please let us know if you would like to see this happen.

This code chunk creates a table named `data_types` which includes a column for each of the DuckDB data types. It then inserts a row of default values and a row of alternative values. You can use it to test how different DuckDB data types are converted to Stencila node types.

```sql exec
DROP TYPE IF EXISTS enum_type CASCADE;
CREATE TYPE enum_type AS ENUM ('variant1', 'variant2', 'variant3');

DROP TABLE IF EXISTS data_types;
CREATE TABLE data_types (
  -- Column types for which conversion is supported

  col_boolean BOOLEAN DEFAULT true,

  col_enum enum_type DEFAULT 'variant1',

  col_tinyint TINYINT DEFAULT -123, -- 1 byte
  col_smallint SMALLINT DEFAULT -123, -- 2 bytes
  col_integer INTEGER DEFAULT -123, -- 4 bytes
  col_bigint BIGINT DEFAULT -123, -- 8 bytes
  col_hugeint HUGEINT DEFAULT -123, -- 16 bytes

  col_utinyint UTINYINT DEFAULT 123, -- 1 byte
  col_usmallint USMALLINT DEFAULT 123, -- 2 bytes
  col_uinteger UINTEGER DEFAULT 123, -- 4 bytes
  col_ubigint UBIGINT DEFAULT 123, -- 8 bytes

  col_real REAL DEFAULT 3.14, -- 4 bytes
  col_double DOUBLE DEFAULT 3.14, -- 8 bytes
  col_decimal DECIMAL(3,2) DEFAULT 3.14,

  col_varchar VARCHAR DEFAULT 'Hello',

  col_date DATE DEFAULT '2022-09-10',

  -- Column types not yet supported

  col_blob BLOB,
  col_time TIME,
  col_timestamp TIMESTAMP,
  col_timestamptz TIMESTAMP WITH TIME ZONE,
  col_interval INTERVAL,

  col_list INTEGER[],
  col_struct STRUCT(a INTEGER)
);
INSERT INTO data_types(col_boolean) VALUES(false);
INSERT INTO data_types(col_enum) VALUES('variant2');

SELECT * FROM data_types;
```

### Nulls

The DuckDB [`NULL`](https://duckdb.org/docs/sql/data_types/nulls) value is a special value that is used to represent missing data. Columns of any type can contain `NULL`.

```sql exec
-- Insert a null value into a table
DROP TABLE IF EXISTS integers;
CREATE TABLE integers(col_int INTEGER);
INSERT INTO integers VALUES (NULL);
SELECT * FROM integers;
```

### Booleans

The DuckDB [`BOOLEAN`](https://duckdb.org/docs/sql/data_types/boolean) type represents a statement of truth (“true” or “false”). In SQL, the boolean field can also have a third state “unknown” which is represented by the SQL NULL value.

```sql exec
-- Select the three possible values of a boolean column
SELECT TRUE AS 'T', FALSE AS 'F', NULL::BOOLEAN;
```

### Enums

The DuckDB [`ENUM`](https://duckdb.org/docs/sql/data_types/enum) type represents possible values of a column. For example, a column storing the days of the week can be an `ENUM` holding all possible days. Enums are particularly interesting for string columns with high cardinality. This is because the column only stores a numerical reference to the string in the Enum dictionary, resulting in immense savings in disk storage and faster query performance.

```sql exec
-- Create a new type for tribes of ducks
DROP TYPE IF EXISTS tribe CASCADE;
CREATE TYPE tribe AS ENUM ('dabbling', 'diving', 'sea');

-- Create a table ducks, with attributes name (string type) and tribe (tribe type)
CREATE TABLE ducks (
    name text,
    tribe tribe
);

-- Insert tuples in the ducks table
INSERT INTO ducks VALUES
  ('Gadwall','dabbling'),
  ('Shoveler','dabbling'),
  ('Pochard', 'diving'),
  ('Mallard', NULL),
  ('Bufflehead', 'sea');

-- This would fail since the tribe enum does not have a 'quacking' value.
-- INSERT INTO ducks VALUES ('Daffy','quacking');

-- The string 'dabbling' is cast to the type `tribe`, returning a integer reference value
-- to the enum variant.
SELECT * FROM ducks WHERE tribe == 'dabbling';
```

### Numbers

See the [DuckDB docs](https://duckdb.org/docs/sql/data_types/numeric) for details on which numeric types it supports. Conversion of integers and floats should work as expected.

```sql exec
SELECT 127::TINYINT, 32767::SMALLINT, 2147483647::INTEGER, 9223372036854775807::BIGINT;
```

```sql exec
SELECT pi()::REAL, pi()::DOUBLE, pi()::DECIMAL, pi()::DECIMAL(18, 10);
```

However, both `HUGEINT` and `UBIGINT` will hit the maximum (and minimum) value that can be represented in memory by a 64-bit integer (what Stencila uses to represent all integers) of 9223372036854775807. The minimum possible 64-bit integer is 9223372036854775808.

```sql exec
SELECT 9223372036854775807::HUGEINT, 9223372036854775808::HUGEINT, 170141183460469231731687303715884105727::HUGEINT;
SELECT -9223372036854775808::HUGEINT, -9223372036854775809::HUGEINT, -170141183460469231731687303715884105727::HUGEINT;
SELECT 9223372036854775807::UBIGINT, 9223372036854775808::UBIGINT, 18446744073709551615::UBIGINT;
```

### Strings

The DuckDB [`VARCHAR`](https://duckdb.org/docs/sql/data_types/text) type is used to store strings. There are many [functions](https://duckdb.org/docs/sql/functions/char) that operate on `VARCHAR` values e.g.

```sql exec
SELECT concat('Hello from DuckDB version ', version(), '. The date it ', current_date, '.') AS greeting;
```

### Dates

The DuckDB [`DATE`](https://duckdb.org/docs/sql/data_types/date) type can be used to represent calendar dates. They are converted to a Stencila `Date` type e.g.

```sql exec
SELECT DATE '2022-09-10';
SELECT DATE '3000-01-01';
```

Note that the DuckDB special infinity date values convert to a Stencila `Null` (because they overflow the internal numeric representation):

```sql exec
SELECT '-infinity'::DATE, 'epoch'::DATE, 'infinity'::DATE;
```

&[par_date]{num def=false}

```sql exec
INSERT INTO data_types(col_date) VALUE($par_date);
```

### Times

The DuckDB [`TIME`](https://duckdb.org/docs/sql/data_types/date) type is represents in Stencila as the number of microseconds since midnight,

```sql exec
SELECT '00:00:00'::TIME, '00:00:01'::TIME, '00:01:00'::TIME, '01:00:00'::TIME;
```

### Timestamps

The DuckDB [`TIMESTAMP`](https://duckdb.org/docs/sql/data_types/timestamp) type is converted to a Stencila `Timestamp` with a `timeUnit` of microseconds,

```sql exec
SELECT '2022-01-02T01:01:01'::TIMESTAMP;
```

### Durations

The DuckDB [`INTERVAL`](https://duckdb.org/docs/sql/data_types/interval) type is used to represent a period of time. In Stencila, an `INTERVAL` is represented by a `Duration` with a `timeUnit` of milliseconds,

```sql exec
SELECT '1 year'::INTERVAL, '30 seconds'::INTERVAL, '123 milliseconds'::INTERVAL;
```

Note that intervals result from differences in timestamps and can be negative:

```sql exec
SELECT '2022-01-02'::TIMESTAMP - '2022-01-01'::TIMESTAMP;
SELECT '2022-01-01'::TIMESTAMP - '2022-01-02'::TIMESTAMP;
```

## Deriving parameters

As for other database integrations, Stencila is able to derive a `Parameter` node from a DuckBD database column definition. To do so, set the `deriveFrom` property of a parameter to a dot separated path to the column of the form `[schema.]table[.column]`. If `schema` is omitted it will be assumed to be `main`. If `column` is omitted it will be assumed to be the name of the parameter itself.

When the parameter is compiled, Stencila will query the [`information_schema.columns` table](https://duckdb.org/docs/sql/information_schema#columns) of the database to retrieve metadata about that column. The following executable code chunk illustrates the metadata available from this table,

```sql exec
-- @db duckdb://:memory:

-- Create an example table
create or replace table example(
    col_a integer default 42,
    col_b real not null check(col_b < 1),
    col_c text,
    col_e date default '2022-02-22' not null,
    col_f time default '12:34:00'
);

-- Query the `information_schema.columns` table to get metadata
-- about the columns of interest
select data_type, column_default, is_nullable, character_maximum_length
from information_schema.columns
where table_schema = 'main'
  and table_name = 'example';
```

Stencila then uses this information to derive properties of the parameter using the following rules:

- the `validator` property of the parameter will be set based on the `data_type` of the column (e.g. a `StringValidator` for a `text` column)
- the `default` property of the parameter will be set to the `column_default`
- if `is_nullable` is `FALSE` and the parameter's `default` property is empty (i.e. is `None` in Rust) then it will be set to the default for the validator type
- if `character_maximum_length` is not `NULL` and the `validator` property of the parameter is a `StringValidator`, then the `maxLength` property of the validator will be set

At the time of writing, it is not possible to obtain other column metadata such as `CHECK` clauses for a DuckDB column. This limits the `validator` properties which can be inferred (c.f. Postgres and SQLite where `CHECK` clauses can be obtained).
