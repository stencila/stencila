import { html } from 'lit'
import { property, state } from 'lit/decorators'
import { TW } from 'twind'

import { isCodeWriteable } from '../../mode'
import Executable from './executable'

import '../base/icon-button'
import '../editors/code-editor/code-language'

/**
 * A base component to represent the `CodeExecutable` node type
 *
 * @slot text The `CodeExecutable.text` property
 */
export default class StencilaCodeExecutable extends Executable {
  /**
   * The `CodeExecutable.programmingLanguage` property
   */
  @property({ attribute: 'programming-language', reflect: true })
  programmingLanguage = ''

  /**
   * The `CodeExecutable.guessLanguage` property
   *
   * Because of how patching works, this property must must be a string, not a HTML boolean attribute.
   */
  @property({ attribute: 'guess-language', reflect: true })
  guessLanguage?: string

  /**
   * The `CodeExecutable.text` property
   *
   * Note that we use a convention of representing the `CodeExecutable.text` property
   * as `<slot name="text">`, rather than as an attribute `text="..."` for better
   * discover-ability.
   *
   * Also, using a slot is a more natural fit with using a code editor
   * on that content. We "relay" the `text` slot to the <stencila-code-editor>
   * using `<slot name="text" slot="code"></slot>` in components derived from this class.
   *
   * However, we also maintain this state so that derived components can use it
   * to update other state e.g. the `IfClause.isElse` property.
   */
  @property({ reflect: true })
  public text?: string

  /**
   * An observer to update `text` from the slot
   */
  private textObserver: MutationObserver

  /**
   * Handle a change, including on initial load, of the `text` slot
   */
  protected onTextSlotChange(event: Event) {
    const textElem = (event.target as HTMLSlotElement).assignedElements({
      flatten: true,
    })[0]

    this.text = textElem.textContent ?? ''

    this.textObserver = new MutationObserver(() => {
      this.text = textElem.textContent ?? ''
    })
    this.textObserver.observe(textElem, {
      subtree: true,
      characterData: true,
    })
  }

  /**
   * Is the code of the node (the `text` property) visible?
   */
  @state()
  protected isCodeVisible: boolean = true

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
   * Render a button to toggle code visibility
   */
  protected renderViewCodeButton(tw: TW) {
    return html` <stencila-icon-button
      name=${this.isCodeVisible ? 'eye-slash' : 'eye'}
      color="blue"
      adjust="ml-0.5"
      @click=${() => {
        this.isCodeVisible = !this.isCodeVisible
      }}
      @keydown=${(event: KeyboardEvent) => {
        if (
          event.key == 'Enter' ||
          (event.key == 'ArrowUp' && this.isCodeVisible) ||
          (event.key == 'ArrowDown' && !this.isCodeVisible)
        ) {
          event.preventDefault()
          this.isCodeVisible = !this.isCodeVisible
        }
      }}
    >
    </stencila-icon-button>`
  }

  /**
   * The element assigned to the `outputs` slot
   */
  public outputs?: HTMLElement

  /**
   * Does the node have any outputs?
   */
  @state()
  protected hasOutputs: boolean

  /**
   * An observer to update `hasOutputs`
   */
  private outputsObserver: MutationObserver

  protected onOutputsSlotChange(event: Event) {
    const outputs = (event.target as HTMLSlotElement).assignedElements({
      flatten: true,
    })[0] as HTMLElement | undefined

    this.outputs = outputs
    this.hasOutputs = outputs !== undefined && outputs.childElementCount > 0

    if (outputs) {
      this.outputsObserver = new MutationObserver(() => {
        this.hasOutputs = outputs.childElementCount > 0
      })
      this.outputsObserver.observe(outputs, {
        childList: true,
      })
    }
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

  protected renderLanguageMenu(tw: TW) {
    const readOnly = !isCodeWriteable()

    return html`<stencila-code-language
      class=${tw`ml-0.5 text(base blue-500)`}
      programming-language=${this.programmingLanguage}
      ?guess-language=${this.guessLanguage == 'true'}
      include='["bash", "calc", "javascript", "http", "postgrest", "prql", "python", "r", "sql", "tailwind"]'
      ?is-guessable=${true}
      ?disabled=${readOnly}
    ></stencila-code-language>`
  }
}
