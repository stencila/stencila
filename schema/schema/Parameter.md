---
title: Parameter
---

## Structure

### Name

Parameters must have a `name`. This allows them to be referred to by other executable nodes in a document and for callers to specify which parameter of a document or function that they are referring to. So, the simplest parameter would look like this:

```json
{
  "type": "Parameter",
  "name": "weight"
}
```

### Schema

You can also specify a `schema` for a parameter which specifies constraints on the values bound to the parameter.

For example,

```json
{
  "type": "Parameter",
  "name": "weight",
  "schema": {
    "type": "ArraySchema",
    "items": {
      "type": "NumberSchema",
      "exclusiveMinimum": 0
    }
  }
}
```

### Default

A parameter can have a default value. Of course if you have specified a `schema` for the parameter, then the...

## Usage

Parameters can be used in two places:

- as an item in the `parameters` property of a `CreativeWork` such as an `Article` or `Function`

- as a node within the `content` tree of a `CreativeWork` such as an `Article`

### Within `parameters`

Put a parameter in the `parameters` property when you want it to be invisible to the end user of the document. These parameters will not be modifiable by the reader but authors will be able to bind to them when calling a document e.g.

```json
{
  "type": "Call",
  "target": "./my-figure.md",
  "arguments": {
    ...
  }
}

```

### Within `content`

When a parameter is placed in the content of the document it becomes readable, and modifiable by the end user. This allows them to alter the parameter to see how it's value affects other content nodes in the document (e.g. a calculated value, a table, or a plot).

When defining a parameter within the content of a document it is strongly recommended to specify its `schema`. This provides the user interfaces that are custom to the type of data specified by the schema (e.g. a number keyboard for a parameter with `schema` of type `NumberSchema`).

## Encodings

### HTML

> Discuss how different schema types are encoded as different HTML [`<input>` types](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/input#Form_%3Cinput%3E_types)

```html
<input id="alpha" type="number" min="0" />
```
