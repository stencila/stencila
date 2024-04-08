import '@shoelace-style/shoelace/dist/components/icon/icon'
import {
  foldGutter,
  defaultHighlightStyle,
  LanguageDescription,
  syntaxHighlighting,
  LanguageSupport,
  StreamLanguage,
} from '@codemirror/language'
import { EditorState, Extension } from '@codemirror/state'
import { lineNumbers, EditorView } from '@codemirror/view'
import { NodeType } from '@stencila/types'
import { apply } from '@twind/core'
import { LitElement, html } from 'lit'
import { customElement, property } from 'lit/decorators'

import { withTwind } from '../../../twind'
import { nodeUi } from '../icons-and-colours'
import '../../buttons/chevron'

/**
 * A component for rendering the `code` property of `CodeStatic`, `CodeExecutable`
 * `Math`, and `Styled` nodes
 */
@customElement('stencila-ui-node-code')
@withTwind()
export class UINodeCode extends LitElement {
  @property()
  type: NodeType

  /**
   * The code to be rendered
   */
  @property()
  code: string

  /**
   * The language of the code. Used to determine the syntax highlighting
   */
  @property()
  language: string

  /**
   * Whether the code, and language, are readonly or not
   */
  @property({ type: Boolean, attribute: 'read-only' })
  readOnly: boolean = false

  /**
   * Whether the code shown be collapsed by default or not
   */
  @property({ type: Boolean })
  collapsed: boolean = false

  /**
   * A CodeMirror editor for the code
   */
  private editorView?: EditorView

  /**
   * Array of CodeMirror `LanguageDescription` objects available for the edit view
   */
  static languageDescriptions = [
    LanguageDescription.of({
      name: 'bash',
      alias: ['sh', 'shell'],
      load: async () => {
        return import('@codemirror/legacy-modes/mode/shell').then(
          (mode) => new LanguageSupport(StreamLanguage.define(mode.shell))
        )
      },
    }),
    LanguageDescription.of({
      name: 'jinja',
      load: async () => {
        return import('@codemirror/legacy-modes/mode/jinja2').then(
          (mode) => new LanguageSupport(StreamLanguage.define(mode.jinja2))
        )
      },
    }),
    LanguageDescription.of({
      name: 'javascript',
      alias: ['js'],
      load: async () => {
        return import('@codemirror/lang-javascript').then((obj) =>
          obj.javascript()
        )
      },
    }),
    LanguageDescription.of({
      name: 'latex',
      alias: ['tex'],
      load: async () => {
        return import('@codemirror/legacy-modes/mode/stex').then(
          (mode) => new LanguageSupport(StreamLanguage.define(mode.stexMath))
        )
      },
    }),
    LanguageDescription.of({
      name: 'python',
      alias: ['py'],
      load: async () => {
        return import('@codemirror/lang-python').then((obj) => obj.python())
      },
    }),
    LanguageDescription.of({
      name: 'r',
      load: async () => {
        return import('codemirror-lang-r').then((obj) => obj.r())
      },
    }),
    LanguageDescription.of({
      name: 'sql',
      load: async () => {
        return import('@codemirror/lang-sql').then((obj) => obj.sql())
      },
    }),
    LanguageDescription.of({
      name: 'xml',
      alias: ['mathml'],
      load: async () => {
        return import('@codemirror/lang-xml').then((obj) => obj.xml())
      },
    }),
  ]

  /**
   * Get the CodeMirror editor extensions
   */
  private async getEditorExtensions(): Promise<Extension[]> {
    const languageDescription = LanguageDescription.matchLanguageName(
      UINodeCode.languageDescriptions,
      this.language,
      true
    )

    let languageExtension: LanguageSupport[]
    if (languageDescription) {
      languageExtension = [await languageDescription.load()]
    } else {
      languageExtension = []
    }

    return [
      EditorView.editable.of(!this.readOnly),
      EditorState.readOnly.of(this.readOnly),
      ...languageExtension,
      lineNumbers(),
      foldGutter(),
      syntaxHighlighting(defaultHighlightStyle, { fallback: true }),
    ]
  }

  /**
   * Get the title of the language to display
   */
  protected getLanguageTitle(): string {
    switch (this.language.toLowerCase()) {
      case 'asciimath':
        return 'AsciiMath'
      case 'bash':
        return 'Bash'
      case 'jinja':
        return 'Jinja'
      case 'js':
        return 'JavaScript'
      case 'latex':
        return 'LaTeX'
      case 'mathml':
        return 'MathML'
      case 'py':
      case 'python':
        return 'Python'
      case 'r':
        return 'R'
      case 'rhai':
        return 'Rhai'
      case 'shell':
        return 'Shell'
      case 'sql':
        return 'SQL'
      case 'tex':
        return 'TeX'
      default:
        return this.language
    }
  }

  override update(changedProperties: Map<string, string | boolean>) {
    super.update(changedProperties)

    if (changedProperties.has('language')) {
      // Destroy the existing editor if there is one
      this.editorView?.destroy()

      // Create a new editor
      this.getEditorExtensions().then((extensions) => {
        this.editorView = new EditorView({
          parent: this.renderRoot.querySelector('#codemirror'),
          extensions,
          doc: this.code,
        })
      })
    } else if (changedProperties.has('code')) {
      // Update the editor state
      const view = this.editorView
      const state = view?.state
      if (view && state) {
        view.dispatch(
          state.update({
            changes: { from: 0, to: state.doc.length, insert: this.code },
          })
        )
      }
    }
  }

  override render() {
    const { borderColour } = nodeUi(this.type)

    const headerClasses = apply([
      'flex flex-row justify-between items-center',
      'px-4 py-1.5',
      `bg-[${borderColour}]`,
      'cursor-pointer',
    ])

    const contentClasses = apply([
      this.collapsed ? 'max-h-0' : 'max-h-full',
      'transition-max-h duration-200',
    ])

    // Unable to use `<stencila-ui-node-collapsible-property>` for this as that prevents
    // the CodeMirror stylesheet from being applied to the `<slot name="content">`
    return html`<div class="overflow-hidden">
      <div
        class=${headerClasses}
        @click=${() => (this.collapsed = !this.collapsed)}
      >
        <div class="flex items-center">
          <sl-icon name="code-square" class="text-base"></sl-icon>
          <span class="ml-4 text-sm">Code</span>
        </div>
        <div class="flex items-center">
          <stencila-chevron-button
            position=${this.collapsed ? 'left' : 'down'}
          ></stencila-chevron-button>
        </div>
      </div>

      <div class=${contentClasses}>
        <div hidden><slot></slot></div>
        <div id="codemirror" class=${`bg-gray-50`}></div>
      </div>
    </div>`
  }
}
