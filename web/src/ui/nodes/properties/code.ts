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
 * A component for displaying the `code` property of `CodeStatic`, `CodeExecutable`
 * and `Math` nodes
 */
@customElement('stencila-ui-node-code')
@withTwind()
export class UINodeCode extends LitElement {
  @property()
  type: NodeType

  /**
   * The language of the code. Used to determine the syntax highlighting.
   */
  @property()
  language: string

  /**
   * Whether the code, and language, are readonly or not
   */
  @property({ type: Boolean, attribute: 'read-only' })
  readonly: boolean = false

  /**
   * Whether the code shown be collapsed by default or not
   */
  @property({ type: Boolean })
  collapsed: boolean = false

  /**
   * A CodeMirror editor for the code
   */
  private editorView: EditorView

  /**
   * Array of CodeMirror `LanguageDescription` objects available for the edit view
   *
   * Note: The first language description is used as the default.
   */
  static languageDescriptions = [
    LanguageDescription.of({
      name: 'javascript',
      extensions: ['js'],
      load: async () => {
        return import('@codemirror/lang-javascript').then((obj) =>
          obj.javascript()
        )
      },
    }),
    LanguageDescription.of({
      name: 'latex',
      extensions: ['latex', 'tex'],
      load: async () => {
        return import('@codemirror/legacy-modes/mode/stex').then(
          (mode) => new LanguageSupport(StreamLanguage.define(mode.stexMath))
        )
      },
    }),
    LanguageDescription.of({
      name: 'python',
      extensions: ['py'],
      load: async () => {
        return import('@codemirror/lang-python').then((obj) => obj.python())
      },
    }),
    LanguageDescription.of({
      name: 'r',
      extensions: ['r'],
      load: async () => {
        return import('codemirror-lang-r').then((obj) => obj.r())
      },
    }),
    LanguageDescription.of({
      name: 'sql',
      extensions: ['sql'],
      load: async () => {
        return import('@codemirror/lang-sql').then((obj) => obj.sql())
      },
    }),
  ]

  /**
   * Get the CodeMirror editor extensions
   */
  private async getEditorExtensions(): Promise<Extension[]> {
    const lang =
      LanguageDescription.matchLanguageName(
        UINodeCode.languageDescriptions,
        this.language
      ) ?? UINodeCode.languageDescriptions[0]

    return [
      EditorView.editable.of(!this.readonly),
      EditorState.readOnly.of(this.readonly),
      await lang.load(),
      lineNumbers(),
      foldGutter(),
      syntaxHighlighting(defaultHighlightStyle, { fallback: true }),
    ]
  }

  /**
   * Get the title of the language to display
   */
  private getLanguageTitle(): string {
    switch (this.language.toLowerCase()) {
      case 'js':
        return 'JavaScript'
      case 'latex':
        return 'LaTeX'
      case 'py':
      case 'python':
        return 'Python'
      case 'r':
        return 'R'
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

      // Get the code content
      const code =
        this.shadowRoot
          .querySelector('slot')
          ?.assignedElements({ flatten: true })[0]?.textContent ?? ''

      // Create a new editor
      this.getEditorExtensions().then((extensions) => {
        this.editorView = new EditorView({
          parent: this.renderRoot.querySelector('#codemirror'),
          extensions,
          doc: code,
        })
      })
    }
  }

  override render() {
    const { colour, borderColour } = nodeUi(this.type)

    const headerClasses = apply([
      'flex flex-row justify-between items-center',
      'px-6 py-1.5',
      `bg-[${borderColour}]`,
      'cursor-pointer',
    ])

    const languageClasses = apply([
      'flex items-center',
      `bg-[${colour}]`,
      'px-1.5 py-0.5 mr-3',
      'rounded-full',
      'text-xs',
    ])

    const contentClasses = apply([
      this.collapsed ? 'max-h-0' : 'max-h-full',
      'transition-max-h duration-200',
    ])

    const language = this.getLanguageTitle()

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
          <span class=${languageClasses}>
            <span>${language}</span>
          </span>
          <stencila-chevron-button
            position=${this.collapsed ? 'left' : 'down'}
          ></stencila-chevron-button>
        </div>
      </div>

      <div class=${contentClasses}>
        <div hidden><slot></slot></div>
        <div id="codemirror" class="bg-gray-50"></div>
      </div>
    </div>`
  }
}
