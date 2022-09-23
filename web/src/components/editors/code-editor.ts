import { capitalCase, camelCase } from 'change-case'
import { html } from 'lit'
import { customElement, property } from 'lit/decorators'
import { css } from 'twind/css'

import {
  autocompletion,
  closeBrackets,
  closeBracketsKeymap,
  completionKeymap,
} from '@codemirror/autocomplete'
import { defaultKeymap, history, historyKeymap } from '@codemirror/commands'
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
  highlightActiveLine,
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
  language: string = 'other'

  /**
   * A list of languages supported by this editor
   *
   * This list is presented in a dropdown language selector.
   * If the list is empty, no selector will be provided.
   */
  @property({ type: Array })
  private languagesSupported = [
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
   * The editor theme
   */
  @property({ reflect: true })
  theme: string = 'Tomorrow'

  /**
   * A list of themes supported by this editor
   *
   * This list is presented in a dropdown theme selector.
   * If the list is empty, no selector will be provided.
   */
  @property({ type: Array })
  private themes = Object.keys(themes).map((name) => capitalCase(name))

  /**
   * The CodeMirror editor
   */
  private editorView?: EditorView

  /**
   * The CodeMirror language configuration
   */
  private languageConfig = new Compartment()

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
    return [
      // Fixed extensions based off `basic-setup`
      lineNumbers(),
      highlightActiveLineGutter(),
      highlightSpecialChars(),
      history(),
      foldGutter(),
      drawSelection(),
      dropCursor(),
      EditorState.allowMultipleSelections.of(true),
      indentOnInput(),
      syntaxHighlighting(defaultHighlightStyle, { fallback: true }),
      bracketMatching(),
      closeBrackets(),
      autocompletion(),
      rectangularSelection(),
      crosshairCursor(),
      highlightActiveLine(),
      highlightSelectionMatches(),
      keymap.of([
        ...closeBracketsKeymap,
        ...defaultKeymap,
        ...searchKeymap,
        ...historyKeymap,
        ...foldKeymap,
        ...completionKeymap,
        ...lintKeymap,
      ]),

      // Change-able extensions
      this.languageConfig.of(await this.getLanguageSupport(this.language)),
      this.themeConfig.of(this.getTheme(this.theme)),
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
    if (name == 'md' || name == 'markdown') {
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

    return LanguageDescription.matchLanguageName(
      this.languageDescriptions,
      name
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
  private getLanguageSupport(language: string): Promise<LanguageSupport> {
    const languageDesc = this.matchLanguage(language) ?? this.fallbackLanguage()
    return languageDesc.load()
  }

  /**
   * On a change in the language selector update the property, dispatch
   * an event and dispatch an effect to the editor
   */
  private async onLanguageChange(event: Event) {
    this.language = (event.target as HTMLSelectElement).value

    // TODO: Dispatch event

    const languageSupport = await this.getLanguageSupport(this.language)
    const effect = this.languageConfig.reconfigure(languageSupport)
    this.dispatchEffect(effect)
  }

  /**
   * Get a CodeMirror theme `Extension`
   */
  private getTheme(title: string): Extension {
    const name = camelCase(title)
    return themes[name]
  }

  /**
   * On a change in the theme selector update the property and dispatch
   * an effect to the editor
   */
  private async onThemeChange(event: Event) {
    this.theme = (event.target as HTMLSelectElement).value

    const theme = this.getTheme(this.theme)
    const effect = this.themeConfig.reconfigure(theme)
    this.dispatchEffect(effect)
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
   */
  private async onCodeSlotChange(event: Event) {
    // Get the text content from the slot
    const childNodes = (event.target as HTMLSlotElement).assignedNodes({
      flatten: true,
    })
    const content = childNodes.map((node) => node.textContent ?? '').join('')

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
  }

  render() {
    return html`<div
      class="${tw(
        css`
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

          [part='language'] sl-select::part(control) {
            ${varApply(
              'border-style',
              'border-width',
              'border-color',
              'border-radius'
            )}
          }

          [part='language'] sl-select::part(control),
          [part='language'] sl-menu-item::part(display-label),
          [part='language'] sl-menu-item::part(prefix),
          [part='language'] sl-menu-item::part(label) {
            ${varApply('text-font', 'text-size', 'text-color')}
          }
        `
      )}"
    >
      <div class="${tw`flex flex-row items-center justify-between`}">
        <div class="start">
          <slot name="info"></slot>
        </div>
        <div part="language" class="${tw`w-40`}">
          <sl-select size="small" @sl-change=${this.onLanguageChange}>
            <stencila-icon
              slot="prefix"
              name="code"
              label="Programming language"
            ></stencila-icon>
            ${this.languagesSupported.map((lang) => {
              return html`<sl-menu-item value="${lang}">
                <stencila-icon
                  slot="prefix"
                  name="lightning-fill"
                  label="Executable"
                  class="${tw`text-yellow-500`}"
                ></stencila-icon>
                ${lang}
              </sl-menu-item>`
            })}
          </sl-select>
        </div>
        <div part="theme" class="${tw`w-40`}">
          <sl-select size="small" @sl-change=${this.onThemeChange}>
            <stencila-icon
              slot="prefix"
              name="palette"
              label="Theme"
            ></stencila-icon>
            ${this.themes.map(
              (theme) =>
                html`<sl-menu-item value="${theme}"> ${theme} </sl-menu-item>`
            )}
          </sl-select>
        </div>
      </div>

      <slot
        part="code"
        name="code"
        @slotchange=${this.onCodeSlotChange}
        class="${tw`hidden`}"
      ></slot>
      <div id="codemirror"></div>

      <slot name="messages"></slot>
    </div>`
  }
}
