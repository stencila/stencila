import '@shoelace-style/shoelace/dist/components/dropdown/dropdown'
import '@shoelace-style/shoelace/dist/components/tab-group/tab-group'
import '@shoelace-style/shoelace/dist/components/tab-panel/tab-panel'
import '@shoelace-style/shoelace/dist/components/tab/tab'
import { sentenceCase } from 'change-case'
import { css, html } from 'lit'
import { customElement, property } from 'lit/decorators'
import { TW } from 'twind'
import StencilaIconButton from '../base/icon-button'
import StencilaCodeEditor from '../editors/code-editor'
import { twSheet } from '../utils/css'
import copy from 'clipboard-copy'

import StencilaElement from '../utils/element'

/**
 * A base component to represent the `Entity` node type
 */
export default class StencilaEntity extends StencilaElement {
  /**
   * The id of the entity
   */
  @property()
  id: string

  renderEntityDownload(formats: string[], color: string, shade = 50) {
    return html`<stencila-entity-download
      color=${color}
      shade=${shade}
      formats=${JSON.stringify(formats)}
    ></stencila-entity-download>`
  }
}

const { tw, sheet } = twSheet()

/**
 * A component for providing an `Entity` node type in various formats
 * for download or copy/paste
 */
@customElement('stencila-entity-download')
export class StencilaEntityDownload extends StencilaElement {
  static styles = [
    sheet.target,
    css`
      sl-tab::part(base) {
        padding: 0.5em;
      }
      sl-tab-panel {
        --padding: 0;
      }
    `,
  ]

  /**
   * The color palette to use for this component
   */
  @property()
  color = 'gray'

  /**
   * The base color shade to use for this component
   */
  @property()
  shade = 50

  /**
   * The formats that are available for the entity
   */
  @property({ type: Array })
  formats: string[] = ['markdown', 'python', 'javascript', 'r', 'yaml', 'json']

  static formatDetails = {
    json: ['JSON'],
    markdown: ['MD'],
    python: ['Python'],
    javascript: ['JavaScript'],
    r: ['R'],
    yaml: ['YAML'],
  }

  render() {
    const load = async (format: string) => {
      // Get a reference to the code editor
      const panel = this.renderRoot.querySelector(
        `sl-tab-panel[name=${format}]`
      )
      if (!panel) {
        throw new Error(`No matching panel for ${format}`)
      }
      const editor = panel.querySelector(
        'stencila-code-editor'
      ) as StencilaCodeEditor

      // Clear the content to show it as loading
      editor.setCode('')

      // Make a request to dump the node in the format
      const nodeId = this.closestElement('[id]')?.id
      const content = await window.stencilaClient.dump(format, nodeId)

      // Replace the content of the panel's editor with the dump
      editor.setCode(content)

      // Ensure the copy-to-clipboard button is not checked
      const button = this.renderRoot.querySelector(
        'stencila-icon-button[name=clipboard]'
      ) as StencilaIconButton
      button.name = 'clipboard'
    }

    return html`<sl-dropdown
      distance="12"
      placement="bottom-end"
      @sl-show=${() => {
        let activeFormat = window.localStorage.getItem(
          'StencilaEntityDownload.format'
        )
        if (activeFormat == null || !this.formats.includes(activeFormat)) {
          activeFormat = this.formats[0]
        }

        const tabGroup = this.renderRoot.querySelector('sl-tab-group')
        tabGroup?.show(activeFormat)

        load(activeFormat)
      }}
    >
      <stencila-icon-button
        slot="trigger"
        name="download"
        color=${this.color}
      ></stencila-icon-button>

      <div
        class=${tw`relative rounded border(& ${this.color}-200) bg-${this.color}-${this.shade}`}
      >
        <stencila-icon-button
          name="clipboard"
          adjust="absolute top-1 right-1 z-50"
          @click=${(event: Event) => {
            const editor = this.renderRoot.querySelector(
              'sl-tab-panel[active] stencila-code-editor'
            ) as StencilaCodeEditor
            const text = editor.getCode()
            copy(text)

            const icon = event.target as StencilaIconButton
            icon.name = 'clipboard-check'
            setTimeout(() => {
              icon.name = 'clipboard'
            }, 5000)
          }}
        ></stencila-icon-button>

        <sl-tab-group
          @sl-tab-show=${(event: CustomEvent) => {
            const format = event.detail.name
            load(format)
            window.localStorage.setItem('StencilaEntityDownload.format', format)
          }}
        >
          ${this.formats.map((format) => {
            const [label] = StencilaEntityDownload.formatDetails[format]

            return html`<sl-tab slot="nav" panel=${format}>
                <stencila-icon
                  name="${format}-color"
                  class=${tw`text-[1.25em] mr-1`}
                ></stencila-icon>
                <span class=${tw`text-xs`}>${label}</span>
              </sl-tab>
              <sl-tab-panel name=${format} class=${tw`sm:w-[35em]`}>
                <stencila-code-editor
                  class=${tw`w-full text-sm`}
                  language=${format}
                  placeholder="Loading..."
                  read-only
                  no-controls
                >
                  <pre slot="code"></pre>
                </stencila-code-editor>
              </sl-tab-panel>`
          })}
        </sl-tab-group>
      </div>
    </sl-dropdown>`
  }
}
