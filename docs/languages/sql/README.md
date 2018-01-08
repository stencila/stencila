# SQL

Cells and functions can be written using Structured Query Language. SQL is implemented in many alternative database engines each having slightly different dialects. Currently, Stencila execution contexts are written available for [SQLite](https://www.sqlite.org/) as a `SqliteContext` implemented in the Stencila packages for R, Python and Node. To use SQL in a Stencila document it will need access to a `Host` from one of these packages.

## Data interchange

Currently, only the `table` data type can be exchanged with SQL.

All data values are stored as temporary tables.

## Cells

Cells can be written in SQL. Cell inputs are determined by parsing the `FROM` clause of the SQL statement. For example, the following cell has a single input `people`, and no outputs:

```sql
SELECT * FROM people ORDER BY height DESC
```

Parsing of inputs from more complicated SQL statements involving joins or sub-queries is currently not supported.

To support cell outputs, SQL execution contexts support the assignment operator for `SELECT` statements. For example, the following cell has an output `brown_hair` and an input `people`.

```sql
brown_hair = SELECT * FROM people WHERE hair_color == 'Brown'
```

## Functions

Stencila functions can be implemented using SQL. The approach to type checking and dispatching is similar to other languages. But instead of using native functions in the implementation language, you generate SQL that is exeuted within the database.

Lets say you have a table called `data` in a database with columns `height` and `width`. You would like to be able to perform computations on that data without having to convert all that data into JSON. For example, 

```mini
people | extend(bmi = (.mass * .mass) / .height) | sort(-.bmi)
```

Let's try to implement the `extend` function first. For the example call above we want it to generate SQL like this:

```sql
SELECT *, (mass * mass) / height AS bmi FROM people
```

```python
def extend(args):
  # Get the database table to be extended, by
  # selecting the argument named `value`, or the first argument
  value = select(args, ['value', '1'])
  assert_type(value, 'table')
  # Get the remaining arguments and execute them
  others = [execute(arg) for arg in remainder(args, value)]

  # Generate SQL
  return 'SELECT *, '
```

> :sparkles: Writing of functions in SQL is not yet implemented
