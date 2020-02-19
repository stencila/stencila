/**
 * Convenience functions for modifying the elements in the DOM.
 */

import { semanticToAttributeSelectors } from '../selectors'

/**
 * Select all DOM elements matching a CSS selector
 *
 * @param {string} selector The selector to match
 * @param {Element} elem The element to query (defaults to the `window.document`)
 * @returns {Element[]} An array of elements
 */
export function select(selector: string): Element[]
export function select(elem: Document | Element, selector: string): Element[]
export function select(...args: (string | Document | Element)[]): Element[] {
  const [elem, selector] = (args.length === 1
    ? [document, args[0]]
    : args.slice(0, 2)) as [Element, string]
  return Array.from(
    elem.querySelectorAll(semanticToAttributeSelectors(selector))
  )
}

/**
 * Type definition for something that can be used as a wrapper for
 * HTML elements: an existing element, HTML for an element, a function
 * that creates an element.
 */
type Creator = string | Element | ((elems?: Element[]) => Element)

/**
 * Create a DOM element using a fragment of HTML or a CSS selector
 *
 * Credit to [dom-create-element-query-selector](https://github.com/hekigan/dom-create-element-query-selector)
 * for the regexes.
 *
 * @param {string} [spec='div'] Specification of element.
 * @param {...Element[]} children Additional child elements to add
 * @returns {Element}
 */
export function create(
  spec = 'div',
  ...children: (string | number | Element)[]
): Element {
  if (/^\s*</.test(spec)) {
    const wrapper = document.createElement('div')
    wrapper.innerHTML = spec
    return wrapper.firstElementChild as Element
  }

  const tag = /^[a-z0-9]+/i.exec(spec)?.[0] ?? 'div'
  const id = spec.match(/#([a-z]+[a-z0-9-]*)/gi) ?? []
  const classes = spec.match(/\.([a-z]+[a-z0-9-]*)/gi) ?? []
  const attrs = spec.match(/\[([a-z][a-z-]+)(=['|"]?([^\]]*)['|"]?)?\]/gi) ?? []

  const elem = document.createElement(tag)

  if (id.length === 1) elem.id = id[0].slice(1)
  else if (id.length > 1)
    console.warn(`Got more than one id; ignoring all but first`)

  if (classes.length > 0)
    elem.setAttribute('class', classes.map(item => item.slice(1)).join(' '))

  attrs.forEach(item => {
    let [label, value] = item.slice(1, -1).split('=')
    if (value !== undefined) value = value.replace(/^['"](.*)['"]$/, '$1')
    elem.setAttribute(label, value ?? '')
  })

  children.forEach(item =>
    elem.appendChild(
      item instanceof Element ? item : document.createTextNode(`${item}`)
    )
  )

  return elem
}

export function before(target: Element, insert: Element): void {
  const parent = target.parentNode
  if (parent !== null) {
    parent.insertBefore(insert, target)
  }
}

export function after(target: Element, ...insert: Element[]): void {
  const parent = target.parentNode
  if (parent !== null) {
    insert
      .reverse()
      .forEach(elem => parent.insertBefore(elem, target.nextSibling))
  }
}

export function replace(
  target: string | Element,
  replacement: string | Element
): void {
  if (typeof target === 'string') target = select(target)[0]
  if (typeof replacement === 'string') replacement = create(replacement)

  const parent = target.parentNode
  if (parent !== null) {
    parent.replaceChild(replacement, target)
  }
}

/**
 * Wrap a DOM element in a wrapper.
 *
 * @param {string} within CSS selector for the elements within which wrapping happens
 * @param {string} target CSS selector for the elements to be wrapped
 * @param {Creator} wrapper The wrapper to create
 */
export function wrap(target: string, wrapper: Creator): void
export function wrap(within: string, target: string, wrapper: Creator): void
export function wrap(...args: (string | Creator | undefined)[]): void {
  const wrapper = args.pop() ?? 'div'
  const target = args.pop() as string
  if (target === undefined)
    throw new Error('Required argument `target` is missing')
  const within = (args.pop() as string) ?? 'body'

  select(document, within).forEach(parent => {
    const wrapees = select(parent, target)

    let wrapperElem: Element
    if (wrapper instanceof Element) wrapperElem = wrapper.cloneNode() as Element
    else if (typeof wrapper === 'string') wrapperElem = create(wrapper)
    else if (typeof wrapper === 'function')
      wrapperElem = wrapper(Array.from(wrapees))
    else throw new Error(`Unhandled wrapper type: ${typeof wrapper}`)

    if (wrapees.length > 0) {
      const first = wrapees[0]
      const parent = first.parentNode
      if (parent !== null) {
        parent.insertBefore(wrapperElem, wrapees[0])
      }
    }
    wrapees.forEach(wrapee => wrapperElem.appendChild(wrapee))
  })
}
