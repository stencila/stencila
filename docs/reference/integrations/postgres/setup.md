```bash exec
# @once
docker run -d --rm -p5432:5432 -e POSTGRES_PASSWORD=postgres postgres
```

Alternatively,

```bash
docker run -it --rm -p5432:5432 -e POSTGRES_PASSWORD=postgres postgres
```

```bash exec
# @once
PGPASSWORD=postgres createdb --host=localhost --user postgres temp
```

```bash
docker run --rm -p5050:80 -e PGADMIN_DEFAULT_PASSWORD=postgres dpage/pgadmin4
```

```sql exec
-- @global @db postgres://postgres:postgres@localhost:5432/temp
```
