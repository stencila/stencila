import {
  autocompletion,
  startCompletion,
  completionKeymap,
  CompletionContext,
  CompletionResult,
} from '@codemirror/autocomplete'
import { history, historyKeymap, indentWithTab } from '@codemirror/commands'
import {
  foldGutter,
  bracketMatching,
  defaultHighlightStyle,
  indentOnInput,
  LanguageDescription,
  LanguageSupport,
  syntaxHighlighting,
  StreamLanguage,
} from '@codemirror/language'
import { searchKeymap, search } from '@codemirror/search'
import { Extension, Compartment, StateEffect } from '@codemirror/state'
import {
  dropCursor,
  EditorView as CodeMirrorView,
  highlightActiveLineGutter,
  highlightActiveLine,
  highlightSpecialChars,
  keymap,
  lineNumbers,
} from '@codemirror/view'
import { css as twCSS } from '@twind/core'
import { html } from 'lit'
import { customElement, property } from 'lit/decorators'
import { ref, Ref, createRef } from 'lit/directives/ref'

import { CodeMirrorClient } from '../clients/codemirror'
import { DomClient } from '../clients/dom'
import { MappingEntry } from '../clients/format'
import { markdownHighlightStyle } from '../languages/markdown'
import type { DocumentId, DocumentAccess } from '../types'
import { TWLitElement } from '../ui/twind'

import { bottomPanel } from './source/bottom-panel'
import { editorStyles } from './source/editorStyles'
import { statusGutter } from './source/gutters'
import { infoSideBar } from './source/infoSideBar'
import { autoWrapKeys } from './source/keyMaps'

/**
 * Source code editor for a document
 *
 * A view which provides read-write access to the document using
 * a particular format.
 */
@customElement('stencila-source-view')
export class SourceView extends TWLitElement {
  /**
   * The id of the document
   */
  @property()
  doc: DocumentId

  /**
   * The access level of the editor
   *
   * This property is passed through to the `CodeMirrorClient`
   * and used to determine whether or not the document is
   * read-only or writable.
   *
   * This should not be `edit`, `write` or `admin` since this view
   * does not provide the means to modify those.
   */
  @property()
  access: DocumentAccess = 'write'

  /**
   * The format of the source code
   */
  @property()
  format: string = 'Markdown'

  /**
   * Turn on/off editor line wrapping
   */
  @property({ attribute: 'line-wrapping', type: Boolean })
  lineWrap: boolean = true

  /**
   * Turn on/off the write only mode for the source view.
   * This disables the SourceView instance from recieving
   * and reacting to messages from the server.
   */
  @property({ attribute: 'write-only', type: Boolean })
  writeOnly: boolean = false

  /**
   * Where is this component rendered? Either as a single view or in a split
   * code & preview editor?
   */
  @property()
  displayMode?: 'single' | 'split' = 'single'

  /**
   * A read-only client which updates an invisible DOM element when the
   * document changes on the server. We use this to extract custom elements
   * for nodes to use in tooltips etc.
   */
  // @ts-expect-error "dom client is set, but not read"
  private domClient: DomClient

  /**
   * A ref for the hidden `DomClient` element
   */
  public domElement: Ref<HTMLElement> = createRef()

  /**
   * A read-write client which sends and receives string patches
   * for the source code to and from the server
   */
  private codeMirrorClient: CodeMirrorClient

  /**
   * A CodeMirror editor view which the client interacts with
   */
  private codeMirrorView: CodeMirrorView

  /**
   * `Compartment` for setting `CodeMirrorView.lineWrapping` extension
   */
  private lineWrappingConfig = new Compartment()

  /**
   * `Compartment` for setting codemirror extensions which rely
   * on receiving messages from the `codeMirrorClient`
   */
  private clientRecieverConfig = new Compartment()

  /**
   * Array of CodeMirror `LanguageDescription` objects available for the edit view
   *
   * Note: The first language description is used as the default.
   */
  static languageDescriptions = [
    LanguageDescription.of({
      name: 'markdown',
      extensions: ['md'],
      load: async () => {
        return import('../languages/markdown').then((md) =>
          md.stencilaMarkdown()
        )
      },
    }),
    LanguageDescription.of({
      name: 'jats',
      extensions: ['jats.xml'],
      load: async () => {
        return import('@codemirror/lang-xml').then((obj) => obj.xml())
      },
    }),
    LanguageDescription.of({
      name: 'json',
      extensions: ['json'],
      load: async () => {
        return import('@codemirror/lang-json').then((obj) => obj.json())
      },
    }),
    LanguageDescription.of({
      name: 'json5',
      extensions: ['json5'],
      load: async () => {
        return import('codemirror-json5').then((obj) => obj.json5())
      },
    }),
    LanguageDescription.of({
      name: 'html',
      extensions: ['html'],
      load: async () => {
        return import('@codemirror/lang-html').then((obj) => obj.html())
      },
    }),
    LanguageDescription.of({
      name: 'yaml',
      extensions: ['yaml', 'yml'],
      load: async () => {
        return import('@codemirror/legacy-modes/mode/yaml').then(
          (yml) => new LanguageSupport(StreamLanguage.define(yml.yaml))
        )
      },
    }),
    LanguageDescription.of({
      name: 'dom',
      extensions: ['dom'],
      load: async () => {
        return import('@codemirror/lang-html').then((obj) => obj.html())
      },
    }),
  ]

  /**
   * Dispatch a CodeMirror `StateEffect` to the editor
   */
  private dispatchEffect(effect: StateEffect<unknown>) {
    const docState = this.codeMirrorView?.state

    const transaction =
      docState?.update({
        effects: [effect],
      }) ?? {}

    this.codeMirrorView?.dispatch(transaction)
  }

  private stencilaCompleteOptions(
    context: CompletionContext
  ): CompletionResult {
    const char = context.matchBefore(/:+/)
    if (char?.from == char?.to && !context.explicit) return null
    const suggestions: string[] = [':::', '::: if', '::: for']

    return {
      from: char.from,
      options: suggestions.map((label) => ({
        label,
        type: 'keyword',
        apply: label,
      })),
    }
  }

  /**
   * Get the CodeMirror `LanguageSupport` for a particular format
   *
   * Defaults to the first `SourceView.languageDescriptions` if it does no
   * matching language extension is found.
   *
   * @param {string} format `format` parameter of the source view
   * @returns `LanguageSupport` instance
   */
  private async getLanguageExtension(format: string): Promise<LanguageSupport> {
    const ext =
      LanguageDescription.matchLanguageName(
        SourceView.languageDescriptions,
        format
      ) ?? SourceView.languageDescriptions[0]

    return await ext.load()
  }

  /**
   * Send an 'execute' operation on the selection of the document
   *
   * @returns false to tell CodeMirror that this does not change the editor state
   */
  private executeSelection(view: CodeMirrorView): boolean {
    this.codeMirrorClient.sendSpecial('execute', view.state.selection.main)
    return false
  }

  /**
   * Get the CodeMirror editor view extensions
   */
  private async getViewExtensions(): Promise<Extension[]> {
    const langExt = await this.getLanguageExtension(this.format)

    const lineWrapping = this.lineWrappingConfig.of(CodeMirrorView.lineWrapping)

    const clientReceiver = this.clientRecieverConfig.of(
      !this.writeOnly ? [statusGutter(this), infoSideBar(this)] : []
    )

    const keyMaps = keymap.of([
      indentWithTab,
      ...historyKeymap,
      ...completionKeymap,
      ...searchKeymap,
      { key: 'Ctrl-Space', run: startCompletion },
      { key: 'Ctrl-Enter', run: (view) => this.executeSelection(view) },
      ...autoWrapKeys,
    ])

    const syntaxHighlights =
      this.format === 'markdown'
        ? markdownHighlightStyle
        : defaultHighlightStyle

    const extensions = [
      langExt,
      keyMaps,
      history(),
      search({ top: true }),
      lineNumbers(),
      foldGutter(),
      lineWrapping,
      autocompletion({ override: [this.stencilaCompleteOptions] }),
      dropCursor(),
      highlightActiveLine(),
      highlightActiveLineGutter(),
      indentOnInput(),
      highlightSpecialChars(),
      syntaxHighlighting(syntaxHighlights, { fallback: true }),
      bracketMatching(),
      autocompletion(),
      bottomPanel(this),
      editorStyles,
      clientReceiver,
    ]

    return extensions
  }

  /**
   * Override `LitElement.firstUpdated` so that `DomClient` is instantiated _after_ this
   * element has a document `[root]` element in its `renderRoot`.
   */
  override firstUpdated(changedProperties: Map<string, string | boolean>) {
    super.firstUpdated(changedProperties)

    this.domClient = new DomClient(
      this.doc,
      this.renderRoot.querySelector('[root]') as HTMLElement
    )
  }

  /**
   * Override `LitElement.update` to dispatch any changes to editor config
   * to the editor.
   */
  override async update(changedProperties: Map<string, string | boolean>) {
    super.update(changedProperties)

    if (changedProperties.has('format')) {
      // Destroy the existing editor if there is one
      this.codeMirrorView?.destroy()

      // Setup client and editor for the format
      this.getViewExtensions().then((extensions) => {
        this.codeMirrorClient = new CodeMirrorClient(
          this.doc,
          this.access,
          this.format,
          this.writeOnly
        )

        this.codeMirrorView = new CodeMirrorView({
          extensions: [this.codeMirrorClient.sendPatches(), ...extensions],
          parent: this.renderRoot.querySelector('#codemirror'),
        })

        this.codeMirrorClient.receivePatches(this.codeMirrorView)
      })
    }

    if (changedProperties.has('lineWrap')) {
      this.dispatchEffect(
        this.lineWrappingConfig.reconfigure(
          this.lineWrap ? CodeMirrorView.lineWrapping : []
        )
      )
    }

    if (changedProperties.has('writeOnly')) {
      // set codeMirrorClient property
      this.codeMirrorClient.writeOnly = this.writeOnly
      // remove/add required extensions
      this.dispatchEffect(
        this.clientRecieverConfig.reconfigure(
          !this.writeOnly ? [statusGutter(this), infoSideBar(this)] : []
        )
      )
    }
  }

  /**
   * Get the Stencila Schema node, and property name (if any), corresponding to a character position
   *
   * @param position The character position. Defaults to the current cursor position.
   */
  public getNodeAt(position?: number): MappingEntry | undefined {
    position = position ?? this.codeMirrorView.state.selection.main.from

    return this.codeMirrorClient.nodeAt(position)
  }

  /**
   * Get the hierarchy of Stencila Schema nodes corresponding to a character position
   *
   * @param position The character position. Defaults to the current cursor position.
   */
  public getNodesAt(position?: number): MappingEntry[] {
    position = position ?? this.codeMirrorView.state.selection.main.from

    return this.codeMirrorClient.nodesAt(position)
  }

  /**
   * CSS styling for the CodeMirror editor
   *
   * Overrides some of the default styles used by CodeMirror.
   *
   * This needs to be defined outside of the static styles property, as we need
   * to be able to access the instance property "displayMode" to determine the
   * height to use.
   */
  private get codeMirrorCSS() {
    return twCSS`
      .cm-editor {
        height: 100%;
        max-width: calc(100vw-4rem);
      }
    `
  }

  protected override render() {
    return html`
      <div class="max-h-screen h-[calc(100vh-5rem)] relative">
        <div id="codemirror" class="h-full ${this.codeMirrorCSS}"></div>
      </div>
      <div hidden ${ref(this.domElement)}>
        <stencila-article root></stencila-article>
      </div>
    `
  }
}
