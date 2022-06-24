# Nontextual Annotation

**Inline text that has a non-textual annotation.**

Use cases include annotating spelling errors, denoting proper names in Chinese text, representing underline text, and other forms of annotation. See - https://developer.mozilla.org/en-US/docs/Web/HTML/Element/u - http://html5doctor.com/u-element/ - https://jats.nlm.nih.gov/archiving/tag-library/1.1/element/underline.html - https://github.com/jgm/pandoc-types/blob/master/src/Text/Pandoc/Definition.hs#L320

## Properties

| Name        | `@id`                                                       | Type                                       | Description                         | Inherited from      |
| ----------- | ----------------------------------------------------------- | ------------------------------------------ | ----------------------------------- | ------------------- |
| **content** | [stencila:content](https://schema.stenci.la/content.jsonld) | Array of [InlineContent](InlineContent.md) | The content that is marked.         | [Mark](Mark.md)     |
| id          | [schema:id](https://schema.org/id)                          | string                                     | The identifier for this item.       | [Entity](Entity.md) |
| meta        | [stencila:meta](https://schema.stenci.la/meta.jsonld)       | object                                     | Metadata associated with this item. | [Entity](Entity.md) |

## Related

- Parent: [Mark](Mark.md)
- Descendants: None

## Available as

- [JSON-LD](https://schema.stenci.la/NontextualAnnotation.jsonld)
- [JSON Schema](https://schema.stenci.la/v1/NontextualAnnotation.schema.json)
- Python [`class NontextualAnnotation`](https://stencila.github.io/schema/python/docs/types.html#schema.types.NontextualAnnotation)
- TypeScript [`interface NontextualAnnotation`](https://stencila.github.io/schema/ts/docs/interfaces/nontextualannotation.html)
- R [`class NontextualAnnotation`](https://cran.r-project.org/web/packages/stencilaschema/stencilaschema.pdf)
- Rust [`struct NontextualAnnotation`](https://docs.rs/stencila-schema/latest/stencila_schema/struct.NontextualAnnotation.html)

## Source

This documentation was generated from [NontextualAnnotation.schema.yaml](https://github.com/stencila/stencila/blob/master/schema/schema/NontextualAnnotation.schema.yaml).
