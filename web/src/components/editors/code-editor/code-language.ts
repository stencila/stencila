import '@shoelace-style/shoelace/dist/components/dropdown/dropdown'
import '@shoelace-style/shoelace/dist/components/menu/menu'
import { capitalCase } from 'change-case'
import { css, html } from 'lit'
import { customElement, property } from 'lit/decorators'
import { twSheet } from '../../utils/css'
import StencilaElement from '../../utils/element'
import '../../base/icon-button'
import StencilaCodeEditor from './code-editor'
import { LanguageDescription } from '@codemirror/language'

const { tw, sheet } = twSheet()

/**
 * A component for changing the `programmingLanguage` (and `guessLanguage` properties)
 * of code nodes.
 *
 * Uses a static list of languages currently supported by Stencila but indicates which
 * are not supported in the current execution environment. If the `programmingLanguage`
 * is not in the list it will be added.
 *
 * If `guessLanguage == true` then the `programmingLanguage` may be patched when the parent node
 * (e.g. a `CodeChunk` is compiled). If the user explicitly selects a language then `guessLanguage`
 * should be set to `false`.
 */
@customElement('stencila-code-language')
export class StencilaCodeLanguage extends StencilaElement {
  static styles = [
    sheet.target,
    css`
      sl-menu-item::part(label) {
        line-height: 1;
      }
    `,
  ]

  /**
   * The programming language
   *
   * Used to reflect/set the `CodeExecutable.programmingLanguage`
   * and `CodeStatic.programmingLanguage` properties.
   */
  @property({ attribute: 'programming-language', reflect: true })
  programmingLanguage: string

  /**
   * Whether the language should be guessed
   *
   * Used to reflect/set the `CodeExecutable.guessLanguage` property.
   */
  @property({ type: Boolean, attribute: 'guess-language', reflect: true })
  guessLanguage: boolean

  /**
   * Whether the language is able to be guessed
   */
  @property({ type: Boolean, attribute: 'is-guessable', reflect: true })
  isGuessable: boolean = false

  /**
   * Whether only executable languages should be included
   */
  @property({ type: Boolean, attribute: 'executable-only', reflect: true })
  executableOnly: boolean = false

  /**
   * Languages that are executable
   */
  static executables: string[] = [
    'bash',
    'calc',
    'javascript',
    'http',
    'postgrest',
    'prql',
    'python',
    'r',
    'sql',
    'tailwind',
  ]

  /**
   * Languages to include in the the list of selectable languages
   *
   * Rather than show all available CodeMirror languages, the default
   * is to only include the most well known and used languages. Override
   * this property to include more.
   */
  @property({ type: Array, attribute: 'include' })
  include: string[] = [
    'bash',
    'c',
    'c#',
    'c++',
    'calc',
    'clojure',
    'crystal',
    'css',
    'd',
    'diff',
    'dockerfile',
    'erlang',
    'f#',
    'fortran',
    'go',
    'haskell',
    'html',
    'http',
    'java',
    'javascript',
    'jinja2',
    'json',
    'json5',
    'jsx',
    'julia',
    'kotlin',
    'lua',
    'objective-c',
    'ocaml',
    'perl',
    'php',
    'pgrest',
    'prql',
    'python',
    'r',
    'ruby',
    'rust',
    'scss',
    'shell',
    'sql',
    'swift',
    'tailwind',
    'toml',
    'tsx',
    'typescript',
    'xml',
    'yaml',
  ]

  /**
   * Languages to exclude from the list of selectable languages
   */
  @property({ type: Array, attribute: 'exclude' })
  exclude: string[] = []

  /**
   * The color palette for the trigger icon
   */
  @property()
  color = 'blue'

  /**
   * Whether the menu is disabled
   */
  @property({ type: Boolean })
  disabled = false

  /**
   * Override to ensure that the property is changed on this element
   * AND on the parent `Entity` that contains this menu
   */
  protected changeProperties(properties: [string, unknown][]) {
    const parent = StencilaElement.closestElement(this.parentElement!, '[id]')!
    for (const [property, value] of properties) {
      parent[property] = value
    }

    return super.changeProperties(properties)
  }

  render() {
    // Currently selected language
    const language = this.programmingLanguage.trim().toLowerCase()

    // List of language descriptions filtered by `includes` and `excludes`
    const languages = StencilaCodeEditor.languageDescriptions
      .filter((desc) => {
        const name = desc.name.toLowerCase()
        const included = this.executableOnly
          ? StencilaCodeLanguage.executables
          : this.include
        return included.includes(name) && !this.exclude.includes(name)
      })
      .sort((a, b) => a.name.localeCompare(b.name))

    // Icon for selected language for the trigger button
    const desc =
      languages.find((desc) => desc.extensions.includes(language)) ??
      LanguageDescription.matchLanguageName(languages, language)
    const icon = desc?.name.toLowerCase() ?? 'code'

    const select = (event: CustomEvent) => {
      const value = event.detail.item.value
      if (value == 'guess') {
        // Toggle `guessLanguage`
        const guessLanguage = !this.guessLanguage
        this.changeProperty('guessLanguage', guessLanguage)
      } else {
        // Change the `programmingLanguage` and set `guessLanguage` to false if necessary
        const changedProperties: [string, unknown][] = []
        if (this.programmingLanguage !== value) {
          changedProperties.push(['programmingLanguage', value])
          if (this.guessLanguage) {
            changedProperties.push(['guessLanguage', false])
          }
          this.changeProperties(changedProperties)
        }
      }
    }

    // Whether the selected language is in the list of languages
    let inLanguages = false

    return html`
      <sl-dropdown class=${tw`flex items-center`} ?disabled=${this.disabled}>
        <stencila-icon-button
          slot="trigger"
          name=${icon}
          fallback="code"
          color=${this.color}
          ?disabled=${this.disabled}
        >
        </stencila-icon-button>

        <sl-menu @sl-select=${select}>
          ${
            // Menu item for each language
            languages.map((desc) => {
              const value = desc.name.toLowerCase()
              const checked =
                language == value ||
                desc.extensions.includes(language) ||
                desc.alias.includes(language)

              if (!inLanguages && checked) {
                inLanguages = true
              }

              return html` <sl-menu-item value=${value} ?checked=${checked}>
                <stencila-icon
                  slot="prefix"
                  name="${value}-color"
                  fallback="code"
                ></stencila-icon>
                <span class=${tw`text-sm`}>${desc.name}</span>
              </sl-menu-item>`
            })
          }
          ${
            // Menu item if not in `languages`
            language.length > 0 && !inLanguages
              ? html` <sl-menu-item value=${language} checked>
                  <stencila-icon slot="prefix" name="code"></stencila-icon>
                  <span class=${tw`text-sm`}
                    >${capitalCase(this.programmingLanguage)}</span
                  >
                </sl-menu-item>`
              : ''
          }

          <sl-divider class=${tw`border(t gray-100)`}></sl-divider>

          ${
            // Menu item to turn on/off guessing
            this.isGuessable
              ? html`<sl-menu-item value="guess" ?checked=${this.guessLanguage}>
                  <stencila-icon
                    class=${tw`text-gray-500`}
                    slot="prefix"
                    name="magic"
                  ></stencila-icon>
                  <span class=${tw`text-sm`}>Guess</span>
                </sl-menu-item>`
              : ''
          }
        </sl-menu>
      </sl-dropdown>
    `
  }
}
