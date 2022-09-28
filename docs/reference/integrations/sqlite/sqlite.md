## Deriving parameters

As for other database integrations, Stencila is able to derive a `Parameter` node from a SQLite database column definition. To do so, set the `deriveFrom` property of a parameter to a dot separated path to the column of the form `[schema.]table[.column]`. If `schema` is omitted it will be assumed to be `main`. If `column` is omitted it will be assumed to be the name of the parameter itself.

When the parameter is compiled, Stencila will query the [`sqlite_schema` table](https://www.sqlite.org/schematab.html) of the database to retrieve the SQL that describes the `table` (usually a normalized copy of the SQL used to create the table). The following executable code chunk illustrates this,

```sql exec
-- @db sqlite://:memory:

-- Create an example table
drop table if exists example;
create table example(
    col_a integer check(col_a > 1 and col_a < 10),
    col_b real,
    col_c text,
    col_d date check (col_d > '2001-01-01') default '2001-01-02'
);

-- Query the `sqlite_master` table to get the SQL that was
-- used to create the `example` table
select "sql"
from main.sqlite_master
where name = 'example';
```

Stencila then parses the SQL (using its `SqlParser` which is based on `tree-sitter-sql`) and, for the relevant column, extracts the following information:

- the type of the column e.g. `INTEGER`, `DATE` which will determine the type of the parameter's `validator`

- any `DEFAULT` the column may have which will determine the `default` of the parameter (only literal values are used, expressions for `DEFAULT` are ignored)

- whether the column has a `NOT NULL` clause which is used to determine if the parameter must have a `default`

- any `CHECK` expression, either singular or `AND`ed together (if there is any `OR` in the expression it will be completely ignored), of the form,

  - `column > value` which will set the validator's `exclusiveMinimum` (or `minimum` if the validator type has no `exclusiveMinimum` e.g `DateValidator`) to `value`
  - `column >= value` which will set the validator's `minimum` to `value`
  - `column < value` which will set the validator's `exclusiveMaximum` (or `maximum` if the validator type has no `exclusiveMaximum`) to `value`
  - `column <= value` which will set the validator's `maximum` to `value`
  - `column = value` which will set the validator's `minimum` and `maximum` to `value`
  - `length(column) > value` which will set a `StringValidator`'s `minLength` to `value+1`, etc for other comparison operators as above.
  - `column BETWEEN min AND max` equivalent to `column >= min AND column <= max`

- if the column is a `VARCHAR(n)` then set a `StringValidator`'s `maxLength` to `n`

When deriving the properties of a `Parameter` from a SQLite column, only those properties that are missing (i.e. are `None` in Rust) are set based on the column. e.g. if the parameter already has a `default`, then any `DEFAULT` clause on the column will be ignored.

Because Stencila is not able to watch for changes in the schema of a SQLite table, deriving from a column will only be done once.
