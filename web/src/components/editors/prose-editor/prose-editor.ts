import {
  baseKeymap,
  joinDown,
  joinUp,
  lift,
  selectParentNode,
  setBlockType,
  toggleMark,
  wrapIn,
} from 'prosemirror-commands'
import { dropCursor } from 'prosemirror-dropcursor'
import { gapCursor } from 'prosemirror-gapcursor'
import { history, redo, undo } from 'prosemirror-history'
import { undoInputRule } from 'prosemirror-inputrules'
import { keymap } from 'prosemirror-keymap'
import { DOMParser } from 'prosemirror-model'
import {
  liftListItem,
  sinkListItem,
  splitListItem,
  wrapInList,
} from 'prosemirror-schema-list'
import { EditorState } from 'prosemirror-state'
import { columnResizing, goToNextCell, tableEditing } from 'prosemirror-tables'
import { EditorView } from 'prosemirror-view'
import { transactionToOps } from '../../../patches/prosemirror'

import { stencilaInputRules } from './input-rules'
import { articleSchema, nodeViews } from './nodes'
import { ensureIds } from './plugins/ensureIds'
import { placeholder } from './plugins/placeholder'

const extendedKeymap = {
  // History
  'Mod-z': undo,
  Backspace: undoInputRule,
  'Shift-Mod-z': redo,

  // Node navigation
  'Mod-BracketLeft': lift,
  Escape: selectParentNode,
  'Alt-ArrowUp': joinUp,
  'Alt-ArrowDown': joinDown,

  // Toggling marks
  // These are consistent with Google Docs (and others?)
  'Mod-i': toggleMark(articleSchema.marks.Emphasis),
  'Mod-b': toggleMark(articleSchema.marks.Strong),
  'Mod-u': toggleMark(articleSchema.marks.Underline),
  'Alt-Shift-5': toggleMark(articleSchema.marks.Strikeout),
  'Mod-.': toggleMark(articleSchema.marks.Superscript),
  'Mod-,': toggleMark(articleSchema.marks.Subscript),

  // Changing the type of blocks
  'Shift-Mod-0': setBlockType(articleSchema.nodes.Paragraph),
  'Shift-Mod-1': setBlockType(articleSchema.nodes.Heading, { depth: 1 }),
  'Shift-Mod-2': setBlockType(articleSchema.nodes.Heading, { depth: 2 }),
  'Shift-Mod-3': setBlockType(articleSchema.nodes.Heading, { depth: 3 }),
  'Shift-Mod-4': setBlockType(articleSchema.nodes.Heading, { depth: 4 }),
  'Shift-Mod-5': setBlockType(articleSchema.nodes.Heading, { depth: 5 }),
  'Shift-Mod-6': setBlockType(articleSchema.nodes.Heading, { depth: 6 }),

  // Wrapping blocks in another type
  'Mod->': wrapIn(articleSchema.nodes.QuoteBlock),

  // List creation / manipulation
  'Shift-Mod-8': wrapInList(articleSchema.nodes.List, { order: 'Unordered' }),
  'Shift-Mod-9': wrapInList(articleSchema.nodes.List, { order: 'Ascending' }),
  Enter: splitListItem(articleSchema.nodes.ListItem),
  'Mod-[': liftListItem(articleSchema.nodes.ListItem),
  'Mod-]': sinkListItem(articleSchema.nodes.ListItem),
}

/**
 * A ProseMirror prose editor
 *
 * Previously this was a custom element but that seemed to cause complications /issues
 * and wasn't really necessary. So now its a plain old class.
 */
export default class StencilaProseEditor {
  /**
   * The ProseMirror editor view
   */
  private editorView?: EditorView

  /**
   * Handle a change to the `content` slot (including initial render) to get the
   * DOM of the content and initialize the editor with it
   */
  constructor(
    type: string = 'article',
    contentElem: Element,
    viewElem: Element
  ) {
    // Resolve schema, keymap and plugins to use
    let schema
    let plugins
    if (type === 'article') {
      schema = articleSchema
      plugins = [
        // Locally defined input rules
        stencilaInputRules, // Should go before keymap
        keymap(extendedKeymap),
        keymap(baseKeymap),
        // TODO; See if only enabling drop cursor on inner works
        dropCursor(),
        gapCursor(),
        history(),
        // Table related plugins
        columnResizing({ cellMinWidth: 10 }),
        tableEditing(),
        keymap({
          Tab: goToNextCell(1),
          'Shift-Tab': goToNextCell(-1),
        }),
        // Locally defined plugins
        placeholder(),
        ensureIds(),
      ]
    } else {
      throw new Error(`Unknown schema: ${type}`)
    }

    // Parse the content into a doc
    const doc = DOMParser.fromSchema(schema).parse(contentElem)

    // Add editor CSS
    const stylesheet = new CSSStyleSheet()
    stylesheet.replaceSync(css)
    document.adoptedStyleSheets = [...document.adoptedStyleSheets, stylesheet]

    // Create the editor view
    this.editorView = new EditorView(viewElem, {
      state: EditorState.create({
        doc,
        plugins,
      }),
      dispatchTransaction(transaction) {
        // Cast this just to get correct typing
        const view = this as EditorView

        // Apply the transaction to the state to get a new state
        const newState = view.state.apply(transaction)

        // Generate a patch and send to the editor
        const ops = transactionToOps(transaction, view.state, newState)
        if (ops.length > 0) {
          window.dispatchEvent(
            new CustomEvent('stencila-document-patch', {
              detail: { patch: { ops } },
            })
          )
        }

        // Update this view with the new state
        this.updateState(newState)
      },
      nodeViews,
    })
  }

  destroy() {
    this.editorView?.destroy()
  }
}

const css = `
/* From https://github.com/ProseMirror/prosemirror-view/blob/master/style/prosemirror.css */

.ProseMirror {
  position: relative;
}

.ProseMirror {
  word-wrap: break-word;
  white-space: pre-wrap;
  white-space: break-spaces;

  -webkit-font-variant-ligatures: none;
  font-variant-ligatures: none;
  font-feature-settings: 'liga' 0; /* the above doesn't seem to work in Edge */
}

.ProseMirror pre {
  white-space: pre-wrap;
}

.ProseMirror li {
  position: relative;
}

.ProseMirror-hideselection *::selection {
  background: transparent;
}
.ProseMirror-hideselection *::-moz-selection {
  background: transparent;
}
.ProseMirror-hideselection {
  caret-color: transparent;
}

.ProseMirror-selectednode {
  outline: 2px solid #8cf;
}

/* Make sure li selections wrap around markers */

li.ProseMirror-selectednode {
  outline: none;
}

li.ProseMirror-selectednode:after {
  content: '';
  position: absolute;
  left: -32px;
  right: -2px;
  top: -2px;
  bottom: -2px;
  border: 2px solid #8cf;
  pointer-events: none;
}

/* Protect against generic img rules */

img.ProseMirror-separator {
  display: inline !important;
  border: none !important;
  margin: 0 !important;
}

/* Tables */

.ProseMirror table {
  border: 1px solid black;
  border-collapse: collapse;
  margin: 16px auto;
  max-width: 100%;
  table-layout: fixed;
}

.ProseMirror table td,
.ProseMirror table th {
  border: 2px solid #ced4da;
  padding: 3px 5px;
  vertical-align: top;
  box-sizing: border-box;
  position: relative;
}

.ProseMirror table > * {
  margin-bottom: 0;
}

.ProseMirror table .selectedCell:after {
  z-index: 2;
  position: absolute;
  content: "";
  left: 0;
  right: 0;
  top: 0;
  bottom: 0;
  background: rgba(200, 200, 255, 0.4);
  pointer-events: none;
}

.ProseMirror table .column-resize-handle {
  position: absolute;
  right: -2px;
  top: 0;
  bottom: -2px;
  width: 4px;
  background-color: #adf;
  pointer-events: none;
  cursor: col-resize;
}


/* Placeholder plugin */

.ProseMirror .placeholder {
  color: rgba(209, 213, 219);
  pointer-events: none;
}

.ProseMirror:focus .placeholder {
  display: none;
}


/* Customizations */

.ProseMirror {
  outline: none;
}
`
