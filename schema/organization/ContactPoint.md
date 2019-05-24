# ContactPoint

The `ContactPoint` type allows you to provide details about an contact information such as languages (human) available for communication, emails and telephone numbers. This type of often used to describe the `ContactPoints` of an [`Organization`](/Organization).

## Examples

The examples below are based on the University of Otago and show how contact points for this organization can be specified. [The University of Otago](https://www.otago.ac.nz/) is the oldest university in Aotearoa New Zealand.

```json
{
  "type": "ContactPoint",
  "availableLanguages": ["English", "Māori"],
  "emails": ["office@otago.ac.nz"],
  "telephone": "00641234567"
}
```

## Related

### JATS

`ContactPoint` is analagous, and structurally similar to, the JATS [`<corresp>`](https://jats.nlm.nih.gov/archiving/tag-library/1.1/element/corresp.html) element.

A journal author `<corresp>` links to a number of JATS elements which
are also properties of `ContactPoint` in Stencila schema. These JATS elements are:

| Stencila Schema    | JATS                |
| ------------------ | ------------------- |
| availableLanguages | @xml:langLanguage\* |
| emails             | email               |
| telephone          | phone               |

\* `@xml:langLanguage` is an attribute describing the language of the intellectual content of the element (for which this is an attribute).

### HTML5

`ContactPoint` is analagous to the HTML5 [`<address>`](https://dev.w3.org/html5/html-author/#the-address-element) element. The `<address>` element represents the contact information for the section it applies to.
