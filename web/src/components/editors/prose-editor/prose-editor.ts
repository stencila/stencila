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
import { EditorView } from 'prosemirror-view'

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
