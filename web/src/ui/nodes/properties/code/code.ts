import '@shoelace-style/shoelace/dist/components/icon/icon'
import { defaultKeymap, history, historyKeymap } from '@codemirror/commands'
import {
  foldGutter,
  defaultHighlightStyle,
  LanguageDescription,
  syntaxHighlighting,
  LanguageSupport,
  StreamLanguage,
} from '@codemirror/language'
import { Diagnostic, linter, setDiagnostics } from '@codemirror/lint'
import { Compartment, EditorState, Extension } from '@codemirror/state'
import { lineNumbers, EditorView, keymap, ViewUpdate } from '@codemirror/view'
import { ExecutionRequired, NodeType } from '@stencila/types'
import { apply } from '@twind/core'
import { LitElement, PropertyValues, html } from 'lit'
import { customElement, property } from 'lit/decorators'

import { patchValue, patchValueExecute } from '../../../../clients/commands'
import { CompilationMessage } from '../../../../nodes/compilation-message'
import { ExecutionMessage } from '../../../../nodes/execution-message'
import { withTwind } from '../../../../twind'
import { isCSSFeatureVisible } from '../../../../utilities/cssVariables'
import '../../../buttons/chevron'

import { createTheme } from './theme'
import { AuthorshipRun, AuthorshipMarker as AuthorshipMarker } from './types'
import {
  createProvenanceDecorations,
  provenanceTooltip,
  createLinterDiagnostics,
  clipBoardKeyBindings,
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
   * Id of the node whose code is being rendered
   */
  @property({ attribute: 'node-id' })
  nodeId: string

  /**
   * Name of the node property that the code is for
   *
   * Used when sending patches to update the code.
   */
  @property({ attribute: 'node-property' })
  nodeProperty: string = 'code'

  /**
   * The code to be rendered
   */
  @property()
  code: string

  /**
   * The runs of authorship of the code.
   *
   * **IMPORTANT** this will be a stringified array, and must be parsed before using
   */
  @property({ attribute: 'code-authorship', type: String })
  codeAuthorship?: string

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
   * Limits the editor to single line
   */
  @property({ type: Boolean, attribute: 'single-line' })
  singleLine: boolean = false

  /**
   * A CodeMirror editor for the code
   */
  private editorView?: EditorView

  // Compartments for toggling on editor functionality

  private viewEditable: Compartment

  private viewReadOnly: Compartment

  private viewAuthorship: Compartment

  private authorshipExtensions: Extension | Extension[] = []

  /**
   * Whether the code is currently being edited
   *
   * Used to ignore incoming changes from the server
   * (patches applied to the `code` attribute) while the
   * user is actively editing the code.
   */
  isBeingEdited: boolean = false

  /**
   * Runs the current code
   *
   * Dispatches a `path-value-execute` command so the code is up to date upon execution
   */
  private runCode() {
    const value = this.editorView.state.doc.toString()
    this.dispatchEvent(
      patchValueExecute(this.type, this.nodeId, this.nodeProperty, value)
    )
  }

  /**
   * Returns a codemirror `Extension`, which dispatches a
   * debounced `patchValue` event when the document is
   * changed via user input.
   */
  private patchInput() {
    // Millisecond debounce for debouncing timers
    const BEING_EDITED_TIMEOUT = 5_000
    const PATCH_DEBOUNCE = 300

    let isBeingEditedTimer: NodeJS.Timeout
    let patchTimer: NodeJS.Timeout
    return EditorView.updateListener.of((update: ViewUpdate) => {
      if (update.docChanged) {
        // Check whether this is a user event so we can ignore changes coming in
        // from the server. It is necessary to list all user event types that may
        // change the content
        // See https://codemirror.net/docs/ref/#state.Transaction^userEvent
        let isUserEvent = false
        for (const t of update.transactions) {
          if (
            t.isUserEvent('input') ||
            t.isUserEvent('delete') ||
            t.isUserEvent('move') ||
            t.isUserEvent('select') ||
            t.isUserEvent('undo') ||
            t.isUserEvent('redo')
          ) {
            isUserEvent = true
            break
          }
        }

        if (!isUserEvent) {
          return
        }

        // Ignore any incoming changes from server for a while
        this.isBeingEdited = true
        clearTimeout(isBeingEditedTimer)
        isBeingEditedTimer = setTimeout(() => {
          this.isBeingEdited = false
        }, BEING_EDITED_TIMEOUT)

        // Send patch to server with debounce
        const newValue = update.state.doc.toString()
        clearTimeout(patchTimer)
        patchTimer = setTimeout(() => {
          this.dispatchEvent(
            patchValue(this.type, this.nodeId, this.nodeProperty, newValue)
          )
        }, PATCH_DEBOUNCE)
      }
    })
  }

  /**
   * Update the editable and readonly extension of the `editorView`, and toggle authorship extensions
   */
  private updateViewEditability() {
    this.editorView.dispatch({
      effects: [
        this.viewEditable.reconfigure(EditorView.editable.of(!this.readOnly)),
        this.viewReadOnly.reconfigure(EditorState.readOnly.of(this.readOnly)),
        this.viewAuthorship.reconfigure(
          this.readOnly ? this.authorshipExtensions : []
        ),
      ],
    })
  }

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
      name: 'css',
      load: async () => {
        return import('@codemirror/lang-css').then((obj) => obj.css())
      },
    }),
    LanguageDescription.of({
      name: 'cypher',
      alias: ['kuzu', 'docs'],
      load: async () => {
        return import('@codemirror/legacy-modes/mode/cypher').then(
          (mode) => new LanguageSupport(StreamLanguage.define(mode.cypher))
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
      // JSON visualization specs rendered by the Jviz kernel
      alias: ['cytoscape', 'echarts', 'plotly', 'vegalite'],
      load: async () => {
        return import('@codemirror/lang-json').then((obj) => obj.json())
      },
    }),
    LanguageDescription.of({
      name: 'javascript',
      alias: ['js', 'quickjs', 'nodejs'],
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
      name: 'mermaid',
      load: async () => {
        return import('codemirror-lang-mermaid').then((obj) => obj.mermaid())
      },
    }),
    LanguageDescription.of({
      name: 'python',
      // Previously 'docsql' was an alias for 'jinja'. However, the Jinja mode
      // oly highlights Jinja syntax like `{{% if a < 1 %}}` and `{{ 1 + 3 }}`.
      // It does not highlight bare Jinja expressions. So given, keyword-style
      // syntax for filters in DocsQL are similar to Python, putting that alias
      // here.
      alias: ['py', 'docsql'],
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
      alias: ['mathml', 'svg'],
      load: async () => {
        return import('@codemirror/lang-xml').then((obj) => obj.xml())
      },
    }),
    LanguageDescription.of({
      name: 'yaml',
      alias: ['yml'],
      load: async () => {
        return import('@codemirror/lang-yaml').then((obj) => obj.yaml())
      },
    }),
  ]

  /**
   * Compilation and execution messages
   */
  private messages: (CompilationMessage | ExecutionMessage)[] = []

  /**
   * Diagnostics associated with the code
   *
   * This state is maintained because it is returned from the `LinterSource`
   * function that is passed to the linter (see below).
   */
  private diagnostics: Diagnostic[] = []

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

    const linterExtension = [linter(() => this.diagnostics)]

    const authorshipMarkers = this.getAuthorshipMarkers()
    this.authorshipExtensions = authorshipMarkers
      ? [
          EditorView.decorations.of(
            createProvenanceDecorations(authorshipMarkers)
          ),
          provenanceTooltip(authorshipMarkers, this.diagnostics),
        ]
      : []

    this.viewEditable = new Compartment()
    this.viewReadOnly = new Compartment()
    this.viewAuthorship = new Compartment()

    const singleLineEditor = EditorState.transactionFilter.of((tr) =>
      tr.newDoc.lines > 1 ? [] : tr
    )

    const filteredKeyMap = defaultKeymap.filter((kb) => {
      if (kb.key) {
        return !['mod-enter', 'ctrl-enter'].includes(kb.key.toLowerCase())
      }
      return true
    })

    return [
      this.patchInput(),
      this.viewEditable.of(EditorView.editable.of(!this.readOnly)),
      this.viewReadOnly.of(EditorState.readOnly.of(this.readOnly)),
      syntaxHighlighting(defaultHighlightStyle, { fallback: true }),
      this.singleLine ? singleLineEditor : [],
      ...languageExtension,
      ...linterExtension,
      this.viewAuthorship.of(this.readOnly ? this.authorshipExtensions : []),
      this.getGutterExtensions(),
      history(),
      keymap.of(filteredKeyMap),
      keymap.of(historyKeymap),
      keymap.of(clipBoardKeyBindings),
      createTheme(),
      keymap.of([
        {
          key: 'Ctrl-Enter',
          run: () => {
            this.runCode()
            return true
          },
        },
      ]),
    ]
  }

  /**
   * Get gutter extensions based on CSS custom properties
   */
  private getGutterExtensions(): Extension[] {
    const extensions: Extension[] = []

    if (isCSSFeatureVisible(this, '--code-line-numbers', true)) {
      extensions.push(lineNumbers())
    }

    if (isCSSFeatureVisible(this, '--code-fold-gutter', true)) {
      extensions.push(foldGutter())
    }

    return extensions
  }

  /**
   * Takes the string value of the `code-authorship` property and attempts to
   * parse it as a jSoN array. If successful will convert the item of the
   * array into `CodeAuthorshipMarker` objects
   *
   * Returns `null` if value is falsy, empty, or if parsing fails
   */
  private getAuthorshipMarkers = (): AuthorshipMarker[] | null => {
    if (this.codeAuthorship) {
      try {
        const authorshipRun = JSON.parse(this.codeAuthorship) as AuthorshipRun[]

        if (authorshipRun.length == 0) {
          return null
        }

        return authorshipRun.map((run) => ({
          from: run[0],
          to: run[1],
          count: run[2],
          provenance: run[4],
          mi: run[5],
        }))
        // eslint-disable-next-line @typescript-eslint/no-unused-vars
      } catch (e) {
        return null
      }
    }
    return null
  }

  /**
   * A mutation observer to update the `messages` array when
   * the `messages` slot changes
   */
  private messagesObserver: MutationObserver

  /**
   * Handle a change, including on initial load, of the `messages` slot
   */
  private onMessagesSlotChange(event: Event) {
    // Get the messages elements: may be more than one, compilation and execution messages
    // are both assigned to the messages slot
    const messagesElems = (event.target as HTMLSlotElement).assignedElements({
      flatten: true,
    }) as HTMLElement[]

    if (!messagesElems || messagesElems.length == 0) {
      return
    }

    // Update messages
    this.updateMessages(messagesElems)

    // Also update the messages when the element is mutated
    this.messagesObserver = new MutationObserver(() => {
      this.updateMessages(messagesElems)
    })
    for (const messagesElem of messagesElems) {
      this.messagesObserver.observe(messagesElem, {
        attributes: true,
        childList: true,
        subtree: true,
      })
    }
  }

  /**
   * Updates the `messages` property, generates linter
   * diagnostics for them, and updates the editor with those diagnostics
   */
  private updateMessages(messagesElems: HTMLElement[]) {
    this.messages = messagesElems.flatMap((elem) =>
      Array.from(elem.children)
    ) as (CompilationMessage | ExecutionMessage)[]

    if (this.editorView) {
      this.updateDiagnostics(
        createLinterDiagnostics(this.editorView, this.messages)
      )
    }
  }

  /**
   * Update the `diagnostics` property and the editor with those
   */
  private updateDiagnostics(diagnostics: Diagnostic[]) {
    if (this.editorView) {
      this.diagnostics = diagnostics
      const transaction = setDiagnostics(
        this.editorView.state,
        this.diagnostics
      )
      this.editorView.dispatch(transaction)
    }
  }

  override updated(changedProperties: PropertyValues) {
    super.updated(changedProperties)

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
        // Update diagnostics when editor first created
        this.updateDiagnostics(
          createLinterDiagnostics(this.editorView, this.messages)
        )
      })
    } else if (changedProperties.has('readOnly')) {
      // update the editorView readonly state
      this.updateViewEditability()
    } else if (changedProperties.has('code')) {
      // Update the editor state if not currently being edited
      const view = this.editorView
      const state = view?.state
      if (!this.isBeingEdited && view && state) {
        view.dispatch(
          state.update({
            changes: { from: 0, to: state.doc.length, insert: this.code },
          })
        )
      }
    }

    // Clear diagnostics if the code or language has changed either explicitly, or
    // via an update to executionRequired
    if (
      changedProperties.has('code') ||
      changedProperties.has('language') ||
      (changedProperties.has('executionRequired') &&
        ['StateChanged', 'SemanticsChanged'].includes(this.executionRequired))
    ) {
      this.updateDiagnostics([])
    }
  }

  override render() {
    const contentClasses = apply([
      'text-black',
      this.collapsed ? 'max-h-0' : 'max-h-full',
      'transition-max-h duration-200',
    ])

    // Unable to use `<stencila-ui-node-collapsible-property>` for this as that prevents
    // the CodeMirror stylesheet from being applied to the `<slot name="content">`
    return html`
      <div class=${contentClasses}>
        <div hidden>
          <slot
            name="messages"
            @slotchange=${this.onMessagesSlotChange}
          ></slot>
        </div>
        <div id="codemirror" class="bg-gray-50"></div>
      </div>
    `
  }
}
