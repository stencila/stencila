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
  return t.type
}
