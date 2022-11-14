import {
  baseKeymap,
  lift,
  selectParentNode,
  toggleMark,
} from 'prosemirror-commands'
import { dropCursor } from 'prosemirror-dropcursor'
import { gapCursor } from 'prosemirror-gapcursor'
import { history, redo, undo } from 'prosemirror-history'
import { undoInputRule } from 'prosemirror-inputrules'
import { keymap } from 'prosemirror-keymap'
import { DOMParser } from 'prosemirror-model'
import { schema as basicSchema } from 'prosemirror-schema-basic'
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

  // Toggling marks
  'Mod-i': toggleMark(basicSchema.marks.em),
  'Mod-b': toggleMark(basicSchema.marks.strong),

  // TODO: more
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
