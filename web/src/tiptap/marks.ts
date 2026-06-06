/**
 * Common inline marks for the Stencila Tiptap editor.
 *
 * These marks intentionally use the same JSON names as the Rust Tiptap codec so
 * documents can round-trip between the browser editor and server-side codec.
 */
import { Mark } from '@tiptap/core'
import Link from '@tiptap/extension-link'

function omitNullishAttributes(attributes: Record<string, unknown>) {
  return Object.fromEntries(
    Object.entries(attributes).filter(([, value]) => value !== null && value !== undefined)
  )
}

/**
 * Inline code mark.
 */
export const CodeMark = Mark.create({
  name: 'code',

  code: true,

  addAttributes() {
    return {
      programmingLanguage: {
        default: null,
        rendered: false,
      },
    }
  },

  parseHTML() {
    return [{ tag: 'code' }]
  },

  renderHTML({ HTMLAttributes }) {
    return ['code', HTMLAttributes, 0]
  },
})

/**
 * Hyperlink mark.
 */
export const LinkMark = Link.extend({
  addAttributes() {
    return {
      href: {
        default: null,
      },
      title: {
        default: null,
      },
      rel: {
        default: null,
      },
      labelOnly: {
        default: null,
        rendered: false,
      },
    }
  },

  renderHTML({ mark, HTMLAttributes }) {
    const rendered = this.parent?.({ mark, HTMLAttributes })

    if (Array.isArray(rendered) && rendered[0] === 'a') {
      return ['a', omitNullishAttributes(rendered[1] as Record<string, unknown>), 0]
    }

    return ['a', omitNullishAttributes(HTMLAttributes), 0]
  },
}).configure({
  autolink: false,
  linkOnPaste: false,
  openOnClick: true,
  HTMLAttributes: {
    target: null,
    rel: null,
    class: null,
  },
})

/**
 * Strikeout mark.
 */
export const StrikeoutMark = Mark.create({
  name: 'strike',

  parseHTML() {
    return [{ tag: 's' }, { tag: 'del' }, { tag: 'strike' }]
  },

  renderHTML({ HTMLAttributes }) {
    return ['s', HTMLAttributes, 0]
  },
})

/**
 * Subscript mark.
 */
export const SubscriptMark = Mark.create({
  name: 'subscript',

  excludes: 'superscript',

  parseHTML() {
    return [{ tag: 'sub' }]
  },

  renderHTML({ HTMLAttributes }) {
    return ['sub', HTMLAttributes, 0]
  },
})

/**
 * Superscript mark.
 */
export const SuperscriptMark = Mark.create({
  name: 'superscript',

  excludes: 'subscript',

  parseHTML() {
    return [{ tag: 'sup' }]
  },

  renderHTML({ HTMLAttributes }) {
    return ['sup', HTMLAttributes, 0]
  },
})

/**
 * Underline mark.
 */
export const UnderlineMark = Mark.create({
  name: 'underline',

  parseHTML() {
    return [{ tag: 'u' }]
  },

  renderHTML({ HTMLAttributes }) {
    return ['u', HTMLAttributes, 0]
  },
})
