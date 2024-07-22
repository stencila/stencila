import { html, LitElement } from 'lit'
import { customElement, property } from 'lit/decorators'

import { withTwind } from '../../../twind'

export const programmingLanguages = {
  asciimath: {
    displayName: 'AsciiMath',
    icon: ['code-block', 'stencila'],
  },
  bash: {
    displayName: 'Bash',
    icon: ['file-terminal', 'lucide'],
  },
  jinja: {
    displayName: 'Jinja',
    icon: ['braces', 'lucide'],
  },
  json: {
    displayName: 'JSON',
    icon: ['braces', 'default'],
  },
  json5: {
    displayName: 'JSON5',
    icon: ['braces', 'default'],
  },
  javascript: {
    displayName: 'JavaScript',
    icon: ['bxl-javascript', 'boxicons'],
  },
  js: {
    displayName: 'JavaScript',
    icon: ['bxl-javascript', 'boxicons'],
  },
  latex: {
    displayName: 'LaTeX',
    icon: ['latex', 'stencila'],
  },
  mathml: {
    displayName: 'MathML',
    icon: ['code-block', 'stencila'],
  },
  node: {
    displayName: 'NodeJS',
    icon: ['bxl-nodejs', 'boxicons'],
  },
  py: {
    displayName: 'Python',
    icon: ['bxl-python', 'boxicons'],
  },
  python: {
    displayName: 'Python',
    icon: ['bxl-python', 'boxicons'],
  },
  r: {
    displayName: 'R',
    icon: ['bx-code', 'boxicons'],
  },
  rhai: {
    displayName: 'Rhai',
    icon: ['bx-code', 'boxicons'],
  },
  shell: {
    displayName: 'Shell',
    icon: ['file-terminal', 'lucide'],
  },
  sql: {
    displayName: 'SQL',
    icon: ['bx-code', 'boxicons'],
  },
  tex: {
    displayName: 'TeX',
    icon: ['bx-code', 'boxicons'],
  },
  default: {
    displayName: 'code',
    icon: ['bx-code', 'boxicons'],
  },
}

export type ProgrammingLanguage = keyof typeof programmingLanguages

/**
 * A component for the `programmingLanguage` of a node
 */
@customElement('stencila-ui-node-programming-language')
@withTwind()
export class UINodeContentPlaceholder extends LitElement {
  /**
   * The language
   */
  @property({ attribute: 'programming-language' })
  programmingLanguage: string

  override render() {
    const {
      displayName,
      icon: [iconName, iconLibrary],
    } =
      this.programmingLanguage in programmingLanguages
        ? programmingLanguages[this.programmingLanguage as ProgrammingLanguage]
        : {
            displayName: this.programmingLanguage,
            icon: programmingLanguages['default'].icon,
          }

    return html`
      <div class="mr-4 flex items-center">
        <sl-icon
          class="text-lg"
          name=${iconName}
          library=${iconLibrary}
        ></sl-icon
        ><span class="text-xs ml-1 font-sans">${displayName}</span>
      </div>
    `
  }
}
