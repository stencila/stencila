/**
 * Native math nodes for the Stencila Tiptap editor.
 */
import {
  type Attributes,
  InputRule,
  type InputRuleMatch,
  Node,
  type NodeViewRenderer,
  type NodeViewRendererProps,
} from '@tiptap/core'
import katex from 'katex'
import 'katex/dist/katex.css'

type MathAttrs = {
  code?: string | null
  mathLanguage?: string | null
  compilationMessages?: unknown
  mathml?: string | null
  images?: unknown
  [key: string]: unknown
}

type MathInputRuleData = {
  code: string
}

type MathNodeViewOptions = {
  displayMode: boolean
  inline: boolean
}

const DEFAULT_MATH_LANGUAGE = 'tex'

/**
 * Build shared attrs for both native math node types.
 *
 * Math blocks and math inlines carry mostly the same Stencila fields. Keeping
 * these attrs together avoids schema drift between block and inline editor
 * nodes, and lets the Rust Tiptap codec preserve display output even though the
 * browser editor primarily edits the source string.
 */
function sharedMathAttributes(): Attributes {
  return {
    id: {
      default: null,
      rendered: false,
    },
    code: {
      default: '',
      rendered: false,
    },
    mathLanguage: {
      default: DEFAULT_MATH_LANGUAGE,
      rendered: false,
    },
    compilationMessages: {
      default: null,
      rendered: false,
    },
    mathml: {
      default: null,
      rendered: false,
    },
    images: {
      default: null,
      rendered: false,
    },
  }
}

/**
 * Build the attrs used only by native math block nodes.
 *
 * Math blocks can be labelled and auto-identified in the Stencila schema,
 * whereas math inlines cannot. These attrs are preserved on the Tiptap node so
 * choosing a native editor representation for math blocks does not drop label
 * state that previously survived inside opaque Stencila payloads.
 */
function mathBlockAttributes(): Attributes {
  return {
    ...sharedMathAttributes(),
    idAutomatically: {
      default: null,
      rendered: false,
    },
    label: {
      default: null,
      rendered: false,
    },
    labelAutomatically: {
      default: null,
      rendered: false,
    },
  }
}

/**
 * Remove compiled output attrs after the math source changes.
 *
 * Compiled MathML, images, and diagnostics describe a previous source string.
 * Clearing them when the user edits source prevents the rendered atom from
 * showing stale output until the document is compiled again.
 */
export function attrsWithUpdatedMathSource(
  attrs: MathAttrs,
  code: string
): MathAttrs {
  return {
    ...attrs,
    code,
    compilationMessages: null,
    mathml: null,
    images: null,
  }
}

/**
 * Find inline `$...$` math typed at the end of a text block.
 *
 * The inline matcher is deliberately separate from the block matcher so the
 * second dollar in `$$...$$` does not create an inline math node halfway through
 * typing a block shortcut. Returning normalized data also lets the handler trim
 * delimiter whitespace without relying on capture-group positions.
 */
function findInlineMath(text: string): InputRuleMatch | null {
  const match = /\$([^$\n]+)\$$/.exec(text)

  if (!match || text[match.index - 1] === '$') {
    return null
  }

  const code = match[1].trim()
  if (!code) {
    return null
  }

  return {
    index: match.index,
    text: match[0],
    data: { code },
  }
}

/**
 * Find block `$$...$$` math occupying a whole text block.
 *
 * Block math replaces the containing paragraph with an atom node, so the rule
 * only accepts whole-paragraph shortcuts. That keeps mixed prose like
 * `total $$x$$` from unexpectedly deleting surrounding text.
 */
function findBlockMath(text: string): InputRuleMatch | null {
  const match = /^\$\$([^$\n]+)\$\$$/.exec(text)
  const code = match?.[1]?.trim()

  if (!match || !code) {
    return null
  }

  return {
    index: 0,
    text: match[0],
    data: { code },
  }
}

/**
 * Create an input rule that replaces `$...$` with an inline math atom.
 *
 * Tiptap's generic node input rule inserts inline nodes but is too broad for
 * the delimiter conflict between inline and block math. This custom rule uses
 * the normalized match data and keeps the replacement range exactly to the
 * typed inline shortcut.
 */
function inlineMathInputRule(type: NodeViewRendererProps['node']['type']) {
  return new InputRule({
    find: findInlineMath,
    handler: ({ state, range, match }) => {
      const data = match.data as MathInputRuleData | undefined
      if (!data?.code) {
        return null
      }

      state.tr.replaceWith(
        range.from,
        range.to,
        type.create({
          code: data.code,
          mathLanguage: DEFAULT_MATH_LANGUAGE,
        })
      )
    },
  })
}

/**
 * Create an input rule that replaces `$$...$$` paragraphs with math blocks.
 *
 * Math blocks are atom blocks rather than text blocks, so converting the
 * current paragraph is clearer than trying to reuse Tiptap textblock helpers.
 * The handler verifies the selection parent can be replaced before touching the
 * transaction, which lets unsupported contexts ignore the shortcut harmlessly.
 */
function blockMathInputRule(type: NodeViewRendererProps['node']['type']) {
  return new InputRule({
    find: findBlockMath,
    handler: ({ state, match }) => {
      const data = match.data as MathInputRuleData | undefined
      if (!data?.code) {
        return null
      }

      const { $from } = state.selection
      const parent = $from.parent
      const index = $from.index($from.depth - 1)

      if (!parent.isTextblock || !$from.node(-1).canReplaceWith(index, index + 1, type)) {
        return null
      }

      state.tr.replaceWith(
        $from.before(),
        $from.after(),
        type.create({
          code: data.code,
          mathLanguage: DEFAULT_MATH_LANGUAGE,
        })
      )
    },
  })
}

/**
 * Check whether a math language can be rendered locally by KaTeX.
 *
 * Stencila supports several math languages, but KaTeX only renders TeX-like
 * input. Keeping this test centralized ensures non-TeX math falls back to
 * stored output or source text instead of being misinterpreted as TeX.
 */
function isTexMathLanguage(language: unknown): boolean {
  return (
    typeof language !== 'string' ||
    language === '' ||
    ['tex', 'latex'].includes(language.toLowerCase())
  )
}

/**
 * Render math attrs to a DOM node for the editor atom.
 *
 * Rendering prefers local KaTeX for immediate feedback while editing. Stored
 * MathML remains useful for non-TeX languages and precompiled documents, and a
 * plain source fallback keeps invalid or unsupported math visible and editable.
 */
function renderMathContent(attrs: MathAttrs, displayMode: boolean): HTMLElement {
  const wrapper = document.createElement(displayMode ? 'div' : 'span')
  wrapper.className = 'stencila-tiptap-math-render'
  const code = typeof attrs.code === 'string' ? attrs.code : ''

  if (code && isTexMathLanguage(attrs.mathLanguage)) {
    try {
      wrapper.innerHTML = katex.renderToString(code, {
        displayMode,
        output: 'htmlAndMathml',
        throwOnError: false,
      })
      return wrapper
    } catch {
      // Fall through to stored output or source text.
    }
  }

  if (typeof attrs.mathml === 'string' && attrs.mathml.trim()) {
    wrapper.innerHTML = attrs.mathml
    return wrapper
  }

  const fallback = document.createElement('code')
  fallback.textContent = displayMode ? `$$${code}$$` : `$${code}$`
  wrapper.append(fallback)

  return wrapper
}

/**
 * Patch the source attrs of a math node view.
 *
 * Node views are outside normal ProseMirror content, so saving in-place source
 * edits must update the atom's attrs manually. The patch keeps unrelated
 * metadata and marks intact while clearing stale compiled output.
 */
function dispatchMathSourceEdit(
  props: NodeViewRendererProps,
  attrs: MathAttrs,
  code: string,
  marks: NodeViewRendererProps['node']['marks']
) {
  const pos = props.getPos()

  if (typeof pos !== 'number') {
    return
  }

  const updatedAttrs = attrsWithUpdatedMathSource(attrs, code)
  props.editor.view.dispatch(
    props.editor.state.tr
      .setNodeMarkup(pos, undefined, updatedAttrs, marks)
      .scrollIntoView()
  )
  props.editor.view.focus()
}

/**
 * Create a native node view for rendered, double-click-editable math atoms.
 *
 * The rendered atom keeps document flow stable and matches the selected UX,
 * while the transient input/textarea gives users a direct way to edit source
 * without replacing math with editable ProseMirror text content.
 */
function createMathNodeView({
  displayMode,
  inline,
}: MathNodeViewOptions): NodeViewRenderer {
  return (props) => {
    let currentNode = props.node
    let editing = false

    const dom = document.createElement(inline ? 'span' : 'div')
    dom.className = inline
      ? 'stencila-tiptap-math stencila-tiptap-math-inline'
      : 'stencila-tiptap-math stencila-tiptap-math-block'
    dom.contentEditable = 'false'

    const render = () => {
      dom.replaceChildren(renderMathContent(currentNode.attrs, displayMode))
    }

    const stopEditing = (save: boolean, input: HTMLInputElement | HTMLTextAreaElement) => {
      editing = false

      if (save) {
        dispatchMathSourceEdit(props, currentNode.attrs, input.value, currentNode.marks)
      } else {
        render()
        props.editor.view.focus()
      }
    }

    const startEditing = () => {
      if (editing) {
        return
      }

      editing = true
      const input = displayMode
        ? document.createElement('textarea')
        : document.createElement('input')

      input.className = 'stencila-tiptap-math-source'
      input.value = typeof currentNode.attrs.code === 'string' ? currentNode.attrs.code : ''
      input.spellcheck = false
      input.setAttribute('aria-label', displayMode ? 'Math block source' : 'Math source')

      input.addEventListener('keydown', (event: KeyboardEvent) => {
        if (event.key === 'Escape') {
          event.preventDefault()
          stopEditing(false, input)
        } else if (
          event.key === 'Enter' &&
          (!displayMode || event.metaKey || event.ctrlKey)
        ) {
          event.preventDefault()
          stopEditing(true, input)
        }
      })

      input.addEventListener('blur', () => {
        if (editing) {
          stopEditing(true, input)
        }
      })

      dom.replaceChildren(input)
      input.focus()
      input.select()
    }

    dom.addEventListener('dblclick', (event) => {
      event.preventDefault()
      event.stopPropagation()
      startEditing()
    })

    render()

    return {
      dom,
      update(node) {
        if (node.type !== currentNode.type) {
          return false
        }

        currentNode = node

        if (!editing) {
          render()
        }

        return true
      },
      selectNode() {
        dom.classList.add('ProseMirror-selectednode')
      },
      deselectNode() {
        dom.classList.remove('ProseMirror-selectednode')
      },
      stopEvent() {
        return editing
      },
      ignoreMutation() {
        return true
      },
    }
  }
}

/**
 * Native block math node.
 */
export const MathBlock = Node.create({
  name: 'mathBlock',

  group: 'block',
  atom: true,
  selectable: true,
  draggable: true,

  addAttributes() {
    return mathBlockAttributes()
  },

  parseHTML() {
    return [{ tag: 'stencila-math-block-placeholder' }]
  },

  renderHTML() {
    return [
      'stencila-math-block-placeholder',
      {
        class: 'stencila-tiptap-math stencila-tiptap-math-block',
        contenteditable: 'false',
      },
    ]
  },

  addInputRules() {
    return [blockMathInputRule(this.type)]
  },

  addNodeView() {
    return createMathNodeView({ displayMode: true, inline: false })
  },
})

/**
 * Native inline math node.
 */
export const MathInline = Node.create({
  name: 'mathInline',

  inline: true,
  group: 'inline',
  atom: true,
  selectable: true,

  addAttributes() {
    return sharedMathAttributes()
  },

  parseHTML() {
    return [{ tag: 'stencila-math-inline-placeholder' }]
  },

  renderHTML() {
    return [
      'stencila-math-inline-placeholder',
      {
        class: 'stencila-tiptap-math stencila-tiptap-math-inline',
        contenteditable: 'false',
      },
    ]
  },

  addInputRules() {
    return [inlineMathInputRule(this.type)]
  },

  addNodeView() {
    return createMathNodeView({ displayMode: false, inline: true })
  },
})
