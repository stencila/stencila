# Writing `<type>.schema.yaml` files

The schema for a type is defined, using JSON Schema, in a `<type>.schema.yaml` file. We use YAML, which is a superset of JSON, because it is more readable than JSON.

See the excellent [Understanding JSON Schema](https://json-schema.org/understanding-json-schema/) for guides on writing a JSON Schema. The following sections describe the most important, and custom, keywords in the schema in the order that they normally appear.

## The `title` keyword

Each type schema MUST begin with the title of the type e.g.

```yaml
title: Organization
```

## The `@id` keyword

This is a custom keyword used to define a term with the vocabulary of the Stencila JSON-LD `@context`.

Where possible we use terms from existing vocabularies. Currently, the Stencila context allows you to refer to the following external vocabularies:

- `schema`: https://schema.org/
- `bioschemas`: http://bioschemas.org
- `codemeta`: https://doi.org/10.5063/schema/codemeta-2.0

### Type `@id`s

You MUST declare the `@id` keyword for each type using the format `<context>:<type>`. Note that because this property name begins with the special character `@`, that it needs to be surrounded by quotes e.g.

```yaml
'@id': schema:Person
```

Use existing type names from other vocabularies as much as possible. For example, the type schema to represent a laboratory protocol might use the `@id` of the Bioschemas [`LabProtocol`](http://bioschemas.org/specifications/LabProtocol/).

```yaml
'@id': bioschemas:LabProtocol
```

When a type is not represented in another vocabulary, or has a sufficiently different structure to a similar type elsewhere, define the id within the Stencila context i.e. `'@id': stencila:<type>`

### Property `@id`s

You MUST declare the `@id` keyword for each property of a type using the format `<context>:<property>`.

Often, the `@id` will be the same as the property name. However, you should reuse property names from other vocabularies where possible. For example, the `Person` type schema has a property `givenNames` (not the plural) which is an array of strings.

```yaml
givenNames:
  '@id': schema:givenName
  type: array
  items: string
```

By declaring the `@id` of that property as `schema:givenName` we are saying "within this vocabulary, when we use the term 'givenNames', we mean the same as http://schema.org/givenName".

Sometimes, a property name is not represented in another vocabulary. In these casese, define the property name as a new term within the Stencila vocabulary i.e. `'@id': stencila:<property>`

## The `extends` keyword

This is a custom keyword which allows your type schema to inherit the `properties` and `required` keywords of a parent type schema. It should be the name of another type e.g.

```yaml
extends: Entity
```

## The `role` keyword

A RECOMMENDED custom keyword to indicate the role of the type schema:

- `base`: base types, not usually instantiated but required for other types e.g `Entity` or `Thing`
- `primary`: types that are usually the root of a tree generated from a file e.g. `Article`, `Datatable`, `Collection`
- `secondary`: types usually only referred to by primary types e.g. `Organization` is used for the `publisher` property on a `Article`
- `tertiary`: types usually only referred to by secondary types e.g. `ContactPoint` is used for the `contactPoints` property on an `Organization`

## The `status` keyword

A RECOMMENDED custom keyword to indicate the development status of a type schema e.g.

- `experimental`: new types (i.e. not defined on schema.org or elsewhere) that are still under development
- `unstable`: types that are defined elsewhere (e.g. on http://bioschemas.org) but for which the schema is still being developed
- `stable`: types for which the schema definition can be considered stable

## The `description` keyword

You MUST add a description for all types and properties. Descriptions must be plain text and less than 120 characters. We apply this rule so that descriptions can be rendered in a variety on contexts including documentation strings in a variety of languages. If you need to add more details, or want to use Markdown, put it in the `$comment` property.

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

## The `codec` keyword

A custom keyword which allow you to define allowable shorthand strings for a property or types. The value of the keyword is the name of a codec to use. Parsers always take a string but differ in the type that they produce, for example:

- `ssi`: decodes a space separated list of items to an array of strings
- `csi`: decodes a comma separated list of items to an array of strings
- `person`: decodes a personal name, email or url to a `Person`

You can specify a codec for both types and properties. To specify a codec for a type, add the `codec` keyword at the top level e.g.

```yaml
title: Person
---
codec: person
```

You can specify a codec for a property using `anyOf`. For example, to allow `givenNames` to de provided as either a space separated values string or as an array of strings.

```yaml
title: Person
...
properties:
  ...
  givenNames:
    ...
    anyOf:
      - codec: ssi
      - type: array
        items:
          type: strings
```
