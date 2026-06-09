/**
 * Common inline marks for the Stencila Tiptap editor.
 *
 * These marks intentionally use the same JSON names as the Rust Tiptap codec so
 * documents can round-trip between the browser editor and server-side codec.
 */
import {
  InputRule,
  type InputRuleFinder,
  type InputRuleMatch,
  Mark,
} from '@tiptap/core'
import Link, { isAllowedUri } from '@tiptap/extension-link'
import type { MarkType } from '@tiptap/pm/model'

type MarkInputRuleData = {
  attrs?: Record<string, unknown>
  content: string
}

function omitNullishAttributes(attributes: Record<string, unknown>) {
  return Object.fromEntries(
    Object.entries(attributes).filter(([, value]) => value !== null && value !== undefined)
  )
}

/**
 * Find delimited Markdown text and return the mark content plus attributes.
 *
 * Tiptap input rules need the matched source range, while Stencila marks need
 * only the inner content to remain after delimiters are removed. This helper
 * keeps that range/content bookkeeping in one place and lets individual mark
 * rules reject otherwise valid syntax by returning `null` from `getAttributes`.
 */
function findMarkedText(
  text: string,
  pattern: RegExp,
  contentIndex = 1,
  getAttributes?: (match: RegExpMatchArray) => Record<string, unknown> | null
): InputRuleMatch | null {
  const match = pattern.exec(text)
  const content = match?.[contentIndex]

  if (!match || !content) {
    return null
  }

  const attrs = getAttributes?.(match)

  if (attrs === null) {
    return null
  }

  return {
    index: match.index,
    text: match[0],
    data: {
      attrs,
      content,
    },
  }
}

/**
 * Find Stencila Markdown subscript syntax.
 *
 * Subscript uses single tildes, while strikeout uses double tildes. This custom
 * matcher prevents the subscript rule from firing on the partial `~~text~`
 * state that appears while a user is still typing `~~text~~`.
 */
function findSubscriptText(text: string): InputRuleMatch | null {
  const match = /~([^~\s]+)~$/.exec(text)

  if (!match || text[match.index - 1] === '~') {
    return null
  }

  return findMarkedText(text, /~([^~\s]+)~$/)
}

/**
 * Create an input rule that replaces Markdown delimiters with a mark.
 *
 * The built-in Tiptap mark input rule assumes a regex capture is enough to
 * locate the final content. These shortcuts also need custom matchers for links
 * and conflicting delimiters, so this rule reads the normalized match data and
 * applies the existing mark type to only the inner text.
 */
function markedTextInputRule({
  find,
  type,
}: {
  find: InputRuleFinder
  type: MarkType
}) {
  return new InputRule({
    find,
    handler: ({ state, range, match }) => {
      const data = match.data as MarkInputRuleData | undefined

      if (!data?.content) {
        return null
      }

      const contentStart = match[0].indexOf(data.content)

      if (contentStart < 0) {
        return null
      }

      const contentEnd = contentStart + data.content.length
      const textStart = range.from + contentStart
      const textEnd = range.from + contentEnd
      const { tr } = state

      if (textEnd < range.to) {
        tr.delete(textEnd, range.to)
      }

      if (textStart > range.from) {
        tr.delete(range.from, textStart)
      }

      tr.addMark(
        range.from,
        range.from + data.content.length,
        type.create(data.attrs ?? {})
      )
      tr.removeStoredMark(type)
    },
  })
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

  addInputRules() {
    return [
      markedTextInputRule({
        find: (text) => findMarkedText(text, /`([^`\n]+)`$/),
        type: this.type,
      }),
    ]
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

  addInputRules() {
    return [
      markedTextInputRule({
        find: (text) =>
          findMarkedText(
            text,
            /\[([^\]\n]+)\]\((\S+?)(?:\s+"([^"\n]+)")?\)$/,
            1,
            (match) => {
              const href = match[2]
              const title = match[3] ?? null

              if (
                !this.options.isAllowedUri(href, {
                  defaultValidate: (url) =>
                    Boolean(isAllowedUri(url, this.options.protocols)),
                  protocols: this.options.protocols,
                  defaultProtocol: this.options.defaultProtocol,
                })
              ) {
                return null
              }

              return { href, title }
            }
          ),
        type: this.type,
      }),
    ]
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

  addInputRules() {
    return [
      markedTextInputRule({
        find: (text) => findMarkedText(text, /~~([^~\n]+)~~$/),
        type: this.type,
      }),
    ]
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

  addInputRules() {
    return [
      markedTextInputRule({
        find: findSubscriptText,
        type: this.type,
      }),
    ]
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

  addInputRules() {
    return [
      markedTextInputRule({
        find: (text) => findMarkedText(text, /\^([^^\s]+)\^$/),
        type: this.type,
      }),
    ]
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
