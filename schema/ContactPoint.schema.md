# `ContactPoint`

## Extends

`ContactPoint` extends `Thing`.

## Related

### JATS

`ContactPoint` is analagous, and structurally similar to, the JATS [`<corresp>`](https://jats.nlm.nih.gov/archiving/tag-library/1.1/element/corresp.html) element.

A journal author `<corresp>` links to a number of JATS elements which
are also properties of `ContactPoint` in Stencila schema. These JATS elements are:


| Stencila Schema    | JATS               |
|--------------------|--------------------|
| availableLanguages | @xml:langLanguage* |
| emails             | email              |
| telephone          | phone              |

\* `@xml:langLanguage` is an attribute describing the language of the intellectual content of the element (for which this is an attribute).

### HTML5

`ContactPoint` is analagous to the HTML5 [`<address>`](https://dev.w3.org/html5/html-author/#the-address-element) element. The `<address>` element represents the contact information for the section it applies to.
