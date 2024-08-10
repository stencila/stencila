import { html, LitElement } from 'lit'
import { customElement, property } from 'lit/decorators'

import { withTwind } from '../../../twind'
import { IconName } from '../../icons/icon'

export const programmingLanguages: Record<
  string,
  { title: string; icon: IconName }
> = {
  asciimath: {
    title: 'AsciiMath',
    icon: 'code',
  },
  bash: {
    title: 'Bash',
    icon: 'bash',
  },
  jinja: {
    title: 'Jinja',
    icon: 'braces',
  },
  json: {
    title: 'JSON',
    icon: 'json',
  },
  json5: {
    title: 'JSON5',
    icon: 'json',
  },
  javascript: {
    title: 'JavaScript',
    icon: 'javascript',
  },
  js: {
    title: 'JavaScript',
    icon: 'javascript',
  },
  latex: {
    title: 'LaTeX',
    icon: 'latex',
  },
  mathml: {
    title: 'MathML',
    icon: 'code',
  },
  node: {
    title: 'NodeJS',
    icon: 'nodejs',
  },
  nodejs: {
    title: 'NodeJS',
    icon: 'nodejs',
  },
  py: {
    title: 'Python',
    icon: 'python',
  },
  python: {
    title: 'Python',
    icon: 'python',
  },
  r: {
    title: 'R',
    icon: 'r',
  },
  rhai: {
    title: 'Rhai',
    icon: 'circle',
  },
  shell: {
    title: 'Shell',
    icon: 'bash',
  },
  sql: {
    title: 'SQL',
    icon: 'circle',
  },
  tex: {
    title: 'TeX',
    icon: 'tex',
  },
}

type ProgrammingLanguage = keyof typeof programmingLanguages

/**
 * Get a title and icon for a programming language
 */
export function getTitleIcon(
  lang?: string
): { title: string; icon: IconName } | null {
  return lang in programmingLanguages
    ? programmingLanguages[lang as ProgrammingLanguage]
    : null
}

/**
 * A component for the `programmingLanguage` of a node
 */
@customElement('stencila-ui-node-programming-language')
@withTwind()
export class UINodeProgrammingLanguage extends LitElement {
  /**
   * The programming language
   */
  @property({ attribute: 'programming-language' })
  programmingLanguage: string

  override render() {
    const { title, icon } = getTitleIcon(this.programmingLanguage) ?? {
      title: this.programmingLanguage,
      icon: 'code' as IconName,
    }

    return html`
      <div class="flex items-center gap-1">
        <stencila-ui-icon name=${icon} class="text-lg"></stencila-ui-icon
        ><span class="text-xs font-sans">${title}</span>
      </div>
    `
  }
}
