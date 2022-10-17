import { camelCase, capitalCase } from 'change-case'
import { html } from 'lit'
import { customElement, property } from 'lit/decorators'
import { css } from 'twind/css'

import {
  autocompletion,
  closeBrackets,
  closeBracketsKeymap,
  completeAnyWord,
  completionKeymap,
  startCompletion,
} from '@codemirror/autocomplete'
import {
  defaultKeymap,
  history,
  historyKeymap,
  indentWithTab,
} from '@codemirror/commands'
import {
  bracketMatching,
  defaultHighlightStyle,
  foldGutter,
  foldKeymap,
  indentOnInput,
  LanguageDescription,
  LanguageSupport,
  StreamLanguage,
  syntaxHighlighting,
} from '@codemirror/language'
import { languages } from '@codemirror/language-data'
import { lintKeymap } from '@codemirror/lint'
import { highlightSelectionMatches, searchKeymap } from '@codemirror/search'
import {
  Compartment,
  EditorState,
  Extension,
  StateEffect,
} from '@codemirror/state'
import {
  crosshairCursor,
  drawSelection,
  dropCursor,
  EditorView,
  highlightActiveLineGutter,
  highlightSpecialChars,
  keymap,
  lineNumbers,
  rectangularSelection,
} from '@codemirror/view'

import * as themes from 'thememirror/dist'

import '@shoelace-style/shoelace/dist/components/icon/icon'
import '@shoelace-style/shoelace/dist/components/menu-item/menu-item'
import '@shoelace-style/shoelace/dist/components/select/select'
import '@shoelace-style/shoelace/dist/components/switch/switch'

import SlSwitch from '@shoelace-style/shoelace/dist/components/switch/switch'
import { updateToOps } from '../../patches/codemirror'
import { twSheet, varApply, varLocal } from '../utils/css'
import StencilaElement from '../utils/element'

const { tw, sheet } = twSheet()

@customElement('stencila-code-editor')
export default class StencilaCodeEditor extends StencilaElement {
  static styles = [sheet.target]

  /**
   * The code language
   */
  @property({ reflect: true })
  language?: string

  /**
   * A list of languages supported by this editor
   *
   * This list is presented in a dropdown language selector.
   * If the list is empty, no selector will be provided.
   */
  @property({ type: Array })
  private languages = [
    'C',
    'C++',
    'Calc',
    'CSS',
    'Dockerfile',
    'Go',
    'Haskell',
    'HTML',
    'Java',
    'JavaScript',
    'JSON',
    'Julia',
    'LaTeX',
    'Markdown',
    'PLSQL',
    'PrQL',
    'Python',
    'R',
    'Ruby',
    'Rust',
    'Shell',
    'SQL',
    'SQLite',
    'TOML',
    'TypeScript',
    'XML',
    'YAML',
    'Other',
  ]

  /**
   * An alternative to specifying `language`
   *
   * Used to resolve the language of the code on initialization.
   */
  @property()
  filename?: string

  /**
   * Whether the editor is read-only i.e. only syntax highlighting
   */
  @property({ attribute: 'read-only', type: Boolean })
  readOnly: boolean = false

  /**
   * Whether the editor is single line
   */
  @property({ attribute: 'single-line', type: Boolean })
  singleLine: boolean = false

  /**
   * Whether line wrapping is on
   */
  @property({ attribute: 'line-wrapping', type: Boolean })
  lineWrapping: boolean = false

  /**
   * The editor theme
   */
  @property({ reflect: true })
  theme?: string =
    window
      .getComputedStyle(document.documentElement)
      .getPropertyValue('--stencila-code-editor-theme')
      .trim() ||
    window.localStorage.getItem('StencilaCodeEditor.theme') ||
    'tomorrow'

  /**
   * A list of themes supported by this editor
   *
   * This list is presented in a dropdown theme selector.
   * If the list is empty, no selector will be provided.
   */
  @property({ type: Array })
  private themes = Object.keys(themes)
    .filter((name) => name !== 'createTheme')
    .map((name) => capitalCase(name))

  /**
   * Whether controls for changing language, theme, line wrapping etc should be shown
   */
  @property({ attribute: 'no-controls', type: Boolean })
  noControls: boolean = false

  /**
   * The name of the node property to send patch operations for when code changes
   *
   * Defaults to `text` (the name of the property of `CodeChunk` and `CodeExpression`).
   * Needed for higher-level components that have more than one editor to ensure changes
   * are emitted as patch operations for the correct property.
   */
  @property({ attribute: 'property-name' })
  propertyName: string = 'text'

  /**
   * The element containing the code content
   */
  private codeElem?: HTMLPreElement

  /**
   * A mutation observer to watch for changes in the code content
   */
  private codeObserver?: MutationObserver

  /**
   * A boolean flag to avoid `stencila-code-content-change` events being
   * fired when the editor content is being updated from the `codeElem`
   */
  private codeUpdating: boolean = false

  /**
   * The CodeMirror editor
   */
  private editorView?: EditorView

  /**
   * The CodeMirror language configuration
   */
  private languageConfig = new Compartment()

  /**
   * The CodeMirror `EditorView.editable` configuration
   */
  private editableConfig = new Compartment()

  /**
   * The CodeMirror `EditorState.readonly` configuration
   */
  private readOnlyConfig = new Compartment()

  /**
   * A CodeMirror compartment for dynamically configuring line wrapping
   */
  private lineWrapppingConfig = new Compartment()

  /**
   * The CodeMirror theme configuration
   */
  private themeConfig = new Compartment()

  /**
   * Extensions setup for CodeMirror
   *
   * Initially, based on https://github.com/codemirror/basic-setup/blob/main/src/codemirror.ts which
   * says..
   *
   * "This extension does not allow customization. The idea is that, once you decide you
   * want to configure your editor more precisely, you take this package's source
   * (which is just a bunch of imports and an array literal), copy it into your own code,
   * and adjust it as desired."
   *
   * Runtime configurable extensions added (e.g. `languageConfig`) need to be added
   * here.
   */
  private async editorExtensions() {
    let languageDesc: LanguageDescription | null
    if (this.language !== undefined) {
      languageDesc =
        this.matchLanguage(this.language) ?? this.fallbackLanguage()
    } else if (this.filename !== undefined) {
      languageDesc =
        this.matchLanguage(this.filename) ?? this.fallbackLanguage()
      if (languageDesc !== null) {
        // Set language so it appears in the select box
        this.language = languageDesc.name.toLowerCase()
      }
    } else {
      languageDesc = this.fallbackLanguage()
    }

    const languageSupport = await languageDesc.load()

    return [
      // Fixed extensions based off `basic-setup`
      highlightSpecialChars(),
      history(),
      drawSelection(),
      dropCursor(),
      EditorState.languageData.of(() => [{ autocomplete: completeAnyWord }]),
      EditorState.allowMultipleSelections.of(true),
      indentOnInput(),
      syntaxHighlighting(defaultHighlightStyle, { fallback: true }),
      bracketMatching(),
      closeBrackets(),
      autocompletion(),
      rectangularSelection(),
      crosshairCursor(),
      // Do not include as quite visually noisy
      // highlightActiveLine(),
      highlightSelectionMatches(),
      keymap.of([
        ...closeBracketsKeymap,
        ...searchKeymap,
        ...historyKeymap,
        ...foldKeymap,
        ...completionKeymap,
        ...lintKeymap,
        ...(this.singleLine ? [] : [indentWithTab]),
        {
          key: 'Ctrl-Space',
          run: startCompletion,
        },
        {
          key: 'Ctrl-Enter',
          run: () => {
            this.emit('stencila-ctrl-enter')
            return true
          },
        },
        ...defaultKeymap,
      ]),

      // Extension to emit an event when code changes (if not in read-only mode)
      EditorView.updateListener.of((update) => {
        if (!this.readOnly && update.docChanged && !this.codeUpdating) {
          this.emitOperations(...updateToOps(update, [this.propertyName]))
        }
      }),

      // Extensions based on properties but not change-able
      ...(this.singleLine
        ? [
            EditorState.transactionFilter.of((transaction) =>
              transaction.newDoc.lines > 1 ? [] : transaction
            ),
          ]
        : [lineNumbers(), highlightActiveLineGutter(), foldGutter()]),

      // Change-able extensions
      this.languageConfig.of(languageSupport),
      this.editableConfig.of(EditorView.editable.of(!this.readOnly)),
      this.readOnlyConfig.of(EditorState.readOnly.of(this.readOnly)),
      this.lineWrapppingConfig.of(EditorView.lineWrapping),
      this.themeConfig.of(this.getThemeExtension(this.theme)),
    ]
  }

  /**
   * List of `LanguageDescription`s for available languages
   *
   * This prepends the set of language descriptions from https://github.com/codemirror/language-data/blob/main/src/language-data.ts
   * with descriptions for:
   *
   * - languages where we want to use a different, third-party package for language support
   *   (i.e. we want to override what is in `@codemirror/language-data`)
   *
   * - languages that are not listed in `@codemirror/language-data`
   */
  private languageDescriptions = [
    // TODO: Develop custom grammar for Calc
    LanguageDescription.of({
      name: 'Calc',
      extensions: ['calc'],
      async load() {
        return import('@codemirror/lang-python').then((m) => m.python())
      },
    }),
    // TODO: Develop custom grammar for PrQL
    LanguageDescription.of({
      name: 'PrQL',
      extensions: ['prql'],
      async load() {
        return import('@codemirror/lang-sql').then((m) => m.sql())
      },
    }),
    // Use newer, third-party CodeMirror v6 grammar for R
    LanguageDescription.of({
      name: 'R',
      extensions: ['r'],
      async load() {
        return import('codemirror-lang-r').then((m) => m.r())
      },
    }),
    // Use custom grammar for Tailwind
    LanguageDescription.of({
      name: 'Tailwind',
      extensions: ['tw'],
      async load() {
        return import('codemirror-lang-tailwind').then((m) => m.tailwind())
      },
    }),
    ...languages,
  ]

  /**
   * Match a language name to a `LanguageDescription`
   *
   * This function will search through `languageDescriptions` for a match (using
   * CodeMirrors function for that). However, it has a shortcut which, for Markdown recursively
   * calls itself so that embedded code blocks also get highlighting.
   */
  private matchLanguage(name: string): LanguageDescription | null {
    const lower = name.toLowerCase()
    if (lower == 'md' || lower == 'markdown') {
      const self = this
      return LanguageDescription.of({
        name: 'Markdown',
        extensions: ['md'],
        async load() {
          return import('@codemirror/lang-markdown').then((m) =>
            m.markdown({
              codeLanguages: (info) => self.matchLanguage(info),
            })
          )
        },
      })
    }

    return (
      this.languageDescriptions.find((desc) =>
        desc.extensions.includes(name)
      ) ??
      LanguageDescription.matchLanguageName(this.languageDescriptions, name) ??
      LanguageDescription.matchFilename(this.languageDescriptions, name)
    )
  }

  /**
   * Get a fallback `LanguageDescription` when `matchLanguage` fails
   *
   * This is necessary because `this.languageConfig` always need to have SOME language support
   * in order to be able to change that language support later.
   *
   * Currently uses LaTeX (since that seems closest to plain text) but a more neutral
   * plain text grammar could be used in the future.
   */
  private fallbackLanguage(): LanguageDescription {
    return LanguageDescription.of({
      name: 'Default',
      async load() {
        return import('@codemirror/legacy-modes/mode/stex').then(
          (m) =>
            new LanguageSupport(
              // @ts-ignore
              StreamLanguage.define(m.stex)
            )
        )
      },
    })
  }

  /**
   * Get a CodeMirror `LanguageSupport` for a language
   */
  private getLanguageSupport(language?: string): Promise<LanguageSupport> {
    const languageDesc =
      language !== undefined
        ? this.matchLanguage(language) ?? this.fallbackLanguage()
        : this.fallbackLanguage()
    return languageDesc.load()
  }

  /**
   * Get a CodeMirror theme `Extension` by name
   *
   * If no theme name is provided, or if it does not exists,
   * returns a default theme.
   */
  private getThemeExtension(title?: string): Extension {
    const name = title ? camelCase(title) : 'ayuLight'
    const theme = themes[name]
    return theme ? theme : themes.ayuLight
  }

  /**
   * Get the current content of the editor
   */
  public getCode(): string {
    return this.editorView?.state.doc.toString() ?? ''
  }

  /**
   * Dispatch a CodeMirror `StateEffect` to the editor
   */
  private dispatchEffect(effect: StateEffect<unknown>) {
    const docState = this.editorView?.state

    const transaction =
      docState?.update({
        effects: [effect],
      }) ?? {}

    this.editorView?.dispatch(transaction)
  }

  /**
   * On a change in the `code` slot (including initial render) update the editor
   * and create a `MutationObserver` to update its content when the code changes.
   */
  private async onCodeSlotChange(event: Event) {
    // Get the `code` element and its content
    this.codeElem = (event.target as HTMLSlotElement).assignedElements({
      flatten: true,
    })[0] as HTMLPreElement
    const content = this.codeElem.textContent ?? ''

    // If the editor view does not yet exist then create it, otherwise create a transaction
    // to update its content
    if (this.editorView == undefined) {
      this.editorView = new EditorView({
        doc: content,
        extensions: [await this.editorExtensions()],
        parent: this.renderRoot.querySelector('#codemirror')!,
      })
    } else {
      const docState = this.editorView.state
      const transaction =
        docState?.update({
          changes: {
            from: 0,
            to: docState.doc.length,
            insert: content,
          },
          selection: this.editorView.state.selection,
        }) ?? {}

      this.editorView.dispatch(transaction)
    }

    // Create an observer
    this.codeObserver = new MutationObserver((mutations) =>
      this.onCodeMutation(mutations)
    )
    this.codeObserver.observe(this.codeElem, {
      subtree: true,
      characterData: true,
    })
  }

  /**
   * Handle a mutation event on the `code` slot
   *
   * Gets the new code and updates the content of the editor.
   */
  onCodeMutation(mutationList: MutationRecord[]) {
    const content = this.codeElem?.textContent ?? ''

    this.codeUpdating = true
    const docState = this.editorView!.state
    const transaction =
      docState?.update({
        changes: {
          from: 0,
          to: docState.doc.length,
          insert: content,
        },
      }) ?? {}
    this.codeUpdating = false

    this.editorView!.dispatch(transaction)
  }

  /**
   * Perform reactive updates to properties
   *
   * This allows for changes made both within this component (via dropdowns)
   * and outside (via patches on attributes) to be reflected.
   */
  protected async update(changedProperties: Map<string, any>) {
    super.update(changedProperties)

    if (changedProperties.has('language')) {
      const languageSupport = await this.getLanguageSupport(this.language)
      const effect = this.languageConfig.reconfigure(languageSupport)
      this.dispatchEffect(effect)
    }

    if (changedProperties.has('readOnly')) {
      this.dispatchEffect(
        this.editableConfig.reconfigure(EditorView.editable.of(!this.readOnly))
      )
      this.dispatchEffect(
        this.readOnlyConfig.reconfigure(EditorState.readOnly.of(this.readOnly))
      )
    }

    if (changedProperties.has('lineWrapping')) {
      this.dispatchEffect(
        this.lineWrapppingConfig.reconfigure(
          this.lineWrapping ? EditorView.lineWrapping : []
        )
      )
    }

    if (changedProperties.has('theme')) {
      const theme = this.getThemeExtension(this.theme)
      const effect = this.themeConfig.reconfigure(theme)
      this.dispatchEffect(effect)

      if (this.theme) {
        window.localStorage.setItem('StencilaCodeEditor.theme', this.theme)
      }
    }
  }

  render() {
    return html`<div
      class="${tw(
        css`
          @apply w-full;

          ${varLocal(
            'border-style',
            'border-width',
            'border-color',
            'border-radius',
            'text-font',
            'text-size',
            'text-color'
          )}

          ${varApply(
            'border-style',
            'border-width',
            'border-color',
            'border-radius'
          )}

          display: ${this.singleLine ? 'inline-block' : 'block'};

          [part='header'] sl-select::part(control) {
            ${varApply(
              'border-style',
              'border-width',
              'border-color',
              'border-radius'
            )}
          }

          [part='header'] sl-select::part(control),
          [part='header'] sl-menu-item::part(display-label),
          [part='header'] sl-menu-item::part(prefix),
          [part='header'] sl-menu-item::part(label) {
            ${varApply('text-font', 'text-size', 'text-color')}
          }

          /* Removed dotted outline when editor is focussed */
          .cm-editor.cm-focused {
            outline: none;
          }

          /* Improve appearance of autocomplete prompt */
          .cm-tooltip.cm-tooltip-autocomplete > ul > li {
            font-size: 90%;
            padding-bottom: 2px;
            padding-top: 2px;
          }
          .cm-completionIcon {
            min-width: 1.375em;
            min-width: 2ch;
            margin-right: 1ch;
          }
        `
      )}"
    >
      ${!this.noControls
        ? html`<div
            part="header"
            class="${tw`flex flex-row items-center justify-between`}"
          >
            <div class="end">
              ${this.renderWordWrappingSwitch()}
              ${this.renderLanguageDropdown()} ${this.renderThemeDropdown()}
            </div>
          </div>`
        : ''}

      <slot
        part="code"
        name="code"
        @slotchange=${this.onCodeSlotChange}
        class="${tw`hidden`}"
      ></slot>
      <div part="code" id="codemirror"></div>
    </div>`
  }

  private renderWordWrappingSwitch() {
    return html`<sl-switch
      ?checked=${this.lineWrapping}
      @sl-change=${(event: Event) =>
        (this.lineWrapping = (event.target as SlSwitch).checked)}
    >
      Line wrapping
    </sl-switch>`
  }

  private renderLanguageDropdown() {
    if (this.languages.length === 0) {
      return html``
    }

    return html` <sl-select
      size="small"
      value=${this.language?.toLowerCase() ?? 'other'}
      @sl-change=${(event: Event) =>
        (this.language = (event.target as HTMLSelectElement).value)}
    >
      <stencila-icon
        slot="prefix"
        name="code"
        label="Programming language"
      ></stencila-icon>
      ${this.languages.map(
        (language) =>
          html`<sl-menu-item value="${language.toLowerCase()}">
            ${language}
          </sl-menu-item>`
      )}
    </sl-select>`
  }

  private renderThemeDropdown() {
    if (this.themes.length === 0) {
      return html``
    }

    return html` <sl-select
      size="small"
      value=${camelCase(this.theme)}
      @sl-change=${(event: Event) =>
        (this.theme = (event.target as HTMLSelectElement).value)}
    >
      <stencila-icon slot="prefix" name="palette" label="Theme"></stencila-icon>
      ${this.themes.map(
        (theme) =>
          html`<sl-menu-item value="${camelCase(theme)}">
            ${theme}
          </sl-menu-item>`
      )}
    </sl-select>`
  }
}
