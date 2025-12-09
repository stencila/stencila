import { LitElement, html, css } from 'lit'
import { customElement, property } from 'lit/decorators.js'
import Prism from 'prismjs'

// Load Prism language grammars
// Note: Some languages extend others, so order matters
import 'prismjs/components/prism-markup' // html, xml, svg, mathml
import 'prismjs/components/prism-css'
import 'prismjs/components/prism-clike' // Required by many languages
import 'prismjs/components/prism-javascript'
import 'prismjs/components/prism-bash'
import 'prismjs/components/prism-json'
import 'prismjs/components/prism-python'
import 'prismjs/components/prism-r'
import 'prismjs/components/prism-sql'
import 'prismjs/components/prism-yaml'
import 'prismjs/components/prism-latex'
import 'prismjs/components/prism-markup-templating' // Required by Django/Jinja
import 'prismjs/components/prism-django' // Jinja2 is based on Django templates

/**
 * A lightweight component for rendering syntax-highlighted code
 * in static views using Prism.js
 *
 * This is a read-only alternative to `<stencila-ui-node-code>` that uses
 * Prism.js instead of CodeMirror for significantly smaller bundle size.
 */
@customElement('stencila-ui-node-code-static')
export class UINodeCodeStatic extends LitElement {
  /**
   * The code to be rendered
   */
  @property()
  code: string

  /**
   * The language of the code. Used to determine the syntax highlighting
   */
  @property()
  language: string

  /**
   * Map programming language names to Prism grammar names
   */
  private static languageMap: Record<string, string> = {
    // Shell
    bash: 'bash',
    sh: 'bash',
    shell: 'bash',
    // CSS
    css: 'css',
    // Cypher (not available in Prism core, fallback to plain)
    cypher: 'plain',
    kuzu: 'plain',
    // Graphviz DOT (not available in Prism core, fallback to plain)
    dot: 'plain',
    dotlang: 'plain',
    graphviz: 'plain',
    // Markup (HTML, XML, SVG, MathML)
    html: 'markup',
    xml: 'markup',
    mathml: 'markup',
    svg: 'markup',
    // JSON (and JSON-based visualization specs)
    json: 'json',
    cytoscape: 'json',
    echarts: 'json',
    plotly: 'json',
    vegalite: 'json',
    // JavaScript
    javascript: 'javascript',
    js: 'javascript',
    quickjs: 'javascript',
    nodejs: 'javascript',
    // Jinja (use Django template syntax which is similar)
    jinja: 'django',
    // LaTeX
    latex: 'latex',
    tex: 'latex',
    // Mermaid (not available in Prism core, fallback to plain)
    mermaid: 'plain',
    // Python
    python: 'python',
    py: 'python',
    docsql: 'python',
    // R
    r: 'r',
    // SQL
    sql: 'sql',
    // YAML
    yaml: 'yaml',
    yml: 'yaml',
  }

  /**
   * CSS styles that map Prism token classes to Stencila theme CSS variables.
   * This ensures consistent theming with the CodeMirror-based component.
   */
  static override styles = css`
    :host {
      display: block;
    }

    pre {
      margin: 0;
      overflow-x: auto;
      max-width: 100%;
      word-break: break-all;
      white-space: pre-wrap;
    }

    code {
      display: block;
      font-family: var(--code-font-family);
      font-size: var(--code-font-size-block);
      line-height: var(--code-line-height);
      color: var(--code-color);
      background-color: var(--code-background-block);
      border: var(--code-border-width) solid var(--code-border-color);
      border-radius: var(--code-border-radius);
      padding: var(--code-padding-block);
    }

    /* Comments */
    .token.comment,
    .token.prolog,
    .token.doctype,
    .token.cdata {
      color: var(--code-comment);
    }

    /* Punctuation */
    .token.punctuation {
      color: var(--code-punctuation);
    }

    /* Tags (HTML/XML) */
    .token.tag,
    .token.constant,
    .token.symbol,
    .token.deleted {
      color: var(--code-tag);
    }

    /* Properties */
    .token.property {
      color: var(--code-property);
    }

    /* Numbers and booleans */
    .token.boolean,
    .token.number {
      color: var(--code-number);
    }

    /* Strings */
    .token.selector,
    .token.attr-value,
    .token.string,
    .token.char,
    .token.builtin,
    .token.inserted {
      color: var(--code-string);
    }

    /* Operators */
    .token.operator,
    .token.entity,
    .token.url,
    .language-css .token.string,
    .style .token.string {
      color: var(--code-operator);
    }

    /* Keywords */
    .token.atrule,
    .token.attr-name,
    .token.keyword {
      color: var(--code-keyword);
    }

    /* Functions and classes */
    .token.function,
    .token.class-name {
      color: var(--code-function);
    }

    /* Variables */
    .token.regex,
    .token.important,
    .token.variable {
      color: var(--code-variable);
    }

    /* Namespace */
    .token.namespace {
      color: var(--code-namespace);
    }

    /* Text formatting */
    .token.important,
    .token.bold {
      font-weight: bold;
    }

    .token.italic {
      font-style: italic;
    }

    .token.entity {
      cursor: help;
    }
  `

  /**
   * Get the Prism language identifier for the current language
   */
  private getPrismLanguage(): string {
    if (!this.language) return 'plain'
    const lang = this.language.toLowerCase()
    return UINodeCodeStatic.languageMap[lang] ?? 'plain'
  }

  /**
   * Highlight the code using Prism.js
   */
  private highlightCode(): string {
    const prismLang = this.getPrismLanguage()

    // If language is plain or not available, return escaped HTML
    if (prismLang === 'plain' || !Prism.languages[prismLang]) {
      return this.escapeHtml(this.code ?? '')
    }

    return Prism.highlight(
      this.code ?? '',
      Prism.languages[prismLang],
      prismLang
    )
  }

  /**
   * Escape HTML special characters
   */
  private escapeHtml(text: string): string {
    const div = document.createElement('div')
    div.textContent = text
    return div.innerHTML
  }

  override render() {
    const highlighted = this.highlightCode()

    // Use unsafeHTML-like pattern by setting innerHTML
    // This is safe because Prism.js output only contains span tags with class attributes
    return html`
      <pre><code .innerHTML=${highlighted}></code></pre>
    `
  }
}
