export function getSyntaxTokens(path, tokens) {
  return tokens ? tokens.map((t) => {
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
