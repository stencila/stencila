import { translate } from '../selectors'
export { translate }

/**
 * Convenience functions for manipulating the DOM.
 *
 * Most of the names of these functions mirror their analogs
 * in https://api.jquery.com.
 * Inspiration also taken from https://plainjs.com/javascript.
 */

/**
 * Register a function to be executed when the DOM is fully loaded.
 *
 * @detail
 *
 * Use this to wrap calls to the DOM selection and manipulation functions
 * to be sure that the DOM is ready before working on it.
 *
 * @example
 *
 * ready(() => {
 *   // Use other DOM manipulation functions here
 * })
 *
 * @param {function} func Function to register
 */
export function ready(func: () => unknown): void {
  // Logic from https://stackoverflow.com/questions/9899372/pure-javascript-equivalent-of-jquerys-ready-how-to-call-a-function-when-t
  if (readyFired) {
    // Run the function asynchronously, but right away
    setTimeout(func, 1)
    return
  } else {
    // Add to the list of functions to run when the DOM is ready
    readyList.push(func)
  }

  if (document.readyState === 'complete') {
    // Document is ready so run whenReady asynchronously, but right away
    setTimeout(whenReady, 1)
  } else if (!readyListening) {
    // Add a listener to run whenReady when the DOM is ready
    document.addEventListener('DOMContentLoaded', whenReady)
    readyListening = true
  }
}

// Module global variables used in the `ready` function
let readyList: (() => unknown)[] = []
let readyListening = false
let readyFired = false

/**
 * When the DOM is ready, call all of the functions registered
 * using `ready()`.
 *
 * Clears `readyList` to allow for garbage collection of closures.
 *
 * @private
 */
export function whenReady(): void {
  if (readyFired) return
  readyFired = true
  readyList.forEach(func => func())
  readyList = []
  document.removeEventListener('DOMContentLoaded', whenReady)
}

export function first(selector: string): Element | null
export function first(
  elem: Document | Element,
  selector: string
): Element | null
/**
 * Select the first element matching a CSS selector.
 *
 * @detail This function provides a short hand for `querySelector` but
 * also allowing for the use of semantic selectors.
 * You can use it for the whole document, or scoped to a particular element.
 *
 * @example <caption>Select the first element from the document matching selector</caption>
 *
 * first(':--CodeChunk')
 *
 * @example <caption>Select the first element within an element matching the selector</caption>
 *
 * first(elem, ':--author')
 *
 * @param {Element} [elem] The element to query (defaults to the `window.document`)
 * @param {string} selector The selector to match
 * @returns {Element | null} An `Element` or `null` if no match
 */
export function first(
  ...args: (string | Document | Element)[]
): Element | null {
  const [elem, selector] = (args.length === 1
    ? [document, args[0]]
    : args.slice(0, 2)) as [Element, string]
  return elem.querySelector(translate(selector))
}

export function select(selector: string): Element[]
export function select(elem: Document | Element, selector: string): Element[]
/**
 * Select all elements matching a CSS selector.
 *
 * @detail Provides a short hand for using `querySelectorAll` but
 * also allowing for the use of semantic selectors. You can use it for
 * the whole document, or scoped to a particular element.
 *
 * @example <caption>Select all elements from the document matching selector</caption>
 *
 * select(':--CodeChunk')
 *
 * @example <caption>Select all elements within an element matching the selector</caption>
 *
 * select(elem, ':--author')
 *
 * @param {Element} [elem] The element to query (defaults to the `window.document`)
 * @param {string} selector The selector to match
 * @returns {Element[]} An array of elements
 */
export function select(...args: (string | Document | Element)[]): Element[] {
  const [elem, selector] = (args.length === 1
    ? [document, args[0]]
    : args.slice(0, 2)) as [Element, string]
  return Array.from(elem.querySelectorAll(translate(selector)))
}

/**
 * Create a new element.
 *
 * @detail This function allows creation of new elements using either a
 * (a) HTML string (b) CSS selector like string, or (c) an `Element`.
 * CSS selectors are are convenient way to create elements with attributes,
 * particularly Microdata elements. They can be prone to syntax errors however.
 * Alternatively, the second argument can
 * be an object of attribute name:value pairs.
 *
 * @example <caption>Create a <figure> with id, class and itemtype attributes</caption>
 *
 * create('figure #fig1 .fig :--Figure')
 * // <figure id="fig1" class="fig" itemscope="" itemtype="http://schema.stenci.la/Figure">
 * // </figure>
 *
 * @example <caption>As above but using an object to specify attributes</caption>
 *
 * create('figure', {
 *   id: 'fig1',
 *   class: 'fig',
 *   itemscope: '',
 *   itemtype: translate(':--Figure')
 * })
 *
 * @example <caption>Create a Person with a name property</caption>
 *
 * create(':--Person', create('span :--name', 'John Doe'))
 * // <div itemscope="" itemtype="http://schema.org/Person">
 * //   <span itemprop="name">John Doe</span>
 * // </div>
 *
 * @param {string | Element} [spec] Specification of element to create.
 * @param {(object | undefined | null | boolean | number | string | Element)} [attributes] Attributes for the element.
 * @param {...(undefined | null | boolean | number | string | Element)} children Child nodes to to add as text content or elements.
 * @returns {Element}
 */
export function create(
  spec: string | Element = 'div',
  attributes?:
    | Record<string, undefined | null | boolean | number | string>
    | (object | undefined | null | boolean | number | string | Element),
  ...children: (undefined | null | boolean | number | string | Element)[]
): Element {
  let elem: Element
  if (spec instanceof Element) {
    // Create as clone of existing element
    elem = spec.cloneNode() as Element
  } else if (/^\s*</.test(spec)) {
    // Create from HTML
    const wrapper = document.createElement('div')
    wrapper.innerHTML = spec
    elem = wrapper.firstElementChild as Element
  } else {
    // Create from CSS selector
    // Translate semantic selectors to attribute selectors
    // 1. Type selectors (need itemscope attr too)
    spec = spec.replace(
      /:--[A-Z][a-z]+/g,
      typeSelector => `[itemscope] ${translate(typeSelector)}`
    )
    // 2. Prop selectors
    spec = spec.replace(/:--[a-zA-Z]+/g, translate)

    // Credit to https://github.com/hekigan/dom-create-element-query-selector
    // for the regexes (with some modifications).
    const tagName = /^[a-z0-9]+/i.exec(spec)?.[0] ?? 'div'
    const id = spec.match(/(?:^|\s)#([a-z]+[a-z0-9-]*)/gi) ?? []
    const classes = spec.match(/(?:^|\s)\.([a-z]+[a-z0-9-]*)/gi) ?? []
    const attribs =
      spec.match(/(?:^|\s)\[([a-z][a-z0-9-]+)(~?=['|"]?([^\]]*)['|"]?)?\]/gi) ??
      []

    elem = document.createElement(tagName)

    if (id.length >= 1) elem.id = id[0].split('#')[1]
    if (id.length > 1)
      console.warn(`Got more than one id; ignoring all but first`)

    if (classes.length > 0)
      elem.setAttribute(
        'class',
        classes.map(item => item.split('.')[1]).join(' ')
      )

    attribs.forEach(item => {
      let [label, value] = item
        .split('[')[1]
        .slice(0, -1)
        .split(/~?=/)
      if (value !== undefined) value = value.replace(/^['"](.*)['"]$/, '$1')
      elem.setAttribute(label, value ?? '')
    })
  }

  // If the attrs arg is a Record then use it, otherwise add it to children
  if (
    attributes !== null &&
    typeof attributes === 'object' &&
    !(attributes instanceof Element)
  ) {
    Object.entries(attributes).forEach(([key, value]) => {
      if (value !== undefined) elem.setAttribute(key, `${value}`)
    })
  } else if (attributes !== undefined) {
    children = [attributes as typeof children[0], ...children]
  }

  // Append children as elements or text
  children.forEach(item => append(elem, item))

  return elem
}

export function tag(target: Element): string
export function tag(target: Element, value: string): Element
/**
 * Get or set the tag name of an element.
 *
 * @detail Caution must be used when setting the tag. This function
 * does not actually change the tag of the element (that is not possible)
 * but instead returns a new `Element` that is a clone of the original apart
 * from having the new tag name. Use the `replace` function where necessary
 * i association with this function.
 *
 * @example <caption>Get the tag name as a lowercase string</caption>
 *
 * tag(elem) // "h3"
 *
 * @example <caption>Setting the tag actually returns a new element</caption>
 *
 * tag(tag(elem, 'h2')) // "h2"
 *
 * @example <caption>Change the tag name of an element</caption>
 *
 * replace(elem, tag(elem, 'h2'))
 *
 * @param {Element} target The element to get or set the tag
 * @param {string} [value] The value of the tag (when setting)
 * @returns {string | Element} The lowercase tag name when getting,
 *                             a new element when setting.
 */
export function tag(target: Element, value?: string): string | Element {
  if (value === undefined) return target.tagName.toLowerCase()

  const replacement = create(value, attrs(target))
  replacement.innerHTML = target.innerHTML
  return replacement
}

export function attrs(target: Element): Record<string, string>
export function attrs(target: Element, value: object): undefined
/**
 * Get or set the attributes of an element
 *
 * @param {Element} target The element to get or set the attributes
 * @param {object} [attributes] The name/value pairs of the attributes
 * @returns {object | undefined} The attributes of the element when getting, `undefined` when setting
 */
export function attrs(
  target: Element,
  attributes?: object
): Record<string, string> | undefined {
  if (attributes === undefined)
    return Object.assign(
      {},
      ...Array.from(target.attributes, ({ name, value }) => ({ [name]: value }))
    )
  Object.entries(attributes).forEach(([name, value]) => {
    if (value !== undefined && value !== null) target.setAttribute(name, value)
  })
}

export function attr(target: Element, name: string): string
export function attr(target: Element, name: string, value: string): null
/**
 * Get or set the value of an attribute on an element.
 *
 * @example <caption>Set an attribute value</caption>
 *
 * attr(elem, "attr", "value")
 *
 * @example <caption>Get an attribute</caption>
 *
 * attr(elem, "attr") // "value"
 *
 * @param {Element} target The element to get or set the attribute
 * @param {string} name The name of the attribute
 * @param {string} [value] The value of the attribute (when setting)
 * @returns {string | null} a `string` if the attribute exists, `null` if does not exist, or when setting
 */
export function attr(
  target: Element,
  name: string,
  value?: string
): string | null {
  if (value === undefined && value !== null) return target.getAttribute(name)
  target.setAttribute(name, value)
  return null
}

export function text(target: Element): string | null
export function text(target: Element, value: string): undefined
/**
 * Get or set the text content of an element.
 *
 * @example <caption>Set the text content</caption>
 *
 * text(elem, "text content")
 *
 * @example <caption>Get the text content</caption>
 *
 * text(elem) // "text content"
 *
 * @param {Element} target The element to get or set the text content
 * @param {string} [value] The value of the text content (when setting)
 * @returns {string | null | undefined} `null` if there is no text content,
 *                                      `undefined` when setting
 */
export function text(
  target: Element,
  value?: string
): string | null | undefined {
  if (value === undefined) return target.textContent
  target.textContent = value
}

/**
 * Append new child elements to an element.
 *
 * @param {Element} target The element to append to
 * @param {...Element} elems The elements to append
 */
export function append(
  target: Element,
  ...elems: (undefined | null | boolean | number | string | Element)[]
): void {
  elems.forEach(elem =>
    elem !== undefined && elem !== null
      ? target.appendChild(
          elem instanceof Element ? elem : document.createTextNode(`${elem}`)
        )
      : undefined
  )
}

/**
 * Prepend new child elements to an element.
 *
 * @detail When called with multiple elements to prepend
 * will maintain the order of those elements (at the top
 * of the target element).
 *
 * @param {Element} target The element to prepend to
 * @param {...Element} elems The elements to prepend
 */
export function prepend(target: Element, ...elems: Element[]): void {
  elems
    .reverse()
    .forEach(elem => target.insertBefore(elem, target.childNodes[0] ?? null))
}

/**
 * Insert new elements before an element.
 *
 * @param {Element} target The element before which the elements are to be inserted
 * @param {...Element} elems The elements to insert
 */
export function before(target: Element, ...elems: Element[]): void {
  const parent = target.parentNode
  if (parent !== null) {
    elems.forEach(elem => parent.insertBefore(elem, target))
  }
}

/**
 * Insert new elements after an element.
 *
 * @param {Element} target The element after which the elements are to be inserted
 * @param {...Element} elems The elements to insert
 */
export function after(target: Element, ...elems: Element[]): void {
  const parent = target.parentNode
  if (parent !== null) {
    elems
      .reverse()
      .forEach(elem => parent.insertBefore(elem, target.nextSibling))
  }
}

/**
 * Replace an element with a new element.
 *
 * @param {Element} target The element to replace
 * @param {...Element} elems The elements to replace it with
 */
export function replace(target: Element, ...elems: Element[]): void {
  const parent = target.parentNode
  if (parent !== null) {
    const firstReplacement = elems[0]
    parent.replaceChild(firstReplacement, target)
    after(firstReplacement, ...elems.slice(1))
  }
}

/**
 * Wrap an element with a new element.
 *
 * @detail This function can be useful if you need
 * to create a container element to more easily style
 * a type of element.
 *
 * @example <caption>Wrap all figure captions in a <div></caption>
 *
 * select(':--Figure :--caption')
 *   .forEach(caption => wrap(caption, create('div')))
 *
 * @param target The element to wrap
 * @param elem The element to wrap it in
 */
export function wrap(target: Element, elem: Element): void {
  append(elem, create(target))
  replace(target, elem)
}
