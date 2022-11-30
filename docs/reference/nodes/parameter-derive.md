Parameters can be derived from other objects, often from columns in database tables. Parameter derivation means that you can avoid duplicating the spcification of the validator.

For example,

```sql exec
-- Drop the table so that it is recreated with new
-- column definitions each time we execute this code chunk
drop table if exists plants;

-- Try changing the data types and checks on the column tables
-- and see how the parameters change
create table plants (
    name text default 'Unnamed',
    height real default 1 check (height > 0 and height <= 100)
)
```

&[name]{from=plants.name}

&[height]{from=plants.height label="Height (m)"}
