import { css, html } from 'lit'
import { customElement, property } from 'lit/decorators'
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

import StencilaElement from '../../utils/element'
import { articleInputRules } from './input-rules'
import { articleSchema, nodeViews } from './nodes'
import { placeholder } from './plugins'

const inlinesKeymap = {
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
    // with EDITs noted below.
    css`
      .ProseMirror {
        position: relative;
      }

      .ProseMirror {
        word-wrap: break-word;
        /*
        EDIT: These are excluded aas they affect the styling of custom components
        within the editor.

        white-space: pre-wrap; 
        white-space: break-spaces;
        */
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
   * The schema that this editor should use
   */
  @property({ attribute: 'schema' })
  schema: string = 'article'

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
   * The ProseMirror editor view
   */
  private editorView?: EditorView

  /**
   * Handle a change to the `content` slot (including initial render) to get the
   * DOM of the content and initialize the editor with it
   */
  protected onSlotChange(event: Event) {
    let contentElem = (event.target as HTMLSlotElement).assignedElements({
      flatten: true,
    })[0] as HTMLDivElement | HTMLSpanElement | undefined

    // Return early if no nodes yet assigned to the slot
    if (contentElem === undefined) {
      return
    }

    const viewElem = this.renderRoot.querySelector('.pm')

    let schema
    let extendedKeyMap
    if (this.schema === 'article') {
      schema = articleSchema
      extendedKeyMap = blocksKeymap
    } else {
      throw new Error(`Unknown schema: ${this.schema}`)
    }

    // Parse the content and then remove it so that there are not
    // duplicate ids.
    const doc = DOMParser.fromSchema(schema).parse(contentElem)
    // Removal is done after a very short timeout to avoid exceptions with
    // removal of event handlers (which perhaps did not get a chance to get added
    // without the delay?)
    setTimeout(() => contentElem?.remove(), 1)

    const plugins = [
      // Locally defined input rules
      articleInputRules, // Should go before keymap
      keymap(extendedKeyMap),
      keymap(baseKeymap),
      // TODO; See if only enabling drop cursor on inner works
      dropCursor(),
      gapCursor(),
      history(),
      placeholder(),
    ]

    this.editorView = new EditorView(viewElem, {
      state: EditorState.create({
        doc,
        plugins,
      }),
      nodeViews,
    })
  }

  render() {
    return html`
      <slot
        style="display:none"
        @slotchange=${(event: Event) => this.onSlotChange(event)}
      ></slot>

      <style>
        ${this.cssRules}
      </style>

      <div part="content" class="pm ${this.cssClass}"></div>
    `
  }
}
