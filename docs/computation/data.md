# Data

Computation is done on _data_. Data values are of different _types_.

## Fundamental data types

The fundamental value types are built-in to Stencila and used internally. The fundamental types are the same as in [JSON](https://www.json.org/), a commonly used, lightweight, data-interchange format. In this section, we use JSON to represent values of each fundamental data type. But as is shown later, these values may have alternative representations in different languages.

There are two categories of fundamental types: _atomics_, and _compounds_.

### Atomics

The _atomic_ value types are the smallest kind of value, they can not be broken into smaller parts. There are four atomic types: `boolean`, `number`, `integer`, and `string`.

#### Null

A `null` value is used to represent missing values:

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

`integer` is a child type of `number`, so any function having a `number` as a parameter can also take an `integer`.

#### String

Strings are sequences of characters. e.g.

```json
"hello world"
```

### Compounds

There are two _compound_ fundamental types, `array` and `object`, that consist of combinations of atomic types.

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

## Extended data types

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