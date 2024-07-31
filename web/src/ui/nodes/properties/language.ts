import { LitElement } from 'lit'

export declare class AvailableLanguagesInterface {
  protected languages: Record<
    string,
    { displayName: string; icon: [string, string] }
  >
}

// eslint-disable-next-line @typescript-eslint/no-explicit-any
type Constructor<T> = new (...args: any[]) => T

// eslint-disable-next-line @typescript-eslint/no-explicit-any
type AbstractContructor<T> = abstract new (...args: any[]) => T

/**
 * A mixin that supplies available programming languages & their icons.
 */
export const AvailableLanguagesMixin = <
  T extends Constructor<LitElement> | AbstractContructor<LitElement>,
>(
  superClass: T
) => {
  class AvailableLanguages extends superClass {
    protected languages = {
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
  }
  return AvailableLanguages as unknown as Constructor<AvailableLanguagesInterface> &
    T
}
