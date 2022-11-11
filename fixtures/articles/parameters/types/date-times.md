An example article mainly intended for testing that date and time parameters can be set in different kernels.

These parameters should be echoed in the following `CodeChunk`s which use different execution kernels.

| Type       | Parameter                                                      |
| ---------- | -------------------------------------------------------------- |
| `Date`     | &[par_date]{date min=2022-08-11 max=2022-10-01 def=2022-09-30} |
| `Time`     | &[par_time]{time}                                              |
| `DateTime` | &[par_datetime]{datetime}                                      |

# Type aware kernels

## Storage kernel

```json exec
par_date
par_time
par_datetime
```

## Language kernels

```python exec
print(type(par_date))
print(par_date)
print(type(par_time))
print(par_time)
print(type(par_datetime))
print(par_datetime)
```

```r exec
print(class(par_date))
print(par_date)
print(class(par_time))
print(par_time)
print(class(par_datetime))
print(par_datetime)
```

## SQL kernels

```sql exec
-- @db sqlite://:memory:
SELECT $par_date;
SELECT $par_time;
SELECT $par_datetime;
```

```sql exec
-- @db duckdb://:memory:
SELECT $par_date;
SELECT $par_time;
SELECT $par_datetime;
```

# Other kernels

For these kernels the `Date` will be represented by its JSON,

```bash exec
echo $par_date
echo $par_time
echo $par_datetime;
```

```zsh exec
echo $par_date
echo $par_time
echo $par_datetime;
```
