A little example of using PrSQL to query a database table.

First let's create the table using SQL (defaults to an SQLite in-memory database):

```sql exec
create table if not exists table_1 (
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

Then run some PrQL queries on the table

```prql exec
from table_1
```

```prql exec
from table_1
filter col_boolean == false
```

```prql exec
from table_1
filter col_text == 'Good bye'
```
