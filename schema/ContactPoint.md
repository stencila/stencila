---
title: Contact Point
authors: []
---

include: ../public/ContactPoint.schema.md
:::
A contact point—for example, a R&D department. https&#x3A;//schema.org/ContactPoint.

| Entity       | type               | The name of the type and all descendant types.                                                                 | string |
| ------------ | ------------------ | -------------------------------------------------------------------------------------------------------------- | ------ |
| Entity       | id                 | The identifier for this item.                                                                                  | string |
| Thing        | alternateNames     | Alternate names (aliases) for the item.                                                                        | array  |
| Thing        | description        | A description of the item.                                                                                     | string |
| Thing        | meta               | Metadata associated with this item.                                                                            | object |
| Thing        | name               | The name of the item.                                                                                          | string |
| Thing        | url                | The URL of the item.                                                                                           | string |
| ContactPoint | availableLanguages | Languages (human not programming) in which it is possible to communicate with the organization/department etc. |        |
| array        |                    |                                                                                                                |        |
| ContactPoint | emails             | Email address for correspondence. It must be provided in a valid email format (eg. info@example.com ).         |        |
| array        |                    |                                                                                                                |        |
| ContactPoint | telephone          | "Phone contact number. Accepted formats: +44 123455, (02)12345, 006645667."                                    |        |
| string       |                    |                                                                                                                |        |

:::

The `ContactPoint` type allows you to provide details about an contact information such as languages (human) available for communication, emails and telephone numbers. This type of often used to describe the `ContactPoints` of an [`Organization`](/Organization).

# Examples

A contact point with email, telephone and two supported languages.

```json import=ex1
{
  "type": "ContactPoint",
  "availableLanguages": ["English", "Māori"],
  "emails": ["office@otago.ac.nz"],
  "telephone": "00641234567"
}
```

# Encodings

include: ../docs/type-encodings-intro.md
:::
This section describes common encodings for this node type. These samples are generated from the above examples by [Encoda](https://stencila.github.io/encoda), but you can also author them in each format.
:::

The examples below are based on the University of Otago and show how contact points for this organization can be specified. [The University of Otago](https://www.otago.ac.nz/) is the oldest university in Aotearoa New Zealand.

## JATS

`ContactPoint` is analogous, and structurally similar to, the JATS [`<corresp>`](https://jats.nlm.nih.gov/archiving/tag-library/1.1/element/corresp.html) element.

A journal author `<corresp>` links to a number of JATS elements which are also properties of `ContactPoint` in Stencila schema. These JATS elements are:

| Stencila Schema    | JATS                |
| ------------------ | ------------------- |
| availableLanguages | @xml:langLanguage\* |
| emails             | email               |
| telephone          | phone               |

\* `@xml:langLanguage` is an attribute describing the language of the intellectual content of the element (for which this is an attribute).

```jats export=ex1


```

# HTML5

`ContactPoint` is analogous to the HTML5 [`<address>`](https://dev.w3.org/html5/html-author/#the-address-element) element. The `<address>` element represents the contact information for the section it applies to.

```html export=ex1
<stencila-thing>
  {type:'ContactPoint',availableLanguages:['English','Māori'],emails:['office@otago.ac.nz'],telephone:'00641234567'}
</stencila-thing>
```

[//]: # 'WIP: Needs JATS Fixes'
