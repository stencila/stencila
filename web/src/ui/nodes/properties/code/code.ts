import '@shoelace-style/shoelace/dist/components/icon/icon'
import {
  foldGutter,
  defaultHighlightStyle,
  LanguageDescription,
  syntaxHighlighting,
  LanguageSupport,
  StreamLanguage,
} from '@codemirror/language'
import { Diagnostic, linter, setDiagnostics } from '@codemirror/lint'
import { EditorState, Extension } from '@codemirror/state'
import { lineNumbers, EditorView } from '@codemirror/view'
import { ExecutionRequired, NodeType } from '@stencila/types'
import { apply } from '@twind/core'
import { LitElement, PropertyValues, html } from 'lit'
import { customElement, property } from 'lit/decorators'

import { ExecutionMessage } from '../../../../nodes/execution-message'
import { withTwind } from '../../../../twind'
import '../../../buttons/chevron'

import { AuthorshipRun, AuthorshipMarker as AuthorshipMarker } from './types'
import {
  createProvenanceDecorations,
  stencilaTheme,
  provenanceTooltip,
  createLinterDiagnostics,
} from './utils'

/**
 * A component for rendering the `code` property of `CodeStatic`, `CodeExecutable`
 * `Math`, and `Styled` nodes
 */
@customElement('stencila-ui-node-code')
@withTwind()
export class UINodeCode extends LitElement {
  /**
   * The type of node whose code is being rendered
   */
  @property()
  type: NodeType

  /**
   * The code to be rendered
   */
  @property()
  code: string

  /**
   * The runs of authorship of the code
   */
  @property({ attribute: 'code-authorship', type: Array })
  codeAuthorship?: AuthorshipRun[]

  /**
   * The language of the code. Used to determine the syntax highlighting
   */
  @property()
  language: string

  /**
   * Whether execution is required, and if so, the reason
   *
   * Used to ignore execution messages if there has been changes
   * in the code since it was executed (in which case there could
   * be inconsistencies between the code and the messages).
   */
  @property({ attribute: 'execution-required' })
  executionRequired: ExecutionRequired

  /**
   * Whether line number and other gutters should be shown
   */
  @property({ type: Boolean, attribute: 'no-gutters' })
  noGutters: boolean = false

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
   * Classes to apply to the editor container
   */
  @property()
  containerClasses: string = 'border-t border-black/20'

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
      name: 'dot',
      alias: ['dotlang', 'graphviz'],
      load: async () => {
        return import('@viz-js/lang-dot').then((obj) => obj.dot())
      },
    }),
    LanguageDescription.of({
      name: 'html',
      load: async () => {
        return import('@codemirror/lang-html').then((obj) => obj.html())
      },
    }),
    LanguageDescription.of({
      name: 'json',
      load: async () => {
        return import('@codemirror/lang-json').then((obj) => obj.json())
      },
    }),
    LanguageDescription.of({
      name: 'javascript',
      alias: ['js', 'nodejs'],
      load: async () => {
        return import('@codemirror/lang-javascript').then((obj) =>
          obj.javascript()
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
   * Execution messages
   */
  private executionMessages: ExecutionMessage[] = []

  /**
   * Execution diagnostics associated with the code
   *
   * This state is maintained because it is returned from the `LinterSource`
   * function that is passed to the linter (see below).
   */
  private executionDiagnostics: Diagnostic[] = []

  /**
   * Get the CodeMirror editor extensions
   */
  private async getEditorExtensions(): Promise<Extension[]> {
    const languageDescription = LanguageDescription.matchLanguageName(
      UINodeCode.languageDescriptions,
      this.language,
      true
    )
    const languageExtension: LanguageSupport[] = languageDescription
      ? [await languageDescription.load()]
      : []

    const linterExtension = [linter(() => this.executionDiagnostics)]

    const authorshipMarkers = this.getAuthorshipMarkers()
    const authorshipExtensions = authorshipMarkers
      ? [
          EditorView.decorations.of(
            createProvenanceDecorations(authorshipMarkers)
          ),
          provenanceTooltip(authorshipMarkers, this.executionDiagnostics),
        ]
      : []

    return [
      EditorView.editable.of(!this.readOnly),
      EditorState.readOnly.of(this.readOnly),
      syntaxHighlighting(defaultHighlightStyle, { fallback: true }),
      ...languageExtension,
      ...linterExtension,
      ...authorshipExtensions,
      this.noGutters ? [] : [lineNumbers(), foldGutter()],
      stencilaTheme,
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

  /**
   * Takes the string value of the `code-authorship` property and attempts to
   * parse it into JS, if successful will convert the elements in `CodeAuthorshipMarker` objects
   *
   * Return `null` if value is falsy, or parsing fails
   * @returns `CodeAuthorshipMarker[] | null`
   */
  private getAuthorshipMarkers = (): AuthorshipMarker[] | null => {
    return this.codeAuthorship
      ? this.codeAuthorship.map((run) => ({
          from: run[0],
          to: run[1],
          count: run[2],
          provenance: run[4],
          mi: run[5],
        }))
      : null
  }

  /**
   * A mutation observer to update the `executionMessages` array when
   * the `execution-messages` slot changes
   */
  private executionMessagesObserver: MutationObserver

  /**
   * Handle a change, including on initial load, of the `execution-messages` slot
   */
  private onExecutionMessagesSlotChange(event: Event) {
    // Get the messages element
    const messagesElem = (event.target as HTMLSlotElement).assignedElements({
      flatten: true,
    })[0] as HTMLElement

    if (!messagesElem) {
      return
    }

    // Update messages
    this.updateExecutionMessages(messagesElem)

    // Also update the messages when the element is mutated
    this.executionMessagesObserver = new MutationObserver(() => {
      this.updateExecutionMessages(messagesElem)
    })
    this.executionMessagesObserver.observe(messagesElem, {
      childList: true,
    })
  }

  /**
   * Updates the `executionMessages` property, generates linter
   * diagnostics for them, and updates the editor with those diagnostics
   */
  private updateExecutionMessages(messagesElem?: HTMLElement) {
    if (messagesElem) {
      this.executionMessages = Array.from(messagesElem.children ?? []).filter(
        (message: ExecutionMessage) => message?.message?.length > 0
      ) as ExecutionMessage[]
    }

    if (this.editorView) {
      this.executionDiagnostics =
        this.executionRequired === 'SemanticsChanged' ||
        this.executionRequired === 'StateChanged'
          ? []
          : createLinterDiagnostics(this.editorView, this.executionMessages)

      const transaction = setDiagnostics(
        this.editorView.state,
        this.executionDiagnostics
      )
      this.editorView.dispatch(transaction)
    }
  }

  override update(changedProperties: PropertyValues) {
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

    if (changedProperties.has('executionRequired')) {
      this.updateExecutionMessages()
    }
  }

  override render() {
    const containerClasses = apply([this.containerClasses])

    const contentClasses = apply([
      'text-black',
      this.collapsed ? 'max-h-0' : 'max-h-full',
      'transition-max-h duration-200',
    ])

    // Unable to use `<stencila-ui-node-collapsible-property>` for this as that prevents
    // the CodeMirror stylesheet from being applied to the `<slot name="content">`
    return html`
      <div class=${containerClasses}>
        <div class=${contentClasses}>
          <div hidden>
            <slot
              name="execution-messages"
              @slotchange=${this.onExecutionMessagesSlotChange}
            ></slot>
          </div>
          <div id="codemirror" class="bg-gray-50"></div>
        </div>
      </div>
    `
  }
}
