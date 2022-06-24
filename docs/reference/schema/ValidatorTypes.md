# Validator Types

**All type schemas that are derived from Validator**

This schema type is marked as **experimental** 🧪 and is subject to change.

## Members

| `@id`                                                                           | Type                                      | Description                                                            |
| ------------------------------------------------------------------------------- | ----------------------------------------- | ---------------------------------------------------------------------- |
| [stencila:Validator](https://schema.stenci.la/Validator.jsonld)                 | [Validator](Validator.md)                 | A base for all validator types.                                        |
| [stencila:ArrayValidator](https://schema.stenci.la/ArrayValidator.jsonld)       | [ArrayValidator](ArrayValidator.md)       | A validator specifying constraints on an array node.                   |
| [stencila:BooleanValidator](https://schema.stenci.la/BooleanValidator.jsonld)   | [BooleanValidator](BooleanValidator.md)   | A schema specifying that a node must be a boolean value.               |
| [stencila:ConstantValidator](https://schema.stenci.la/ConstantValidator.jsonld) | [ConstantValidator](ConstantValidator.md) | A validator specifying a constant value that a node must have.         |
| [stencila:EnumValidator](https://schema.stenci.la/EnumValidator.jsonld)         | [EnumValidator](EnumValidator.md)         | A schema specifying that a node must be one of several values.         |
| [stencila:IntegerValidator](https://schema.stenci.la/IntegerValidator.jsonld)   | [IntegerValidator](IntegerValidator.md)   | A validator specifying the constraints on an integer node.             |
| [stencila:NumberValidator](https://schema.stenci.la/NumberValidator.jsonld)     | [NumberValidator](NumberValidator.md)     | A validator specifying the constraints on a numeric node.              |
| [stencila:StringValidator](https://schema.stenci.la/StringValidator.jsonld)     | [StringValidator](StringValidator.md)     | A schema specifying constraints on a string node.                      |
| [stencila:TupleValidator](https://schema.stenci.la/TupleValidator.jsonld)       | [TupleValidator](TupleValidator.md)       | A validator specifying constraints on an array of heterogeneous items. |

## Available as

- [JSON-LD](https://schema.stenci.la/stencila.jsonld)
- [JSON Schema](https://schema.stenci.la/v1/ValidatorTypes.schema.json)
