::: form

Forms can have content

:::

# Deriving forms

Forms can be derived from database table

```sql exec
drop table if exists planets;

create table planets (
  name text,
  diameter real
)
```

```sql exec
select * from planets;
```

::: form {from=planets action=updateordelete}

&[planets_name]{from=planets.name}

&[planets_diameter]{from=planets.diameter}

:::
