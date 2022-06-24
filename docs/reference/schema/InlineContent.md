# Inline Content

**Union type for valid inline content.**

The order of these types is important because it determines the order of attempted coercion (this is particularly important for primitive types).

## Members

| `@id`                                                                                 | Type                                            | Description                                                                           |
| ------------------------------------------------------------------------------------- | ----------------------------------------------- | ------------------------------------------------------------------------------------- |
| [schema:AudioObject](https://schema.org/AudioObject)                                  | [AudioObject](AudioObject.md)                   | An audio file                                                                         |
| [stencila:Cite](https://schema.stenci.la/Cite.jsonld)                                 | [Cite](Cite.md)                                 | A reference to a CreativeWork that is cited in another CreativeWork.                  |
| [stencila:CiteGroup](https://schema.stenci.la/CiteGroup.jsonld)                       | [CiteGroup](CiteGroup.md)                       | A group of Cite nodes.                                                                |
| [stencila:CodeExpression](https://schema.stenci.la/CodeExpression.jsonld)             | [CodeExpression](CodeExpression.md)             | An executable programming code expression.                                            |
| [stencila:CodeFragment](https://schema.stenci.la/CodeFragment.jsonld)                 | [CodeFragment](CodeFragment.md)                 | Inline code.                                                                          |
| [stencila:Delete](https://schema.stenci.la/Delete.jsonld)                             | [Delete](Delete.md)                             | Content that is marked for deletion                                                   |
| [stencila:Emphasis](https://schema.stenci.la/Emphasis.jsonld)                         | [Emphasis](Emphasis.md)                         | Emphasised content.                                                                   |
| [schema:ImageObject](https://schema.org/ImageObject)                                  | [ImageObject](ImageObject.md)                   | An image file.                                                                        |
| [stencila:Link](https://schema.stenci.la/Link.jsonld)                                 | [Link](Link.md)                                 | A hyperlink to other pages, sections within the same document, resources, or any URL. |
| [stencila:MathFragment](https://schema.stenci.la/MathFragment.jsonld)                 | [MathFragment](MathFragment.md)                 | A fragment of math, e.g a variable name, to be treated as inline content.             |
| [stencila:NontextualAnnotation](https://schema.stenci.la/NontextualAnnotation.jsonld) | [NontextualAnnotation](NontextualAnnotation.md) | Inline text that has a non-textual annotation.                                        |
| [stencila:Note](https://schema.stenci.la/Note.jsonld)                                 | [Note](Note.md)                                 | Additional content which is not part of the main content of a document.               |
| [stencila:Parameter](https://schema.stenci.la/Parameter.jsonld)                       | [Parameter](Parameter.md)                       | A parameter of a document or function.                                                |
| [stencila:Quote](https://schema.stenci.la/Quote.jsonld)                               | [Quote](Quote.md)                               | Inline, quoted content.                                                               |
| [stencila:Strong](https://schema.stenci.la/Strong.jsonld)                             | [Strong](Strong.md)                             | Strongly emphasised content.                                                          |
| [stencila:Subscript](https://schema.stenci.la/Subscript.jsonld)                       | [Subscript](Subscript.md)                       | Subscripted content.                                                                  |
| [stencila:Superscript](https://schema.stenci.la/Superscript.jsonld)                   | [Superscript](Superscript.md)                   | Superscripted content.                                                                |
| [schema:VideoObject](https://schema.org/VideoObject)                                  | [VideoObject](VideoObject.md)                   | A video file.                                                                         |
| [stencila:Null](https://schema.stenci.la/Null.jsonld)                                 | [Null](Null.md)                                 | The null value                                                                        |
| [schema:Boolean](https://schema.org/Boolean)                                          | [Boolean](Boolean.md)                           | A value that is either true or false                                                  |
| [schema:Integer](https://schema.org/Integer)                                          | [Integer](Integer.md)                           | A value that is a integer                                                             |
| [schema:Number](https://schema.org/Number)                                            | [Number](Number.md)                             | A value that is a number                                                              |
| [schema:Text](https://schema.org/Text)                                                | [String](String.md)                             | A value comprised of a string of characters                                           |

## Available as

- [JSON-LD](https://schema.stenci.la/stencila.jsonld)
- [JSON Schema](https://schema.stenci.la/v1/InlineContent.schema.json)

## Source

This documentation was generated from [InlineContent.schema.yaml](https://github.com/stencila/stencila/blob/master/schema/InlineContent.schema.yaml).
