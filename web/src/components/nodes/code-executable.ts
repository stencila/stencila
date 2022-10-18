import { capitalCase } from 'change-case'
import { html } from 'lit'
import { customElement, property, state } from 'lit/decorators'

import '@shoelace-style/shoelace/dist/components/dropdown/dropdown'
import '@shoelace-style/shoelace/dist/components/menu/menu'

import { currentMode, Mode } from '../../mode'
import '../base/icon-button'
import { twSheet } from '../utils/css'
import StencilaElement from '../utils/element'
import Executable from './executable'

/**
 * A base component to represent the `CodeExecutable` node type
 */
export default class StencilaCodeExecutable extends Executable {
  /**
   * The `CodeExecutable.text` property
   */
  @property({ reflect: true })
  text: string = ''

  /**
   * The `CodeExecutable.programmingLanguage` property
   */
  @property({ attribute: 'programming-language', reflect: true })
  programmingLanguage: string = ''

  /**
   * The `CodeExecutable.guessLanguage` property
   *
   * Because an _optional_ boolean property in Rust and because of how patching works
   * must be a string, not a HTML boolean property.
   */
  @property({ attribute: 'guess-language', reflect: true })
  guessLanguage: string = 'true'

  /**
   * Is the code of the node (the `text` property) visible?
   */
  @state()
  protected _isCodeVisible: boolean

  private onCodeVisibilityChanged(event: CustomEvent) {
    this._isCodeVisible = event.detail.isVisible
  }

  protected onCodeVisibilityClicked(event: PointerEvent) {
    if (event.shiftKey) {
      this.emit('stencila-code-visibility-change', {
        isVisible: !this._isCodeVisible,
      })
    } else {
      this._isCodeVisible = !this._isCodeVisible
    }
  }

  /**
   * Is the node editable (i.e. code and `programmingLanguage` can be changed) in the current mode
   */
  protected isEditable(): boolean {
    const mode = currentMode()
    return mode >= Mode.Alter && mode != Mode.Edit
  }

  /**
   * Does the node have any outputs?
   */
  @state()
  protected _hasOutputs: boolean

  protected async onOutputsSlotChange(event: Event) {
    const slotted = (event.target as HTMLSlotElement).assignedNodes()[0]
    this._hasOutputs = slotted.childNodes.length > 0
  }

  connectedCallback() {
    super.connectedCallback()

    this.addEventListener(
      'stencila-code-content-change',
      (event: CustomEvent) => {
        return this.emit('stencila-document-patch', {
          target: this.id,
          ops: event.detail.ops,
        })
      }
    )

    window.addEventListener(
      'stencila-code-visibility-change',
      this.onCodeVisibilityChanged.bind(this)
    )
  }

  disconnectedCallback() {
    super.disconnectedCallback()

    window.removeEventListener(
      'stencila-code-visibility-change',
      this.onCodeVisibilityChanged.bind(this)
    )
  }
}

const { tw, sheet } = twSheet()

/**
 * A component for changing the `programmingLanguage` and `guessLanguage` properties of a `CodeExecutable` node
 *
 * Uses a static list of languages currently supported by Stencila but indicates which
 * are not supported in the current execution environment. If the `programmingLanguage`
 * is not in the list it will be added.
 *
 * If `guessLanguage == true` then the `programmingLanguage` may be patched when the parent node
 * (e.g. a `CodeChunk` is compiled). If the user explicitly selects a language then `guessLanguage`
 * should be set to `false`.
 */
@customElement('stencila-executable-language')
export class StencilaExecutableLanguage extends StencilaElement {
  static styles = sheet.target

  static languages = [
    ['bash', '', 'Bash'],
    ['calc', tw`text-green-800`, 'Calc'],
    ['javascript', '', 'JavaScript', 'js'],
    ['json', '', 'JSON'],
    ['json5', '', 'JSON5'],
    ['prql', '', 'PRQL'],
    ['python', '', 'Python', 'py', 'python3'],
    ['r', '', 'R'],
    ['sql', tw`text-blue-600`, 'SQL'],
    ['tailwind', tw`text-blue-600`, 'Tailwind'],
    ['unknown', tw`text-gray-300`, 'Unknown'],
  ]

  /**
   * The `CodeExecutable.programmingLanguage` property
   */
  @property({ attribute: 'programming-language', reflect: true })
  programmingLanguage: string

  /**
   * The `CodeExecutable.guessLanguage` property
   */
  @property({ type: Boolean, attribute: 'guess-language', reflect: true })
  guessLanguage: boolean

  /**
   * Languages to include in the the list of selectable languages
   */
  @property({ type: Array, attribute: 'exclude' })
  include: string[] = StencilaExecutableLanguage.languages.map(
    ([value]) => value
  )

  /**
   * Languages to exclude from the list of selectable languages
   */
  @property({ type: Array, attribute: 'exclude' })
  exclude: string[] = []

  /**
   * The color palette for the icon
   */
  @property()
  color: string = 'gray'

  render() {
    const languages = StencilaExecutableLanguage.languages.filter(
      ([lang]) => this.include.includes(lang) && !this.exclude.includes(lang)
    )

    const language = this.programmingLanguage.toLowerCase()

    let icon = 'code'
    for (const [value, _cls, _title, ...aliases] of languages) {
      if (language === value || aliases.includes(language)) {
        icon = value
        break
      }
    }

    return html`
      <sl-dropdown
        class=${tw`flex items-center`}
        ?disabled=${currentMode() < Mode.Alter}
      >
        <stencila-icon-button
          slot="trigger"
          name=${icon}
          color=${this.color}
          ?disabled=${currentMode() < Mode.Alter}
        >
        </stencila-icon-button>

        <sl-menu
          @sl-select=${(event: CustomEvent) => {
            const value = event.detail.item.value
            if (value == 'guess') {
              // Toggle `guessLanguage`
              this.guessLanguage = !this.guessLanguage
              this.emitReplaceOperations('guessLanguage')
            } else {
              // Change the `programmingLanguage` and set `guessLanguage` to false if necessary
              const changedProperties: string[] = []
              if (this.programmingLanguage !== value) {
                this.programmingLanguage = value
                changedProperties.push('programmingLanguage')
                if (this.guessLanguage) {
                  this.guessLanguage = false
                  changedProperties.push('guessLanguage')
                }
                this.emitReplaceOperations(...changedProperties)
              }
            }
          }}
        >
          ${languages.map(
            ([value, cls, title, ...aliases]) =>
              html` <sl-menu-item
                value=${value}
                ?checked=${language == value || aliases.includes(language)}
              >
                <stencila-icon
                  slot="prefix"
                  name="${value}-color"
                  class=${cls}
                ></stencila-icon>
                ${title}
              </sl-menu-item>`
          )}
          ${language?.trim().length > 0 &&
          languages.filter(
            ([value, _cls, _title, ...aliases]) =>
              language === value || aliases.includes(language)
          ).length === 0
            ? html` <sl-menu-item value=${language} checked>
                <stencila-icon
                  slot="prefix"
                  name="code"
                  class=${tw`text-gray-400`}
                ></stencila-icon>
                ${capitalCase(this.programmingLanguage)}
              </sl-menu-item>`
            : ''}

          <sl-divider class=${tw`border(t gray-100)`}></sl-divider>

          <sl-menu-item value="guess" ?checked=${this.guessLanguage}>
            <stencila-icon
              class=${tw`text-gray-500`}
              slot="prefix"
              name="magic"
            ></stencila-icon>
            Guess
          </sl-menu-item>
        </sl-menu>
      </sl-dropdown>
    `
  }
}
