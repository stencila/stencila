import { css, html } from 'lit'
import { customElement, property } from 'lit/decorators'
import StencilaElement from '../../utils/element'

import { EditorState } from 'prosemirror-state'
import { EditorView } from 'prosemirror-view'
import { Schema, DOMParser } from 'prosemirror-model'
import { schema as basicSchema } from 'prosemirror-schema-basic'
import { addListNodes } from 'prosemirror-schema-list'
import {
  joinDown,
  joinUp,
  lift,
  selectParentNode,
  setBlockType,
  toggleMark,
  wrapIn,
} from 'prosemirror-commands'
import { history, redo, undo } from 'prosemirror-history'
import { undoInputRule } from 'prosemirror-inputrules'
import {
  liftListItem,
  sinkListItem,
  splitListItem,
  wrapInList,
} from 'prosemirror-schema-list'
import { keymap } from 'prosemirror-keymap'
import { baseKeymap } from 'prosemirror-commands'
import { dropCursor } from 'prosemirror-dropcursor'
import { gapCursor } from 'prosemirror-gapcursor'

import { DecorationSet, Decoration } from 'prosemirror-view'
import { Plugin } from 'prosemirror-state'

// TODO Use custom stencila schemas

const inlineSchema = new Schema({
  nodes: {
    doc: { content: 'text*' },
    text: { inline: true },
  },
  marks: basicSchema.spec.marks,
})

const blocksSchema = new Schema({
  nodes: addListNodes(basicSchema.spec.nodes, 'paragraph block*', 'block'),
  marks: basicSchema.spec.marks,
})

const inlineKeymap = {
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
}

const blocksKeymap = {
  // History
  'Mod-z': undo,
  Backspace: undoInputRule,
  'Shift-Mod-z': redo,

  // TODO: more
}

@customElement('stencila-prose-editor')
export default class StencilaProseEditor extends StencilaElement {
  static styles = [
    // From https://github.com/ProseMirror/prosemirror-view/blob/master/style/prosemirror.css
    css`
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
    `,
    // For placeholder plugin
    css`
      .ProseMirror .placeholder {
        text-align: center;
        color: rgba(209, 213, 219);
        pointer-events: none;
        height: 0;
      }

      .ProseMirror:focus .placeholder {
        display: none;
      }
    `,
    // Customizations
    css`
      .ProseMirror {
        outline: none;
      }
    `,
  ]

  /**
   * Whether this editor should only allow inline content
   */
  @property({ attribute: 'inline-only', type: Boolean })
  inlineOnly: boolean = false

  /**
   * A CSS class name to apply to to the editor
   */
  @property({ attribute: 'css-class' })
  cssClass: string = ''

  /**
   * CSS rules for `cssClass`
   */
  @property({ attribute: 'css-rules' })
  cssRules: string = ''

  /**
   * The element containing the content the editor will allow editing of
   */
  private contentElem: HTMLDivElement | HTMLSpanElement

  /**
   * The ProseMirror editor view
   */
  private editorView?: EditorView

  /**
   * Handle a change to the `content` slot (including initial render) to get the
   * DOM of the content and initialize the editor with it
   */
  protected onContentSlotChange(event: Event) {
    this.contentElem = (event.target as HTMLSlotElement).assignedElements({
      flatten: true,
    })[0] as HTMLDivElement | HTMLSpanElement

    const viewElem = this.renderRoot.querySelector('#prosemirror')

    const schema = this.inlineOnly ? inlineSchema : blocksSchema
    const extendedKeyMap = this.inlineOnly ? inlineKeymap : blocksKeymap

    const plugins = [
      keymap(extendedKeyMap),
      keymap(baseKeymap),
      dropCursor(),
      gapCursor(),
      history(),
      placeholder(),
    ]

    this.editorView = new EditorView(viewElem, {
      state: EditorState.create({
        doc: DOMParser.fromSchema(schema).parse(this.contentElem),
        plugins,
      }),
    })
  }

  render() {
    return html`
      <slot
        name="content"
        @slotchange=${(event: Event) => this.onContentSlotChange(event)}
      ></slot>

      <style>
        ${this.cssRules}
      </style>
      <style></style>
      <div part="content" id="prosemirror" class=${this.cssClass}></div>
    `
  }
}

/**
 * Plugin to add placeholder text if no content in document
 *
 * From https://discuss.prosemirror.net/t/how-to-input-like-placeholder-behavior/705/13
 */
function placeholder(text: string = 'Add content') {
  return new Plugin({
    props: {
      decorations(state) {
        const doc = state.doc

        if (
          doc.childCount > 1 ||
          !doc.firstChild?.isTextblock ||
          doc.firstChild?.content.size > 0
        )
          return

        const placeHolder = document.createElement('div')
        placeHolder.classList.add('placeholder')
        placeHolder.textContent = text

        return DecorationSet.create(doc, [Decoration.widget(1, placeHolder)])
      },
    },
  })
}
