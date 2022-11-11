```sql exec
drop table if exists table_1;
create table table_1 (
    col_a text check(length(col_a) > 3) default 'hello',
    col_b text check(col_b in ('one', 'two', 'three'))
)
```

&[col_a]{from=table_1.col_a}

&[col_b]{from=table_1.col_b}

```sql exec
insert into table_1 values($col_a, $col_b);
```
