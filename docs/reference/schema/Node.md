# Node

**Union type for all types of nodes in this schema, including primitives and entities**

## Members

| `@id`                                                                         | Type                                    | Description                                                                 |
| ----------------------------------------------------------------------------- | --------------------------------------- | --------------------------------------------------------------------------- |
| [stencila:Entity](https://schema.stenci.la/Entity.jsonld)                     | [Entity](Entity.md)                     | The most simple compound (ie. non-atomic like `number`, `string` etc) type. |
| [stencila:ArrayValidator](https://schema.stenci.la/ArrayValidator.jsonld)     | [ArrayValidator](ArrayValidator.md)     | A validator specifying constraints on an array node.                        |
| [schema:Article](https://schema.org/Article)                                  | [Article](Article.md)                   | An article, including news and scholarly articles.                          |
| [schema:AudioObject](https://schema.org/AudioObject)                          | [AudioObject](AudioObject.md)           | An audio file                                                               |
| [stencila:BooleanValidator](https://schema.stenci.la/BooleanValidator.jsonld) | [BooleanValidator](BooleanValidator.md) | A schema specifying that a node must be a boolean value.                    |
| [schema:Brand](https://schema.org/Brand)                                      | [Brand](Brand.md)                       | A brand used by an organization or person for labeling a product,           |

product group, or similar.
|
| [stencila:CitationIntentEnumeration](https://schema.stenci.la/CitationIntentEnumeration.jsonld) | [CitationIntentEnumeration](CitationIntentEnumeration.md) | The type or nature of a citation, both factually and rhetorically. |
| [stencila:Cite](https://schema.stenci.la/Cite.jsonld) | [Cite](Cite.md) | A reference to a CreativeWork that is cited in another CreativeWork. |
| [stencila:CiteGroup](https://schema.stenci.la/CiteGroup.jsonld) | [CiteGroup](CiteGroup.md) | A group of Cite nodes. |
| [schema:Claim](https://schema.org/Claim) | [Claim](Claim.md) | A claim represents specific reviewable facts or statements. |
| [stencila:Code](https://schema.stenci.la/Code.jsonld) | [Code](Code.md) | Base type for non-executable (e.g. `CodeBlock`) and executable (e.g. `CodeExpression`) code nodes. |
| [stencila:CodeBlock](https://schema.stenci.la/CodeBlock.jsonld) | [CodeBlock](CodeBlock.md) | A code block. |
| [stencila:CodeChunk](https://schema.stenci.la/CodeChunk.jsonld) | [CodeChunk](CodeChunk.md) | A executable chunk of code. |
| [stencila:CodeError](https://schema.stenci.la/CodeError.jsonld) | [CodeError](CodeError.md) | An error that occurred when parsing, compiling or executing a Code node. |
| [stencila:CodeExecutable](https://schema.stenci.la/CodeExecutable.jsonld) | [CodeExecutable](CodeExecutable.md) | Base type for executable code nodes (i.e. `CodeChunk` and `CodeExpression`). |
| [stencila:CodeExpression](https://schema.stenci.la/CodeExpression.jsonld) | [CodeExpression](CodeExpression.md) | An executable programming code expression. |
| [stencila:CodeFragment](https://schema.stenci.la/CodeFragment.jsonld) | [CodeFragment](CodeFragment.md) | Inline code. |
| [schema:Collection](https://schema.org/Collection) | [Collection](Collection.md) | A collection of CreativeWorks or other artifacts. |
| [schema:Comment](https://schema.org/Comment) | [Comment](Comment.md) | A comment on an item, e.g on a Article, or SoftwareSourceCode. |
| [stencila:ConstantValidator](https://schema.stenci.la/ConstantValidator.jsonld) | [ConstantValidator](ConstantValidator.md) | A validator specifying a constant value that a node must have. |
| [schema:ContactPoint](https://schema.org/ContactPoint) | [ContactPoint](ContactPoint.md) | A contact point, usually within an organization. |
| [schema:CreativeWork](https://schema.org/CreativeWork) | [CreativeWork](CreativeWork.md) | A creative work, including books, movies, photographs, software programs, etc.
|
| [stencila:Datatable](https://schema.stenci.la/Datatable.jsonld) | [Datatable](Datatable.md) | A table of data. |
| [stencila:DatatableColumn](https://schema.stenci.la/DatatableColumn.jsonld) | [DatatableColumn](DatatableColumn.md) | A column of data within a Datatable. |
| [schema:Date](https://schema.org/Date) | [Date](Date.md) | A date encoded as a ISO 8601 string. |
| [schema:DefinedTerm](https://schema.org/DefinedTerm) | [DefinedTerm](DefinedTerm.md) | A word, name, acronym, phrase, etc. with a formal definition. |
| [stencila:Delete](https://schema.stenci.la/Delete.jsonld) | [Delete](Delete.md) | Content that is marked for deletion |
| [stencila:Emphasis](https://schema.stenci.la/Emphasis.jsonld) | [Emphasis](Emphasis.md) | Emphasised content. |
| [stencila:EnumValidator](https://schema.stenci.la/EnumValidator.jsonld) | [EnumValidator](EnumValidator.md) | A schema specifying that a node must be one of several values. |
| [schema:Enumeration](https://schema.org/Enumeration) | [Enumeration](Enumeration.md) | Lists or enumerations, for example, a list of cuisines or music genres, etc. |
| [stencila:Figure](https://schema.stenci.la/Figure.jsonld) | [Figure](Figure.md) | Encapsulates one or more images, videos, tables, etc, and provides captions and labels for them. |
| [stencila:Function](https://schema.stenci.la/Function.jsonld) | [Function](Function.md) | A function with a name, which might take Parameters and return a value of a certain type. |
| [schema:Grant](https://schema.org/Grant) | [Grant](Grant.md) | A grant, typically financial or otherwise quantifiable, of resources. |
| [stencila:Heading](https://schema.stenci.la/Heading.jsonld) | [Heading](Heading.md) | A heading. |
| [schema:ImageObject](https://schema.org/ImageObject) | [ImageObject](ImageObject.md) | An image file. |
| [stencila:Include](https://schema.stenci.la/Include.jsonld) | [Include](Include.md) | Include content from an external source (e.g. file, URL). |
| [stencila:IntegerValidator](https://schema.stenci.la/IntegerValidator.jsonld) | [IntegerValidator](IntegerValidator.md) | A validator specifying the constraints on an integer node. |
| [stencila:Link](https://schema.stenci.la/Link.jsonld) | [Link](Link.md) | A hyperlink to other pages, sections within the same document, resources, or any URL. |
| [schema:ItemList](https://schema.org/ItemList) | [List](List.md) | A list of items. |
| [schema:ListItem](https://schema.org/ListItem) | [ListItem](ListItem.md) | A single item in a list. |
| [stencila:Mark](https://schema.stenci.la/Mark.jsonld) | [Mark](Mark.md) | A base class for nodes that mark some other inline content
in some way (e.g. as being emphasised, or quoted).
|
| [stencila:Math](https://schema.stenci.la/Math.jsonld) | [Math](Math.md) | A mathematical variable or equation. |
| [stencila:MathBlock](https://schema.stenci.la/MathBlock.jsonld) | [MathBlock](MathBlock.md) | A block of math, e.g an equation, to be treated as block content. |
| [stencila:MathFragment](https://schema.stenci.la/MathFragment.jsonld) | [MathFragment](MathFragment.md) | A fragment of math, e.g a variable name, to be treated as inline content. |
| [schema:MediaObject](https://schema.org/MediaObject) | [MediaObject](MediaObject.md) | A media object, such as an image, video, or audio object embedded in a web page or a
downloadable dataset.
|
| [schema:MonetaryGrant](https://schema.org/MonetaryGrant) | [MonetaryGrant](MonetaryGrant.md) | A monetary grant. |
| [stencila:NontextualAnnotation](https://schema.stenci.la/NontextualAnnotation.jsonld) | [NontextualAnnotation](NontextualAnnotation.md) | Inline text that has a non-textual annotation. |
| [stencila:Note](https://schema.stenci.la/Note.jsonld) | [Note](Note.md) | Additional content which is not part of the main content of a document. |
| [stencila:NumberValidator](https://schema.stenci.la/NumberValidator.jsonld) | [NumberValidator](NumberValidator.md) | A validator specifying the constraints on a numeric node. |
| [schema:Organization](https://schema.org/Organization) | [Organization](Organization.md) | An organization such as a school, NGO, corporation, club, etc. |
| [stencila:Paragraph](https://schema.stenci.la/Paragraph.jsonld) | [Paragraph](Paragraph.md) | Paragraph |
| [stencila:Parameter](https://schema.stenci.la/Parameter.jsonld) | [Parameter](Parameter.md) | A parameter of a document or function. |
| [schema:Periodical](https://schema.org/Periodical) | [Periodical](Periodical.md) | A periodical publication. |
| [schema:Person](https://schema.org/Person) | [Person](Person.md) | A person (alive, dead, undead, or fictional). |
| [schema:PostalAddress](https://schema.org/PostalAddress) | [PostalAddress](PostalAddress.md) | A physical mailing address. |
| [schema:Product](https://schema.org/Product) | [Product](Product.md) | Any offered product or service. For example, a pair of shoes;
a haircut; or an episode of a TV show streamed online.
|
| [schema:PropertyValue](https://schema.org/PropertyValue) | [PropertyValue](PropertyValue.md) | A property-value pair. |
| [schema:PublicationIssue](https://schema.org/PublicationIssue) | [PublicationIssue](PublicationIssue.md) | A part of a successively published publication such as a periodical or publication
volume, often numbered.
|
| [schema:PublicationVolume](https://schema.org/PublicationVolume) | [PublicationVolume](PublicationVolume.md) | A part of a successively published publication such as a periodical or multi-volume work.
|
| [stencila:Quote](https://schema.stenci.la/Quote.jsonld) | [Quote](Quote.md) | Inline, quoted content. |
| [stencila:QuoteBlock](https://schema.stenci.la/QuoteBlock.jsonld) | [QuoteBlock](QuoteBlock.md) | A section quoted from somewhere else.
|
| [schema:Review](https://schema.org/Review) | [Review](Review.md) | A review of an item, e.g of an Article, or SoftwareSourceCode. |
| [schema:SoftwareApplication](https://schema.org/SoftwareApplication) | [SoftwareApplication](SoftwareApplication.md) | A software application.
|
| [stencila:SoftwareEnvironment](https://schema.stenci.la/SoftwareEnvironment.jsonld) | [SoftwareEnvironment](SoftwareEnvironment.md) | A computational environment. |
| [stencila:SoftwareSession](https://schema.stenci.la/SoftwareSession.jsonld) | [SoftwareSession](SoftwareSession.md) | Definition of a compute session, including its software and compute resource
requirements and status.
|
| [schema:SoftwareSourceCode](https://schema.org/SoftwareSourceCode) | [SoftwareSourceCode](SoftwareSourceCode.md) | Computer programming source code. Example: Full (compile ready) solutions, code snippet samples, scripts, templates.
|
| [stencila:StringValidator](https://schema.stenci.la/StringValidator.jsonld) | [StringValidator](StringValidator.md) | A schema specifying constraints on a string node. |
| [stencila:Strong](https://schema.stenci.la/Strong.jsonld) | [Strong](Strong.md) | Strongly emphasised content. |
| [stencila:Subscript](https://schema.stenci.la/Subscript.jsonld) | [Subscript](Subscript.md) | Subscripted content. |
| [stencila:Superscript](https://schema.stenci.la/Superscript.jsonld) | [Superscript](Superscript.md) | Superscripted content. |
| [schema:Table](https://schema.org/Table) | [Table](Table.md) | A table. |
| [stencila:TableCell](https://schema.stenci.la/TableCell.jsonld) | [TableCell](TableCell.md) | A cell within a `Table`.
|
| [stencila:TableRow](https://schema.stenci.la/TableRow.jsonld) | [TableRow](TableRow.md) | A row within a Table. |
| [stencila:ThematicBreak](https://schema.stenci.la/ThematicBreak.jsonld) | [ThematicBreak](ThematicBreak.md) | A thematic break, such as a scene change in a story, a transition to another topic, or a new document.
|
| [schema:Thing](https://schema.org/Thing) | [Thing](Thing.md) | The most generic type of item. |
| [stencila:TupleValidator](https://schema.stenci.la/TupleValidator.jsonld) | [TupleValidator](TupleValidator.md) | A validator specifying constraints on an array of heterogeneous items. |
| [stencila:Validator](https://schema.stenci.la/Validator.jsonld) | [Validator](Validator.md) | A base for all validator types. |
| [stencila:Variable](https://schema.stenci.la/Variable.jsonld) | [Variable](Variable.md) | A variable representing a name / value pair. |
| [schema:VideoObject](https://schema.org/VideoObject) | [VideoObject](VideoObject.md) | A video file. |
| [stencila:VolumeMount](https://schema.stenci.la/VolumeMount.jsonld) | [VolumeMount](VolumeMount.md) | Describes a volume mount from a host to container.
|
| [stencila:Null](https://schema.stenci.la/Null.jsonld) | [Null](Null.md) | The null value |
| [schema:Boolean](https://schema.org/Boolean) | [Boolean](Boolean.md) | A value that is either true or false |
| [schema:Integer](https://schema.org/Integer) | [Integer](Integer.md) | A value that is a integer |
| [schema:Number](https://schema.org/Number) | [Number](Number.md) | A value that is a number |
| [schema:Text](https://schema.org/Text) | [String](String.md) | A value comprised of a string of characters |
| [stencila:Object](https://schema.stenci.la/Object.jsonld) | [Object](Object.md) | A value comprised of keyed primitive nodes. |
| [stencila:Array](https://schema.stenci.la/Array.jsonld) | [Array](Array.md) | A value comprised of other primitive nodes |

## Available as

- [JSON-LD](https://schema.stenci.la/stencila.jsonld)
- [JSON Schema](https://schema.stenci.la/v1/Node.schema.json)

## Source

This documentation was generated from [Node.schema.yaml](https://github.com/stencila/stencila/blob/master/schema/schema/Node.schema.yaml).
