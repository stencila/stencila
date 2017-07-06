// TODO: this is a bit hacky,
// maybe we might want to be able to configure this
import { isLanguage } from '../language'

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
