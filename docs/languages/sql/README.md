# SQL

Cells and functions can be written using Structured Query Language. SQL is implemented in many alternative database engines
each having slightly different dialects. Currently, Stencila [execution context](computation/contexts.md) is available for
[SQLite](https://www.sqlite.org/) as a `SqliteContext`, and it is implemented in the Stencila packages for R, Python and Javascript :sparkles: .
To use SQL in a Stencila document, you need to [enable one of these execution contexts](getting-started/installation.md#execution-contexts).


## Data interchange :sparkles:

Stencila provides you with ability to use multiple programming languages to write interactive code within
one document, working on the same data. In other words, you can manipulate the same data switching between different programming
languages. This capability is achieved through `data interchange` feature.

When you pass data between cells Stencila temporarily converts it into its built-in [Mini language](languages/mini/README.md) object.
Currently, only the `table` data type can be exchanged with SQL. All data values are stored as temporary tables.

## Cells
With Stencila you have full control over the sequence in which your code cells are executed. You can run the code in asynchronous order.
You can refer to specific outputs from the given cell in any part of your Stencila document.
Stencila does all this using its [execution engine](computation/engine.md).

Cell inputs are determined by parsing the `FROM` clause of the SQL statement.
For example, the following cell has a single input `people`, the value is the result of
the SQL statement and the output is empty (`null`).

```sql
SELECT * FROM people ORDER BY height DESC
```

Parsing of inputs from more complicated SQL statements involving joins or sub-queries is currently not supported.

The the standard syntax of SQL means that the above cell has empty output in Stencila. If you want to refer to its
output, you need to explicitly capture it by  using an assignment operator before the `SELECT` statements.
For example, the following cell has an output `brown_hair` and an input `people`. The value is the result
of the SQL statement.

```sql
brown_hair = SELECT * FROM people WHERE hair_color == 'Brown'
```

## Functions

Stencila functions can be implemented using SQL. The approach to type checking and dispatching is similar to other languages.
 But instead of using native functions in the implementation language, you generate SQL that is executed within the database.

Lets say you have a table called `data` in a database with columns `height` and `width`. You would like to be able to
perform computations on that data without having to convert all that data into JSON. For example,

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
