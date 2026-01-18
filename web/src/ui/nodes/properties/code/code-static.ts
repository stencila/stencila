import { LitElement, html } from 'lit'
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
import 'prismjs/components/prism-toml'
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
 *
 * Uses Light DOM to allow text selection for site review functionality.
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
    // TOML
    toml: 'toml',
    // YAML
    yaml: 'yaml',
    yml: 'yaml',
  }

  /**
   * Use Light DOM so text selection works for site review
   */
  protected override createRenderRoot() {
    return this
  }

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
