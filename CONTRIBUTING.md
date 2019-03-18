

For each type, there are two files:

- `<type>.schema.yaml` is the JSON Schema, written in YAML, for the type.

- `<type>.md` is a description of design considerations for the schema and includes sections on analagous types in other schemas


The Open Document Format for Office Applications (OpenDocument) http://docs.oasis-open.org/office/v1.2/os/OpenDocument-v1.2-os-part1.html


### Authoring a type schema

The schema for a type is defined, using JSON Schema, in a `<type>.schema.yaml` file. We YAML, which is a superset of JSON, because it is more readable than JSON.

See the excellent [Understanding JSON Schema](https://json-schema.org/understanding-json-schema/) for guides on declaring a JSON Schema.

#### The essentials

Each type schema should begin with the JSON Schema version, the `$id`, `title` and description of the type e.g.

```yaml
$schema: http://json-schema.org/draft-07/schema#
$id: https://stencila.github.com/schema#Organization
title: Organization
description: An organization such as a school, NGO, corporation, club, etc. https://schema.org/Organization.
```

#### The type `@id`

To include the type in the JSON-LD `@context` you should also declare the `@id` property using the format `<context>:<type>`. Note that because this property name begins with the special character `@`, that it needs to be surrounded by quotes e.g.

```yaml
'@id': schema:Organization
```

Currently, the Stencila context allows you to refer to the following external contexts:

```
schema: https://schema.org/
bioschemas: http://bioschemas.org
codemeta: https://doi.org/10.5063/schema/codemeta-2.0
```

For example, the type schema to represent a laboratory protocol might use the `@id` of the Bioschemas [`LabProtocol`](http://bioschemas.org/specifications/LabProtocol/).

```yaml
'@id': bioschemas:LabProtocol
```

#### Extending other types

JSON Schema allows for a limited form of inheritance using the `allOf` keyword. If the type you are declaring extends another type, you should do something like:

```yaml
allOf:
  - $ref: Thing.schema.yaml
  - properties:
      ...
```

#### The property `@id`

Just as for types, properties of types can be linked to the other contexts using a `@id`. For example,

```yaml
  - properties:
      address:
        '@id': schema:address
        type: string
```

