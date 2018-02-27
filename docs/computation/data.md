# Data

At the heart of computation in Stencila are data _values_ of different _types_. Data values are passed to, and returned from, function calls. Some functions are strongly typed, meaning that they will only accept data values of a particular type. This section introduces the various data types in Stencila. 

## Data types

There are several fundamental data types built-in to, and used internally within, Stencila. These fundamental types are the same as in [JSON](https://www.json.org/), a commonly used, lightweight, data-interchange format. In this section, we use JSON to represent values of each fundamental data type. But as is shown later, these values may have alternative representations in different languages (e.g. Mini, R).

### Fundamental data types

There are two categories of fundamental types: _atomics_, and _compounds_. The _atomic_ value types are the smallest kind of value, they can not be broken into smaller parts. There are four atomic types: `boolean`, `number`, `integer`, and `string`. There are two _compound_ fundamental types, `array` and `object`, that consist of combinations of atomic types.

#### Null

A `null` value is used to represent missing or empty data values:

```json
null
```

#### Boolean

A `boolean` can only have one of two values:

```json
true
```

or

```json
false
```

#### Number

The `number` type represents [real numbers](https://en.wikipedia.org/wiki/Real_number) e.g.

```json
3.14159265358979323846264338327950288419716
```

#### Integer

The `integer` type represents [integers](https://en.wikipedia.org/wiki/Integer) e.g.

```json
42
```

The `integer` type is a child type of `number`, so any function having a `number` as a parameter can also take an `integer`.

#### String

Strings are sequences of characters. e.g.

```json
"hello world"
```

#### Array

An `array` is an ordered collection of values e.g. 

```json
[1, 2, 3]
```

This example value is of type `array[integer]` because the values in the array are all of the same type `integer`.

The values in an array can have different types e.g.

```json
['hello world', [1, 2], {}]
```

This example value is of type `array`, a synonym for `array[any]`.


#### Object

An `object` is a collection of values in which each value has a _name_ e.g.

```json
{"height": 1.1, "width": 4.5}
```

This `object` has two _properties_: `height` and `width`.

### Extended data types

Extended types are child types of `object` with a `string` property with the name `type`. For example, an extended type representing a position on the earths surface could be written as:

```json
{
  "type": "position",
  "latitude": -42.4,
  "longitude": 173.7
}
```

The `table` type is a commonly used extended type for tabular data which is similar to a `data.frame` in R. It stores values in columns:

```json
{
  "type": "table",
  "data": {
    "height": [1.1, 1.0, 1.3],
    "width": [4.3, 3.9, 3.7]
  }
}
```

## Data packets

Stencila provides a mechanism for data to be exchanged between documents and execution contexts and among execution contexts. When passing data to/from execution `Contexts`, values are marshaled into a data _package_ which specify the type and value of the data. For example, an array of integers is exchanged in a data packet like this:

```json
{
  "type": "array[integer]",
  "data": [1, 2, 3]
}
```

Note that for extended data types, such as a `table` the data packet, somewhat confusingly, has nested structure:

```json
{
  "type": "table",
  "data": {
    "type": "table",
    "data": {
      "height": [1.1, 1.0, 1.3],
      "width": [4.3, 3.9, 3.7]
    }
  }
}
```

The data packets are converted by each execution `Context` into a native data type. For example, when a `table` value is passed to a `RContext` it is converted to a [`data.frame`](https://stat.ethz.ch/R-manual/R-devel/library/base/html/data.frame.html), and when it is passed to a `PythonContext` it is converted to a [`pandas.DataFrame`](https://pandas.pydata.org/pandas-docs/stable/generated/pandas.DataFrame.html). For more details on the protocols for translating Stencila data types to native data types, see the tables in each language section.

## Data pointers

> :sparkles: Data pointers are yet to be implemented.

Exchanging data using data packets is fine for small data, but it quickly becomes inefficient for large data values such as a large R data frame or a database table. That's because the data packet gets passed, in it entirety, from the execution context to the client user interface and then back to the same execution context or onwards to another execution context.

This is where data _pointers_ come in. Instead of encapsulating the entire data value, a data pointer just stores information on where the data resides.

For example, let's say that you were using the Star Wars example database, and had a SQL cell in a document with the code,

```sql
low_gravity_planets = SELECT * FROM planets WHERE gravity <= 1
```

This would create a temporary table in the database called `low_gravity_planets`, and instead of returning the cell value as a data packet with all the data would return the cell value as a data pointer:

```json
{
  "type": "table",
  "context": "http://127.0.0.1:2010/sqlContext1"
  "name": "low_gravity_planets"
}
```

From this information, the Stencila user interface, and other execution contexts, can access the actual data value only when they need to by calling the `fetch` method of the execution context (e.g `PUT http://127.0.0.1:2010/sqlContext1!fetch` with `{"name": "low_gravity_planets", "firstRow": 0, "lastRow": 100}` as the request body).

Data pointers allow Stencila's execution `Engine` to perform dependency analysis, and update cells when their dependencies change, without wasteful and slow data transfer. Documents will often use a single language and a single execution context. For example, consider a document with two R cells, one which creates a large data frame called `data`

```r
data <- data.frame(x=1:100000, y=rnorm(100000))
```

an one which plots that data frame:

```r
plot(data)
```

If the `RContext` always returned the value of `data` as a data packet, the entire ten thousand rows of `data` would be transferred from the context, to the client and then back again to the context. But with data pointers, when executing the second cell, the R context is able to recognize from the `context` property of the pointer that `data` resides within itself and therefore no data transfer is necessary.

It is left to implementation of the execution context to decide when to return a data value as a data packet or a data pointer. Generally execution context's will return data packets for small data values e.g. an `integer` or a short `array[string]` and a data pointer for larger data e.g. a 100000 row table. 
