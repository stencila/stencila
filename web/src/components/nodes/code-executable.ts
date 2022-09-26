import { sentenceCase } from 'change-case'
import { html } from 'lit'
import { property, state } from 'lit/decorators'
import { apply as twApply, css, TW } from 'twind/css'

import { currentMode, Mode } from '../../mode'
import Executable from './executable'

/**
 * A base component to represent the `CodeExecutable` node type
 */
export default class StencilaCodeExecutable extends Executable {
  @property({
    attribute: 'programming-language',
  })
  programmingLanguage: string

  @state()
  protected isCodeVisible: boolean

  private onCodeVisibilityChanged(event: CustomEvent) {
    this.isCodeVisible = event.detail.isVisible
  }

  protected onCodeVisibilityClicked(event: PointerEvent) {
    if (event.shiftKey) {
      this.emit('stencila-code-visibility-change', {
        isVisible: !this.isCodeVisible,
      })
    } else {
      this.isCodeVisible = !this.isCodeVisible
    }
  }

  /**
   * Is the node editable (i.e. code and `programmingLanguage` can be changed) in the current mode
   */
  protected isEditable(): boolean {
    const mode = currentMode()
    return mode >= Mode.Alter && mode != Mode.Edit
  }

  connectedCallback() {
    super.connectedCallback()

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

  protected renderLanguageSelector(tw: TW) {
    const languages = window.stencilaConfig.executableLanguages ?? []

    if (languages.length === 0) {
      return html`<span class="language">${this.programmingLanguage}</span>`
    }

    return html`<span
      class=${tw(css`
        ${twApply(`inline-block`)}
        sl-select {
          display: inline;
          width: 13ch;
        }
        sl-select::part(form-control) {
          display: inline-block;
        }
        sl-select::part(control) {
          background-color: transparent;
          border: none;
        }
        sl-select.code-invisible::part(control) {
          ${twApply('cursor-pointer')}
        }
        sl-select::part(icon) {
          display: ${this.isCodeVisible && this.isEditable()
            ? 'inherit'
            : 'none'};
        }
        sl-select::part(menu) {
          overflow: hidden;
        }
        sl-menu-item::part(label) {
          ${twApply('text-sm')}
        }
      `)}
    >
      <sl-tooltip>
        <span slot="content"
          >${this.isCodeVisible ? 'Hide' : 'Show'} code<br />Shift click to
          ${this.isCodeVisible ? 'hide' : 'show'} for all code elements</span
        >
        <stencila-icon
          name="${this.isCodeVisible ? 'eye' : 'eye-slash'}"
          @click=${this.onCodeVisibilityClicked}
          class=${tw`cursor-pointer`}
        ></stencila-icon>
      </sl-tooltip>
      ${!this.isCodeVisible
        ? html`<sl-select
            size="small"
            value=${this.programmingLanguage?.toLowerCase() ?? 'other'}
            disabled
            @click=${this.onCodeVisibilityClicked}
            class="code-${this.isCodeVisible ? 'visible' : 'invisible'}"
          >
            <sl-menu-item value=${this.programmingLanguage.toLowerCase()}>
              ${this.labelForLanguage(this.programmingLanguage)}
            </sl-menu-item>
          </sl-select>`
        : html`<sl-select
            size="small"
            value=${this.programmingLanguage?.toLowerCase() ?? 'other'}
            ?disabled=${!this.isEditable()}
            @sl-change=${(event: Event) =>
              (this.programmingLanguage = (
                event.target as HTMLSelectElement
              ).value)}
          >
            ${languages.map(
              (language) =>
                html`<sl-menu-item value="${language.toLowerCase()}">
                  ${this.labelForLanguage(language)}
                </sl-menu-item>`
            )}
          </sl-select>`}
    </span>`
  }

  protected labelForLanguage(language: string): string {
    switch (language.toLowerCase()) {
      case 'javascript':
        return 'JavaScript'
      case 'typescript':
        return 'TypeScript'
      case 'json':
      case 'sql':
        return language.toUpperCase()
      case 'prql':
        return 'PrQL'
      default:
        return sentenceCase(language)
    }
  }
}
