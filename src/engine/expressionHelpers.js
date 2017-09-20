// TODO: this is a bit hacky,
// maybe we might want to be able to configure this
import { isLanguage } from '../language'

// TODO: also make sure that call()/run() only have arguments with name (var, or named arg)
export function validateExpression(expr) {
  let context = getContextName(expr)
  if (context) {
    if (expr.isDefinition()) {
      if (expr.root.rhs.type !== 'call') {
        // HACK: imitating a syntax error
        expr.syntaxError = {
          msg: `Try something like: ${expr.name} = ${context}(...)`
        }
      }
    } else if (expr.root.type !== 'call') {
      expr.syntaxError = {
        msg: `Try something like: ${context}(...)`
      }
    }
  }
}

export function getContextName(expr) {
  const nodes = expr.nodes
  for (var i = 0; i < nodes.length; i++) {
    let node = nodes[i]
    if (node.type === 'call' && isLanguage(node.name)) {
      return node.name
    }
  }
}

export function getSyntaxTokens(path, expr) {
  return expr.tokens ? expr.tokens.map((t) => {
    return {
      type: 'code-highlight',
      name: _getTokenType(t),
      start: { path, offset: t.start },
      end: { path, offset: t.end },
      on() {},
      off() {}
    }
  }) : []
}

function _getTokenType(t) {
  switch(t.type) {
    case 'function-name': {
      if (isLanguage(t.text)) {
        return 'external-language'
      } else {
        return t.type
      }
    }
    default:
      return t.type
  }
}
