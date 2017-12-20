# Extensible type system

Authors:
  - Nokome Bentley

Type: Feature
Status: Draft

# Introduction

Most programming languages have a *type system*. Wikipedia has a useful [introduction](https://en.wikipedia.org/wiki/Type_system) to type systems and why they are useful:

> a type system is a set of rules that assigns a property called type to the various constructs of a computer program, such as variables, expressions, functions or modules.[1] These types formalize and enforce the otherwise implicit categories the programmer uses for data structures and components (e.g. "string", "array of float", "function returning boolean"). The main purpose of a type system is to reduce possibilities for bugs in computer programs[2] by defining interfaces between different parts of a computer program, and then checking that the parts have been connected in a consistent way. This checking can happen statically (at compile time), dynamically (at run time), or as a combination of static and dynamic checking. Type systems have other purposes as well, such as expressing business rules, enabling certain compiler optimizations, allowing for multiple dispatch, providing a form of documentation, etc.

Currently, Stencila has a very simple type system. Types are represented by a string name e.g. `"boolean"`. There are four atomic types: `boolean, number, integer, string` and three compound types `object, array, table`. There is a special type `any` to indicate that any data type can be used. The type system is hierarchical e.g.  an `integer` is a type of `number` and can be used as a parameter to a function that requires a number. If an array contains only elements of one type then it is a *specialised* array e.g. `[1,2,3]` is an `array[integer]` which is a child type of `array`. 

This simple type system is currently defined in the main Stencila repository in the file `src/types.js`. Type checking is done by attaching the `type` property to a value and checking against things like function definitions and column types as required.

# Rationale

This proposal is for a richer, more extensible type system for Stencila. In particular, the community have suggested that it would be useful to provide a way for users to easily create, and *share*, type definitions which encode domain specific knowledge regarding data validity and vocabularies. The community has also suggested integrating the current test system with the type checking system (i.e. type checks are just a particular type of test). 

# Representing types

This proposal to treat types in a similar manner to how functions are treated in Stencila. That is by creating a schema for `Types`, similar to the current schema for `Functions`, and allow them to be "plugged in" via [`libcore`](https://github.com/stencila/libcore), and in the future, domain specific libraries (e.g. `libastro`, `libecons` etc).

Types would be **defined by one or more assertions**. Each assertion would be defined as an expression (defaulting to Mini, but potentially in another language) which evaluates to a boolean `true` or `false`. For example, the `number` type could be defined using a single assertion using the `is_number` function. This treat type definitions similarly to how we treat test definitions.

A type could be represented in XML as:

```xml
<type name="number" coerce="number">
  <asserts>
    <assert condition="is_number(value)" message="Value should be a number" />
  </asserts>
</type>

```

The `coerce` attribute is the name of the function to be used to attempt to coerce any value to that type. In this example the coercing function for the type `number` is also called `number` which in Javascript, for example, would be implemented using the `parseFloat` function. This function would be used when converting values form a string to the column type in a sheet.

Types would be able to extend other types. For example, a type for temperature measured in degrees Celsius might be represented as:

```xml
<type name="celsius" label="°C" parent="number">
  <asserts>
    <assert condition="value >= −273.15" message="Value should be greater than, or equal to, absolute zero (−273.15)" />
  </asserts>
</type>
```

Other meta-data and documentation elements should be added to the type schema `e.g. <description>`

A `category` type would be similar to [R's factors](https://www.stat.berkeley.edu/classes/s133/factors.html) - variables that can have a limited set of values. This may require additional type level elements to store the set of allowable values e.g.


```xml
<type name="category">
  <values/>
  <asserts>
    <assert condition="contains(value, values)" message="Value must be one of values for category" />
  </asserts>
</type>
```

Then domain specific categories could be defined by defining the set of valid values e.g.

```xml
<type name="fish_families" parent="category">
  <description>Families of fish</description>
  <values type="string">
    <value>Acanthuridae</value>
    <value>Acanthuridae</value>
    <value>Acestrorhynchidae</value>
    ...
    <value>Zoarcidae</value>
  </values>
</type>
```
