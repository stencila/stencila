# Writing `<type>.schema.yaml` files

The schema for a type is defined, using JSON Schema, in a `<type>.schema.yaml` file. We use YAML, which is a superset of JSON, because it is more readable than JSON.

See the excellent [Understanding JSON Schema](https://json-schema.org/understanding-json-schema/) for guides on writing a JSON Schema. The following sections describe the most important, and custom, keywords in the schema in the order that they normally appear.

## The `title` keyword

Each type schema MUST begin with the title of the type e.g.

```yaml
title: Organization
```

## The `@id` keyword

This is a custom keyword used when generating the JSON-LD `@context`.

You MUST declare the `@id` keyword for each type using the format `<context>:<type>`. For example, `schema:Person`. Note that because this property name begins with the special character `@`, that it needs to be surrounded by quotes e.g.

```yaml
'@id': schema:Organization
```

Currently, the Stencila context allows you to refer to the following external contexts:

- `schema`: https://schema.org/
- `bioschemas`: http://bioschemas.org
- `codemeta`: https://doi.org/10.5063/schema/codemeta-2.0

For example, the type schema to represent a laboratory protocol might use the `@id` of the Bioschemas [`LabProtocol`](http://bioschemas.org/specifications/LabProtocol/).

```yaml
'@id': bioschemas:LabProtocol
```

Just as for types, properties of types can be linked to the other contexts using the `@id` keyword. For example,

```yaml
- properties:
    address:
      '@id': schema:address
      type: string
```

## The `$extends` keyword

This is a custom keyword which allows your type schema to inherit the `properties` and `required` keywords of a parent type schema. It should be a _relative_ file path e.g.

```yaml
$extends: ../Thing.schema.yaml
```

## The `role` keyword

A RECOMMENDED custom keyword to indicate the role of the type schema:

- `base`: base types, not usually instantiated but required for other types e.g `Thing`
- `primary`: types that are usually the root of a tree generated from a file e.g. `Article`, `Datatable`, `Collection`
- `secondary`: types usually only referred to by primary types e.g. `Organization` is used for the `publisher` property on a `Article`
- `tertiary`: types usually only referred to by secondary types e.g. `ContactPoint` is used for the `contactPoints` property on an `Organization`

## The `status` keyword

A RECOMMENDED custom keyword to indicate the development status of a type schema e.g.

- `experimental`: new types (i.e. not defined on schema.org or elsewhere) that are still under development
- `unstable`: types that are defined elsewhere (e.g. on http://bioschemas.org) but for which the schema is still being developed
- `stable`: types for which the schema definition can be considered stable

## The `description` keyword

It is RECOMMENDED to add a description for all type schemas and properties. Descriptions can be Markdown formatted.

## The `aliases` keyword

A custom keyword which allows you to define aliases for properties. For example,

```yaml
properties:
  ...
  familyNames:
    '@id': schema:familyName
    aliases:
      - familyName
      - surname
      - surnames
      - lastName
      - lastNames
```

## The `parser` keyword

A custom keyword which allow you to define allowable shorthand strings for a property or types. The value of the keyword is the name of a parser to use. Parsers always take a string but differ in the type that they produce. Several parsers are available:

- `ssv`: space separated values to an array of strings
- `csv`: comma separated values to an array of strings
- `person`: personal name, email and url to a `Person`

You can implement and register additional parsers. See the parser for `Person` as an example of how to do that.

You can specify a parser for both types and properties. To specify a parser for a type, add the `parser` keyword at the top level e.g.

```yaml
title: Person
---
parser: person
```

You can specify a parser for a property using `anyOf`. For example, to allow `givenNames` to de provided as either a space separated values string or as an array of strings.

```yaml
title: Person
...
properties:
  ...
  givenNames:
    ...
    anyOf:
      - parser: ssv
      - type: array
        items:
          type: strings
```
