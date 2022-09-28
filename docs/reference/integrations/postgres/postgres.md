```sql exec
-- @global @db postgres://postgres:postgres@localhost:5432/testdb
```

## Deriving parameters

As for other database integrations, Stencila is able to derive a `Parameter` node from a Postgres database column definition. To do so, set the `deriveFrom` property of a parameter to a dot separated path to the column of the form `[schema.]table[.column]`. If `schema` is omitted it will be assumed to be `public`. If `column` is omitted it will be assumed to be the name of the parameter itself.

When the parameter is compiled, Stencila will query view in the [`information_schema`](https://www.postgresql.org/docs/current/information-schema.html) of the database to retrieve metadata about that column. The following executable code chunk illustrates this,

```sql exec
-- Create an example table
drop table if exists example;
create table example(
    col_a integer default 42,
    col_b real not null check(col_a > 1 and col_a < 10),
    col_c text,
    col_d date check (col_d between '2001-01-01' and '2001-01-31') default '2001-01-02',
    col_f time default '12:34:00' not null,
    col_g varchar(6)
);

-- Query the `information_schema.columns` view to get metadata
-- about the columns of interest
select data_type, column_default, is_nullable, character_maximum_length
from information_schema.columns
where table_schema = 'public'
  and table_name = 'example';

-- Query the `information_schema.check_constraints` view to get the SQL
-- of all checks associated with the table
select column_name, check_clause
from information_schema.check_constraints as cc
left join information_schema.constraint_column_usage as ccu
on cc.constraint_name = ccu.constraint_name
where table_name = 'example'
group by column_name;

```

Stencila then uses this information on the column to derive properties of the parameter using the following rules:

- the `validator` property of the parameter will be set based on the `data_type` of the column (e.g. a `StringValidator` for a `text` column)
- the `default` property of the parameter will be set to the `column_default`
- if `is_nullable` is `FALSE` and the parameter's `default` property is empty (i.e. is `None` in Rust) then it will be set to the default for the validator type
- if `character_maximum_length` is not `NULL` and the `validator` property of the parameter is a `StringValidator`, then the `maxLength` property of the validator will be set
- if there are any `check_clause`s associated with the column then they will be parsed
