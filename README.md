# Thema

> 🎨 Semantic themes for use with Stencila [`encoda`](https://github.com/stencila/encoda).

[![Build Status](https://travis-ci.org/stencila/thema.svg?branch=master)](https://travis-ci.org/stencila/thema)
[![Code coverage](https://codecov.io/gh/stencila/thema/branch/master/graph/badge.svg)](https://codecov.io/gh/stencila/thema)
[![Visual Regression Tests](https://img.shields.io/badge/Argos%20CI-Visual%20Regression%20tests-informational?style=flat)](https://www.argos-ci.com/stencila/thema/builds)

- [Quick Start](#quick-start)
- [Available Themes](#available-themes)
- [Theme Structure](#theme-structure)
- [Develop](#develop)
  - [Prerequisites](#prerequisites)
  - [Getting started](#getting-started)
  - [Creating a new theme](#creating-a-new-theme)
    - [Scripted creation](#scripted-creation)
    - [Manual creation](#manual-creation)
    - [Approaches](#approaches)
  - [Extensions](#extensions)
- [Notes](#notes)
  - [Generated code](#generated-code)
  - [Testing](#testing)
- [Acknowledgments](#acknowledgments)
- [Utilities API](#utilities-api)
  - [Functions](#functions)
  - [ready(func)](#readyfunc)
  - [first(elem, selector) ⇒ <code>Element</code> \| <code>null</code>](#firstelem-selector-%e2%87%92-codeelementcode--codenullcode)
  - [select(elem, selector) ⇒ <code>Array.&lt;Element&gt;</code>](#selectelem-selector-%e2%87%92-codearrayltelementgtcode)
  - [create(spec, ...children) ⇒ <code>Element</code>](#createspec-children-%e2%87%92-codeelementcode)
  - [attr(target, name, value) ⇒ <code>string</code> \| <code>null</code> \| <code>undefined</code>](#attrtarget-name-value-%e2%87%92-codestringcode--codenullcode--codeundefinedcode)
  - [text(target, value) ⇒ <code>string</code> \| <code>null</code> \| <code>undefined</code>](#texttarget-value-%e2%87%92-codestringcode--codenullcode--codeundefinedcode)
  - [append(target, ...elems)](#appendtarget-elems)
  - [prepend(target, ...elems)](#prependtarget-elems)
  - [before(target, ...elems)](#beforetarget-elems)
  - [after(target, ...elems)](#aftertarget-elems)
  - [replace(target, ...elems)](#replacetarget-elems)
  - [wrap(target, elem)](#wraptarget-elem)

## Quick Start

```sh
npm install @stencila/thema
```

```css
/* myStyleSheet.css */
@import '@stencila/thema/dist/stencila/styles.css';
```

```js
/* myJavaScript.js */
@import '@stencila/thema/dist/stencila';
```

## Available Themes

- `stencila`: A custom designed theme suited for long-form reading,
  specifically research papers. It strives to deliver an optimal reading
  experience whilst giving figures and other media objects as much room as
  possible.
- `eLife`: Inspired by the eLife website, this theme provides as
  close of an approximation as possible for solely semantically structured
  documents such as Markdown.

## Theme Structure

There are two primary files inside each theme folder. The naming of these two
files is important, and they must not be changed since they are referred to
from `encoda`.

- `styles.css`: CSS and visual styles specific to the theme. We use PostCSS
  to compile the CSS, this is done to utilize PostCSS utilities such as
  autoprefixing vendor flags to selectors, and writing nested selectors.
- `index.ts`: Written in TypeScript, this file is loaded asynchronously. It is
  used to progressively enhance the theme with things like syntax highlighting
  of code blocks.

## Develop

### Prerequisites

- [`node`](https://nodejs.org/en/)
- [`npm`](https://www.npmjs.com)
- [`git`](https://git-scm.com)

### Getting started

The best way to get started is to develop CSS and JS for a theme with the live updating demo running.

Clone this repository,

```sh
git clone git@github.com:stencila/thema.git
cd thema
```

Install dependencies,

```sh
npm install
```

Run the development server,

```sh
npm run dev
```

Then open http://localhost:1234/index.html in your browser to view the demo.

### Creating a new theme

#### Scripted creation

The easiest way to create a new theme is:

```bash
npm run create:theme -- mytheme
```

Theme names should be all lower case, and start with a letter. This creates a new folder in `src/themes` and the following files:

- a `README.md` providing a description of the theme and notes for contributors,
- a `styles.css` file for the theme's CSS,
- a `index.ts` for any Typescript that the style may need

#### Manual creation

You can create this folder structure for your theme manually. If you prefer to use Javascript instead of Typescript, use a `index.js` file instead of `index.ts`. Then update the list of themes in `themes/themes.ts` and elsewhere using:

```bash
npm run update:themes
```

#### Approaches

There are three broad approaches to developing a new theme, each epitomized in three of the themes in this repository:

- the [`skeleton`](./src/themes/skeleton/README.md) approach: flesh things out yourself; start from scratch with nothing but be unaffected from changes to `shared` styles and scripts

- the [`bootstrap`](./src/themes/bootstrap/README.md) approach: reuse existing stylesheets from elsewhere by mapping between Thema's semantic selectors and existing selectors in those stylesheets

It is important to note that the `skeleton` and `bootstrap` themes are extremes of each of the approaches - they apply their approach to _all_ document node types. Depending on your theme, the best approach is probably some combination of these approaches for different node types e.g. starting from scratch for some nodes and using `shared` styles for others.

There are a few key rules enforced by Stylelint:

- All slectors must be descendants of a custom semantic selector. This reduces risks of a theme interfering with
- exsitng stylesheets on a website.
- Avoid hard-coded values for things such as font-sizes, colors, and fonts. Instead, use CSS variables, as these will
- allow simple theme overrides within the browser without having to rebuild the theme.
- Design your themes using a mobile-first approach, adding overrides to refine styles on larger screens.
  At the same time, it is highly recommended to also include print media overrides with your theme.
- These themes are primarily intended for rendering interactive articles and other relatively long forms of prose.
  - As such, good typography is paramount. Reading Matthew Butterick‘s [Typography in Ten
  - Minutes](https://practicaltypography.com/typography-in-ten-minutes.html) will give you a solid foundation and a
  - reference for crafting your own themes.

To tweak or adjust an existing theme, you may override some common CSS variables found in the themes.

Available CSS variables are:
--color-accent: Color for accent elements, primarily used to add a brand highlights to the theme.
--color-key: Color for body text, and other elements using the inherited body text color.
--color-neutral: Subtle color, usually shades of gray, for element borders and other subtle details.
--color-stock: Article/Page background color, but also used for other elements.
--font-family-body: Font-family for paragraphs and other non-headline elements
--font-family-heading: Font-family for paragraphs and other non-headline elements
--font-family-mono: Font-family for monospaced text elements such as \`pre\` and \`code\`.
--max-width-media: Maximum width for media content, including images and interactive Code Chunks.
--max-width: Max width for textual elements and other non-media content.

Note that not all themes make use of all available variables, and that some may expose additional options.
Please refer to the specific theme documentation.

Type selectors

For types defined in http://schema.org (e.g. \`Article\`), or extensions such as,
http://schema.stenci.la (e.g. \`CodeChunk\`), http://bioschemas.org (e.g. \`Taxon\`) etc.

Conventions:

- use the same upper camel case as in the schema the type is defined in
- use a \`[itemtype=...]\` selector if possible (i.e. if Encoda encodes it in HTML)

Property selectors

For properties of types defined in schemas. Note that
some of these select an entire container property e.g. \`authors\` and
selector for a class, and some select items in those properties
e.g. \`author\` and select for a \`itemprop\`.

Conventions:

- use the same lower camel case as in the schema the property is defined in
- use a \`.class\` selector for container properties
- use a \`[itemprop=...]\` selector for singular properties, or items of container properties

### Extensions

The [default set of Web Components](https://github.com/stencila/designa/tree/master/packages/components) to provide interactivity to document nodes.

Currently, the following node types have Web Components. Encoda will encode these nodes types as custom HTML elements that get hydrated into these components.

| Node type        | Custom element               |
| ---------------- | ---------------------------- |
| `CodeChunk`      | `<stencila-code-chunk>`      |
| `CodeExpression` | `<stencila-code-expression>` |

More components will be added over time. In the meantime, the "pseudo-components" in sibling folders to this one, provide styling for some other node types.

## Notes

- Theme authors should be able to override the styles of the web components as part of their theme.

### Generated code

Some files in the `src` directory are auto-generated and should not be edited manually. Generated files include:

- `src/themes/themes.ts`: from the list of sub-folders in the `themes` folder
- `src/examples/examples.ts`: from the functions defined in `generate/examples.ts`
- `src/examples/*`: files generated by those functions, usually using the `@stencila/encoda` package
- `src/selectors.css`: custom selectors from the JSON Schema in the `@stencila/schema` package

Run `npm run update` when you add new themes, addons, or examples, or upgrade one of those upstream packages. In particular, when Encoda is upgraded it is important to regenerate HTML for the examples - `npm run update` is called after `npm install` to ensure this.

### Testing

We use visual regression testing powered by [Sauce Labs](https://saucelabs.com), [Argos](https://www.argos-ci.com), and [WebdriverIO](https://webdriver.io).

As part of the continuous integration for this repository, for each push,

1. themes are [built on Travis](https://travis-ci.org/stencila/thema),
2. examples rendered in a browser running on Sauce Labs,
3. screenshots taken and [uploaded to Argos](https://www.argos-ci.com/stencila/thema/builds).

To run these tests locally, run `npm run test`.
By default Webdriver will try to run the tests using Chrome, but you can switch to Firefox by setting an `TEST_BROWSER=firefox` environment variable.

When testing locally, there are three screenshot folders to be aware of inside the `test/screenshots` directory:

- `reference`: These are the baseline screenshots. To generate them, make sure your Git branch is free of any changes, and run `npm test`.
- `local`: These are screenshots generated by Webdriver tests, and will be compared to those found in the `reference` directory.
- `diff`: If any discrepancies are found between the `reference` and `local` screenshots, the differences will be highlighted and saved to this directory.

There is a pseudo-test in `test/screenshot.test.js` which can be un-skipped to help with debugging the automated running of tests.

## Acknowledgments

We rely on many tools and services for which we are grateful ❤ to their developers and contributors for all their time and energy.

|                                                 Tool                                                 | Use                                  |
| :--------------------------------------------------------------------------------------------------: | ------------------------------------ |
|    <a href="https://saucelabs.com"><img src="./.github/PoweredBySauceLabs.svg" width="150" /></a>    | Cross-browser testing platform       |
| <a href="https://webdriver.io/"><img src="https://webdriver.io/img/webdriverio.png" width="50"/></a> | WebDriver test framework for Node.js |
|        <a href="https://www.argos-ci.com/"><img src="./.github/ArgosCI.svg" width="150"/></a>        | Visual regression system             |

## Utilities API

Several utility functions are provided in the [`util`](./src/util) module for traversing and manipulating the DOM. These may be useful for theme and extension authors when there is a need to modify the HTML structure of the document (e.g. adding additional purely presentational elements such as social sharing buttons). Caution should be taken to not overly modify the content. Only use these functions for things that are not possible with CSS alone.

<!-- prettier-ignore-start -->
<!-- UTIL-API -->
### Functions

<dl>
<dt><a href="#ready">ready(func)</a></dt>
<dd><p>Register a function to be executed when the DOM is fully loaded.</p>
</dd>
<dt><a href="#first">first(elem, selector)</a> ⇒ <code>Element</code> | <code>null</code></dt>
<dd><p>Select the first element matching a CSS selector.</p>
</dd>
<dt><a href="#select">select(elem, selector)</a> ⇒ <code>Array.&lt;Element&gt;</code></dt>
<dd><p>Select all elements matching a CSS selector.</p>
</dd>
<dt><a href="#create">create(spec, ...children)</a> ⇒ <code>Element</code></dt>
<dd><p>Create a new element.</p>
</dd>
<dt><a href="#attr">attr(target, name, value)</a> ⇒ <code>string</code> | <code>null</code> | <code>undefined</code></dt>
<dd><p>Get or set the value of an attribute on an element.</p>
</dd>
<dt><a href="#text">text(target, value)</a> ⇒ <code>string</code> | <code>null</code> | <code>undefined</code></dt>
<dd><p>Get or set the text content of an element.</p>
</dd>
<dt><a href="#append">append(target, ...elems)</a></dt>
<dd><p>Append new child elements to an element.</p>
</dd>
<dt><a href="#prepend">prepend(target, ...elems)</a></dt>
<dd><p>Prepend new child elements to an element.</p>
</dd>
<dt><a href="#before">before(target, ...elems)</a></dt>
<dd><p>Insert new elements before an element.</p>
</dd>
<dt><a href="#after">after(target, ...elems)</a></dt>
<dd><p>Insert new elements after an element.</p>
</dd>
<dt><a href="#replace">replace(target, ...elems)</a></dt>
<dd><p>Replace an element with a new element.</p>
</dd>
<dt><a href="#wrap">wrap(target, elem)</a></dt>
<dd><p>Wrap an element with a new element.</p>
</dd>
</dl>

<a name="ready"></a>

### ready(func)
Register a function to be executed when the DOM is fully loaded.

**Kind**: global function
**Detail**: Use this to wrap calls to the DOM selection and manipulation functions
to be sure that the DOM is ready before working on it.

| Param | Type | Description |
| --- | --- | --- |
| func | <code>function</code> | Function to register |

**Example**
```js
ready(() => {
  // Use other DOM manipulation functions here
})
```
<a name="first"></a>

### first(elem, selector) ⇒ <code>Element</code> \| <code>null</code>
Select the first element matching a CSS selector.

**Kind**: global function
**Returns**: <code>Element</code> \| <code>null</code> - An `Element` or `null` if no match
**Detail**: This function provides a short hand for `querySelector` but
also allowing for the use of semantic selectors.
You can use it for the whole document, or scoped to a particular element.

| Param | Type | Description |
| --- | --- | --- |
| elem | <code>Element</code> | The element to query (defaults to the `window.document`) |
| selector | <code>string</code> | The selector to match |

**Example** *(Select the first element from the document matching selector)*
```js

first(':--CodeChunk')
```
**Example** *(Select the first element within an element matching the selector)*
```js

first(elem, ':--author')
```
<a name="select"></a>

### select(elem, selector) ⇒ <code>Array.&lt;Element&gt;</code>
Select all elements matching a CSS selector.

**Kind**: global function
**Returns**: <code>Array.&lt;Element&gt;</code> - An array of elements
**Detail**: Provides a short hand for using `querySelectorAll` but
also allowing for the use of semantic selectors. You can use it for
the whole document, or scoped to a particular element.

| Param | Type | Description |
| --- | --- | --- |
| elem | <code>Element</code> | The element to query (defaults to the `window.document`) |
| selector | <code>string</code> | The selector to match |

**Example** *(Select all elements from the document matching selector)*
```js

select(':--CodeChunk')
```
**Example** *(Select all elements within an element matching the selector)*
```js

select(elem, ':--author')
```
<a name="create"></a>

### create(spec, ...children) ⇒ <code>Element</code>
Create a new element.

**Kind**: global function
**Detail**: This function allows creation of new elements using either a
(a) HTML string (b) CSS selector like string, or (c) an `Element`.
CSS selectors are are convenient way to create elements with attributes,
particularly Microdata elements. They can be prone to syntax errors however.
Alternatively, the second argument can
be an object of attribute name:value pairs.

| Param | Type | Description |
| --- | --- | --- |
| spec | <code>string</code> \| <code>Element</code> | Specification of element to create. |
| ...children | <code>object</code> \| <code>string</code> \| <code>number</code> \| <code>Element</code> | Additional child elements to add.        If the first is an object then it is used to set the element's attributes. |

**Example** *(Create a &lt;figure&gt; with id, class and itemtype attributes)*
```js

create('figure #fig1 .fig :--Figure')
// <figure id="fig1" class="fig" itemscope="" itemtype="http://schema.stenci.la/Figure">
// </figure>
```
**Example** *(As above but using an object to specify attributes)*
```js

create('figure', {
  id: 'fig1',
  class: 'fig',
  itemscope: '',
  itemtype: translate(':--Figure')
})
```
**Example** *(Create a Person with a name property)*
```js

create(':--Person', create('span :--name', 'John Doe'))
// <div itemscope="" itemtype="http://schema.org/Person">
//   <span itemprop="name">John Doe</span>
// </div>
```
<a name="attr"></a>

### attr(target, name, value) ⇒ <code>string</code> \| <code>null</code> \| <code>undefined</code>
Get or set the value of an attribute on an element.

**Kind**: global function
**Returns**: <code>string</code> \| <code>null</code> \| <code>undefined</code> - `null` if the attribute does not exist,
                                     `undefined` when setting

| Param | Type | Description |
| --- | --- | --- |
| target | <code>Element</code> | The element to get or set the attribute |
| name | <code>string</code> | The name of the attribute |
| value | <code>string</code> | The value of the attribute (when setting) |

**Example** *(Set an attribute value)*
```js

attr(elem, "attr", "value")
```
**Example** *(Get an attribute)*
```js

attr(elem, "attr") // "value"
```
<a name="text"></a>

### text(target, value) ⇒ <code>string</code> \| <code>null</code> \| <code>undefined</code>
Get or set the text content of an element.

**Kind**: global function
**Returns**: <code>string</code> \| <code>null</code> \| <code>undefined</code> - `null` if there is no text content,
                                     `undefined` when setting

| Param | Type | Description |
| --- | --- | --- |
| target | <code>Element</code> | The element to get or set the text content |
| value | <code>string</code> | The value of the text content (when setting) |

**Example** *(Set the text content)*
```js

text(elem, "text content")
```
**Example** *(Get the text content)*
```js

text(elem) // "text content"
```
<a name="append"></a>

### append(target, ...elems)
Append new child elements to an element.

**Kind**: global function

| Param | Type | Description |
| --- | --- | --- |
| target | <code>Element</code> | The element to append to |
| ...elems | <code>Element</code> | The elements to append |

<a name="prepend"></a>

### prepend(target, ...elems)
Prepend new child elements to an element.

**Kind**: global function
**Detail**: When called with multiple elements to prepend
will maintain the order of those elements (at the top
of the target element).

| Param | Type | Description |
| --- | --- | --- |
| target | <code>Element</code> | The element to prepend to |
| ...elems | <code>Element</code> | The elements to prepend |

<a name="before"></a>

### before(target, ...elems)
Insert new elements before an element.

**Kind**: global function

| Param | Type | Description |
| --- | --- | --- |
| target | <code>Element</code> | The element before which the elements are to be inserted |
| ...elems | <code>Element</code> | The elements to insert |

<a name="after"></a>

### after(target, ...elems)
Insert new elements after an element.

**Kind**: global function

| Param | Type | Description |
| --- | --- | --- |
| target | <code>Element</code> | The element after which the elements are to be inserted |
| ...elems | <code>Element</code> | The elements to insert |

<a name="replace"></a>

### replace(target, ...elems)
Replace an element with a new element.

**Kind**: global function

| Param | Type | Description |
| --- | --- | --- |
| target | <code>Element</code> | The element to replace |
| ...elems | <code>Element</code> | The elements to replace it with |

<a name="wrap"></a>

### wrap(target, elem)
Wrap an element with a new element.

**Kind**: global function
**Detail**: This function can be useful if you need
to create a container element to more easily style
a type of element.

| Param | Description |
| --- | --- |
| target | The element to wrap |
| elem | The element to wrap it in |

**Example** *(Wrap all figure captions in a &lt;div&gt;)*
```js

select(':--Figure :--caption')
  .forEach(caption => wrap(caption, create('div')))
```
<!-- UTIL-API-END -->
<!-- prettier-ignore-end -->
