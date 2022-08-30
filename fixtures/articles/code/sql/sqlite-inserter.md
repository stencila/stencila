An example which inserts a row into `table_3` which is `@watch`ed by `./sqlite.md`.

```sql exec
-- @db sqlite://data.sqlite3
```

Here is a parameter, which should be inserted into `table_3` along with the current time: &[number]{num}.

Here is the code chunk that does the insertion:

```sql exec
-- @db sqlite://data.sqlite3
insert into table_3(time, number)
values(current_timestamp, $number);
```
