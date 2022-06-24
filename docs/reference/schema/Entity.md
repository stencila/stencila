# Entity

**The most simple compound (ie. non-atomic like `number`, `string` etc) type.**

This type exists mainly to have a more simple base class than schema.org's `Thing`. This schema includes special properties that are analogous to JSON-LDs `@type` and `@id`, as well as a `meta` property that can be used by applications for ad-hoc extensions.

## Properties

| Name | `@id`                                                 | Type   | Description                         | Inherited from      |
| ---- | ----------------------------------------------------- | ------ | ----------------------------------- | ------------------- |
| id   | [schema:id](https://schema.org/id)                    | string | The identifier for this item.       | [Entity](Entity.md) |
| meta | [stencila:meta](https://schema.stenci.la/meta.jsonld) | object | Metadata associated with this item. | [Entity](Entity.md) |

## Related

- Parent: None
- Descendants: [ArrayValidator](ArrayValidator.md), [Article](Article.md), [AudioObject](AudioObject.md), [BooleanValidator](BooleanValidator.md), [Brand](Brand.md), [CitationIntentEnumeration](CitationIntentEnumeration.md), [Cite](Cite.md), [CiteGroup](CiteGroup.md), [Claim](Claim.md), [Code](Code.md), [CodeBlock](CodeBlock.md), [CodeChunk](CodeChunk.md), [CodeError](CodeError.md), [CodeExecutable](CodeExecutable.md), [CodeExpression](CodeExpression.md), [CodeFragment](CodeFragment.md), [Collection](Collection.md), [Comment](Comment.md), [ConstantValidator](ConstantValidator.md), [ContactPoint](ContactPoint.md), [CreativeWork](CreativeWork.md), [Datatable](Datatable.md), [DatatableColumn](DatatableColumn.md), [Date](Date.md), [DefinedTerm](DefinedTerm.md), [Delete](Delete.md), [Emphasis](Emphasis.md), [EnumValidator](EnumValidator.md), [Enumeration](Enumeration.md), [Figure](Figure.md), [Function](Function.md), [Grant](Grant.md), [Heading](Heading.md), [ImageObject](ImageObject.md), [Include](Include.md), [IntegerValidator](IntegerValidator.md), [Link](Link.md), [List](List.md), [ListItem](ListItem.md), [Mark](Mark.md), [Math](Math.md), [MathBlock](MathBlock.md), [MathFragment](MathFragment.md), [MediaObject](MediaObject.md), [MonetaryGrant](MonetaryGrant.md), [NontextualAnnotation](NontextualAnnotation.md), [Note](Note.md), [NumberValidator](NumberValidator.md), [Organization](Organization.md), [Paragraph](Paragraph.md), [Parameter](Parameter.md), [Periodical](Periodical.md), [Person](Person.md), [PostalAddress](PostalAddress.md), [Product](Product.md), [PropertyValue](PropertyValue.md), [PublicationIssue](PublicationIssue.md), [PublicationVolume](PublicationVolume.md), [Quote](Quote.md), [QuoteBlock](QuoteBlock.md), [Review](Review.md), [SoftwareApplication](SoftwareApplication.md), [SoftwareEnvironment](SoftwareEnvironment.md), [SoftwareSession](SoftwareSession.md), [SoftwareSourceCode](SoftwareSourceCode.md), [StringValidator](StringValidator.md), [Strong](Strong.md), [Subscript](Subscript.md), [Superscript](Superscript.md), [Table](Table.md), [TableCell](TableCell.md), [TableRow](TableRow.md), [ThematicBreak](ThematicBreak.md), [Thing](Thing.md), [TupleValidator](TupleValidator.md), [Validator](Validator.md), [Variable](Variable.md), [VideoObject](VideoObject.md), [VolumeMount](VolumeMount.md)

## Available as

- [JSON-LD](https://schema.stenci.la/Entity.jsonld)
- [JSON Schema](https://schema.stenci.la/v1/Entity.schema.json)
- Python [`class Entity`](https://stencila.github.io/schema/python/docs/types.html#schema.types.Entity)
- TypeScript [`interface Entity`](https://stencila.github.io/schema/ts/docs/interfaces/entity.html)
- R [`class Entity`](https://cran.r-project.org/web/packages/stencilaschema/stencilaschema.pdf)
- Rust [`struct Entity`](https://docs.rs/stencila-schema/latest/stencila_schema/struct.Entity.html)

## Source

This documentation was generated from [Entity.schema.yaml](https://github.com/stencila/stencila/blob/master/schema/schema/Entity.schema.yaml).
