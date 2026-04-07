import Prism from 'prismjs'
import 'prismjs/components/prism-markup'
import 'prismjs/components/prism-markdown'

/**
 * Prism.js grammar for Stencila Markdown (SMD)
 *
 * Extends the built-in Markdown grammar with Stencila-specific syntax:
 *
 * - Colon-fenced blocks (:::) with keywords like figure, table, for, if, etc.
 * - Exec code chunks (```lang exec)
 * - Code expressions (`code`{exec})
 * - Include/call blocks
 * - Instruction and suggestion blocks
 * - Parameters (&[name])
 */

Prism.languages.smd = Prism.languages.extend('markdown', {})

// Remove 4-space indented code blocks inherited from Markdown grammar.
// SMD does not support this syntax and it causes false highlighting.
const smdGrammar = Prism.languages.smd as Record<string, unknown>
const smdCode = smdGrammar['code']
if (Array.isArray(smdCode)) {
  // Keep only fenced code blocks (``` ... ```), remove the indented variant
  smdGrammar['code'] = smdCode.filter(
    (rule) => typeof rule === 'object' && 'greedy' in rule
  )
}

Prism.languages.insertBefore('smd', 'code', {
  // Colon-fenced blocks: ::: keyword [args]
  // Must come before generic colon fence closer
  'colon-fence': [
    {
      // ::: for <variable> in <expression>
      pattern: /^([ \t]*)(:{3,})[ \t]+(for)[ \t]+(\w+)[ \t]+(in)[ \t]+(.+)$/m,
      lookbehind: true,
      inside: {
        'colon-delimiter': /^:{3,}/,
        keyword: /\b(?:for|in)\b/,
        variable: /(?<=\bfor\s+)\w+/,
        'attr-value': /(?<=\bin\s+).+/,
      },
    },
    {
      // ::: if/elif <expression>
      pattern: /^([ \t]*)(:{3,})[ \t]+(if|elif)[ \t]+(.+)$/m,
      lookbehind: true,
      inside: {
        'colon-delimiter': /^:{3,}/,
        keyword: /\b(?:if|elif)\b/,
        'attr-value': /(?<=\b(?:if|elif)\s+).+/,
      },
    },
    {
      // ::: else
      pattern: /^([ \t]*)(:{3,})[ \t]+(else)[ \t]*$/m,
      lookbehind: true,
      inside: {
        'colon-delimiter': /^:{3,}/,
        keyword: /\belse\b/,
      },
    },
    {
      // ::: figure/table [layout] {attributes} <label>
      pattern: /^([ \t]*)(:{3,})[ \t]+(figure|table)([ \t]+(.+))?$/m,
      lookbehind: true,
      inside: {
        'colon-delimiter': /^:{3,}/,
        keyword: /\b(?:figure|table)\b/,
        layout: {
          pattern: /\[([^\]]*)\]/,
          inside: {
            punctuation: /[[\]]/,
            selector: /[^\]]+/,
          },
        },
        attributes: {
          pattern: /\{[^}\n]*\}/,
          inside: {
            punctuation: /[{}=]/,
            'attr-name': /[\w-]+(?=\s*=)/,
            string: /"(?:\\.|[^"\\])*"/,
            number: /(?<==\s*)-?(?:\d+\.\d+|\d+)/,
            boolean: /(?<==\s*)\b(?:true|false)\b/,
            'attr-value': /(?<==\s*)[^\s{}"]+/,
          },
        },
        'attr-value':
          /(?<=\b(?:figure|table)\s+)(?!\[[^\]]*\]|\{[^}\n]*\})\S+/,
      },
    },
    {
      // ::: include/call <source>
      pattern: /^([ \t]*)(:{3,})[ \t]+(include|call)[ \t]+(.+)$/m,
      lookbehind: true,
      inside: {
        'colon-delimiter': /^:{3,}/,
        keyword: /\b(?:include|call)\b/,
        string: /(?<=\b(?:include|call)\s+).+/,
      },
    },
    {
      // ::: style <selector>
      pattern: /^([ \t]*)(:{3,})[ \t]+(style)[ \t]+(.+)$/m,
      lookbehind: true,
      inside: {
        'colon-delimiter': /^:{3,}/,
        keyword: /\bstyle\b/,
        'attr-value': /(?<=\bstyle\s+).+/,
      },
    },
    {
      // ::: prompt [type] [position] [node-type] [@target] [hint]
      pattern: /^([ \t]*)(:{3,})[ \t]+(prompt)(?:[ \t]+(.+))?$/m,
      lookbehind: true,
      inside: {
        'colon-delimiter': /^:{3,}/,
        keyword:
          /\b(?:prompt|create|edit|fix|describe|discuss|previous|prev|above|next|below|heading|paragraph|para|table|figure|fig|code|chunk|cell|list|math|eqn|equation|quote|section)\b/,
        function: /@\S+/,
      },
    },
    {
      // ::: create/edit/fix/describe [mode] [position] [node-type] [@agent] [model] message [:::]
      pattern:
        /^([ \t]*)(:{3,})[ \t]+(create|edit|fix|describe|discuss)(?:[ \t]+(.+?))?(?:[ \t]*:{3,}|[ \t]*>>>)?[ \t]*$/m,
      lookbehind: true,
      inside: {
        'colon-delimiter': /:{3,}/,
        punctuation: />>>/,
        keyword:
          /\b(?:create|edit|fix|describe|discuss|always|auto|demand|need|lock|previous|prev|above|next|below|heading|paragraph|para|table|figure|fig|code|chunk|cell|list|math|eqn|equation|quote|section)\b/,
        function: /@\S+/,
        string: /\[[^\]]*\]/,
      },
    },
    {
      // ::: msg/system|user|model|group
      pattern:
        /^([ \t]*)(:{3,})[ \t]+(msg)\/(system|user|model|group)(?:[ \t]+(.+))?$/m,
      lookbehind: true,
      inside: {
        'colon-delimiter': /^:{3,}/,
        punctuation: /(?<=msg)\//,
        keyword: /\bmsg\b/,
        function: /\b(?:system|user|model|group)\b/,
        string: /\[[^\]]*\]/,
      },
    },
    {
      // ::: suggest [accept|reject] [feedback]
      pattern:
        /^([ \t]*)(:{3,})[ \t]+(suggest)(?:[ \t]+(.+?))?(?:[ \t]*>>)?[ \t]*$/m,
      lookbehind: true,
      inside: {
        'colon-delimiter': /:{3,}/,
        punctuation: />>/,
        keyword: /\b(?:suggest|accept|reject)\b/,
      },
    },
    {
      // Closing colon fence: :::
      pattern: /^([ \t]*)(:{3,})[ \t]*$/m,
      lookbehind: true,
      alias: 'colon-delimiter',
    },
  ],

  // Code chunks: ```lang exec [options]
  'code-chunk': {
    pattern:
      /^(`{3,})(\w+)?[ \t]+(exec)(?:[ \t]+(?:always|auto|demand|need|lock|main|fork|box|echo|hide))*(?:.*)\n[\s\S]*?^\1$/m,
    greedy: true,
    inside: {
      'code-block': {
        pattern: /^(`{3,}.*\n)([\s\S]+?)(?=\n`{3,}$)/m,
        lookbehind: true,
      },
      'code-language': {
        pattern: /^(`{3,})(\w+)/,
        lookbehind: true,
      },
      keyword:
        /\b(?:exec|always|auto|demand|need|lock|main|fork|box|echo|hide)\b/,
      punctuation: /`{3,}/,
    },
  },

  // Code expressions: `code`{exec}
  'code-expression': {
    pattern: /`[^`\n]+`\{exec\}/,
    greedy: true,
    inside: {
      keyword: /\bexec\b/,
      punctuation: /[`{}]/,
      'code-content': {
        pattern: /(?<=`)[^`]+(?=`)/,
      },
    },
  },

  // Images: ![alt](url "title") - including empty alt text which Prism's
  // built-in markdown grammar doesn't handle (it requires 1+ chars in [])
  image: {
    pattern: /!\[[^\]]*\]\([^\s)]+(?:[\t ]+"(?:\\.|[^"\\])*")?\)/,
    greedy: true,
    inside: {
      operator: /^!/,
      content: {
        pattern: /(^\[)[^\]]+(?=\])/,
        lookbehind: true,
      },
      url: {
        pattern: /(^\]\()[^\s)]+/,
        lookbehind: true,
      },
      string: {
        pattern: /([\t ]+)"(?:\\.|[^"\\])*"(?=\)$)/,
        lookbehind: true,
      },
      punctuation: /[[\]()]/,
    },
  },

  // Parameters: &[name]
  parameter: {
    pattern: /&\[[\w-]+\]/,
    greedy: true,
    inside: {
      punctuation: /[&[\]]/,
      variable: /[\w-]+/,
    },
  },
})

// Replace the `markdown` grammar with the extended `smd` grammar so that
// Prism's built-in `after-tokenize` hook (which only fires for language
// `markdown` or `md`) runs for SMD content.  Without this, fenced code
// blocks (e.g. ```svg … ```) are parsed structurally but never
// re-highlighted with the language-specific grammar.
Prism.languages.markdown = Prism.languages.smd
