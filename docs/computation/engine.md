# Engine

The `Engine` is resposible for deciding _when_ (i.e. in which order) and _where_ (i.e. in which execution `Context`) cells are executed.
It can be thought of as a meta-`Context`, that orchestrates other `Contexts`.

The `Engine` takes the code from a cell and converts it into an operation which it then passes
to the `execute` method of the `Context` which executes the operation and returns.

## Cells

With Stencila you have full control over the sequence in which your code cells are executed. You can run the code in asynchronous order.
You can refer to specific outputs and inputs from the given cell in any part of your Stencila document. Depending
on the programming language which you use in the cell, you may need to capture the output explicitly, in order to be able to
refer to it. For more details see the documentation for using different programming languages in Stencila.

If you do not capture the output explicitly, you will not be able to refer to it later on. But the input of the cell
will still be available.


## Pointers

The engine has an array of `Pointers` to data values which it knows about. The data values themselves reside inside an execution context but the array needs to

For example, let's say that you were using the Star Wars example database, and had a SQL cell with the code,

```sql
low_gravity_planets = SELECT * FROM planets WHERE gravity <= ${max_gravity}
```

The `Engine` would send this code the `analyse` method of the `SqliteContext` which would return a JSON response like the following:

```json
{
  "inputs": ["max_gravity"],
  "output": "low_gravity_planets",
  "messages": []
}
```

That tells the `Engine` that this cell requires the variable `max_gravity` as an input and produces `low_gravity_planets` as an output. With that information the engine constructs a call operation:

```json
{
  "type": "call",
  "func": {
    "type": "source",
    "lang": "sql",
    "pars": [{
      type: "par",
      name: "max_gravity"
    }],
    "body": "low_gravity_planets = SELECT * FROM planets WHERE gravity <= ${max_gravity}"
  },
  "args": [1.5]
}
```

```json
{
  "inputs": ["max_gravity"],
  "output": "low_gravity_planets",
  "type": "table",
  "messages": []
}
```

The engine then inserts the

```js
'low_gravity_planets' : {
  type: 'table',
  context: 'http://127.0.0.1:2010'
  name: 'low_gravity_planets'
}
```

Later in the document you might want to use an internal language, such as a data flow diagram or Mini to summarise the `low_gravity_planets` table e.g.

```mini
mean(low_gravity_planets.surface_water)
```

When compiled, this Mini code produces an operation:

```json
{
  "type": "call",
  "func": {"type": "get", "name:": "mean"},
  "args": [
    {"type": "get", "name": "low_gravity_planets"}
  ]
}
```

The `Engine` scans this call operation for any global `get` operations (i.e. a `get` not within a function), finds the `get` for `low_gravity_planets` and thus schedules this call to be run in the `SqliteContext`.
