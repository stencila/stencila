import Prism from '../../tmp/prism.js'
import { extractSymbols } from './expressionHelpers'

const CELL = /\b([a-z0-9_]+[!])?([A-Z]{1,3}[1-9][0-9]*)(?:[:]([A-Z]{1,3}[1-9][0-9]*))?\b/
const DEF = /(^|\n)[a-zA-Z_$][a-zA-Z_$0-9]*(?=\s*[=])/
const KEY = /\b[a-zA-Z_$][a-zA-Z_$0-9]*(?=\s*[=:])/
const ID = /\b[a-zA-Z_$][a-zA-Z_$0-9]*\b/

let languages = {}

languages['mini'] = {
  // taken from Prism.languages.clike.string
  'string': {
    pattern: /(["])(?:\\(?:\r\n|[\s\S])|(?!\1)[^\\\r\n])*\1/,
    greedy: true
  },
  'boolean': /\b(?:true|false)\b/,
  'number': /\b-?(?:0x[\da-f]+|\d*\.?\d+(?:e[+-]?\d+)?)\b/i,
  'function': /[a-z0-9_]+(?=\()/i,
  'lparen': /[(]/,
  'rparen': /[)]/,
  'comma': /[,]/,
  'cell': CELL,
  'def': { pattern: DEF, greedy: true },
  'key': { pattern: KEY, greedy: true },
  'id': { pattern: ID, greedy: true }
}

Prism.languages.insertBefore('r', 'punctuation', {
  'function': /[a-z0-9_]+(?=\()/i,
  'lparen': /[(]/,
  'rparen': /[)]/,
  'comma': /[,]/,
  'cell': CELL,
  'def': { pattern: DEF, greedy: true },
  'key': { pattern: KEY, greedy: true },
  'id': { pattern: ID, greedy: true }
})
languages['r'] = Prism.languages.r

Prism.languages.insertBefore('python', 'punctuation', {
  'function': /[a-z0-9_]+(?=\()/i,
  'lparen': /[(]/,
  'rparen': /[)]/,
  'comma': /[,]/,
  'cell': CELL,
  'def': { pattern: DEF, greedy: true },
  'key': { pattern: KEY, greedy: true },
  'id': { pattern: ID, greedy: true }
})
languages['python'] = languages['py'] = languages['pyjp'] = Prism.languages.python

Prism.languages.insertBefore('javascript', 'punctuation', {
  'function': /[a-z0-9_]+(?=\()/i,
  'lparen': /[(]/,
  'rparen': /[)]/,
  'comma': /[,]/,
  'cell': CELL,
  'def': { pattern: DEF, greedy: true },
  'key': { pattern: KEY, greedy: true },
  'id': { pattern: ID, greedy: true }
})
languages['js'] = languages['node'] = languages['javascript'] = Prism.languages.javascript

Prism.languages.insertBefore('sql', 'punctuation', {
  'function': /[a-z0-9_]+(?=\()/i,
  'lparen': /[(]/,
  'rparen': /[)]/,
  'comma': /[,]/,
  'cell': CELL,
  'def': { pattern: DEF, greedy: true },
  'key': { pattern: KEY, greedy: true },
  'id': { pattern: ID, greedy: true }
})
languages['sql'] = Prism.languages.sql

function tokenize (code, lang) {
  let grammar = languages[lang]
  if (!grammar) {
    console.error(`No tokenizer registered for language ${lang}`)
    return []
  }
  let prismTokens = Prism.tokenize(code, grammar)
  let tokens = []
  let pos = 0
  for (let i = 0; i < prismTokens.length; i++) {
    let t = prismTokens[i]
    let start = pos
    let end = pos + t.length
    switch (typeof t) {
      case 'array':
      case 'string': {
        break
      }
      default:
        tokens.push({
          type: t.type,
          text: t.content,
          start, end
        })
    }
    pos = end
  }
  return tokens
}

// pseudo-parsing to collect information about functions
export default function analyzeCode(code, lang = 'mini') {
  let tokens = tokenize(code, lang)
  let symbols = extractSymbols(code)
  let nodes = []
  let calls = []

  function _push(end) {
    let currentCall = calls[0]
    if (currentCall) {
      // tidy up
      delete currentCall.pos
      delete currentCall.inArgs
      currentCall.end = end
      nodes.push(currentCall)
      calls.shift()
    }
  }

  for (let i = 0; i < tokens.length; i++) {
    const currentCall = calls[0]
    const t = tokens[i]
    switch (t.type) {
      case 'function': {
        let call = {
          type: 'function',
          name: t.text,
          start: t.start,
          // we want the end position of the closing paren here
          end: -1,
          args: [],
          // state used for extracting location of args
          pos: t.end,
          inArgs: false
        }
        calls.unshift(call)
        break
      }
      case 'lparen': {
        if (currentCall && !currentCall.inArgs) {
          currentCall.inArgs = true
          currentCall.pos = t.end
          tokens.splice(i--, 1)
        }
        break
      }
      case 'comma': {
        if (currentCall) {
          currentCall.args.push({
            start: currentCall.pos,
            end: t.start
          })
          currentCall.pos = t.end
          tokens.splice(i--, 1)
        }
        break
      }
      case 'rparen': {
        if (currentCall) {
          if (t.start > currentCall.pos) {
            currentCall.args.push({
              start: currentCall.pos,
              end: t.start
            })
            currentCall.pos = t.end
          }
        }
        _push(t.end)
        tokens.splice(i--, 1)
        break
      }
      default:
        //
    }
  }
  // also push incomplete function calls
  _push(code.length)

  return { tokens, symbols, nodes }
}
