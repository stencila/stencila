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


/*
  Matchers for transclusions and cell references

  Examples:
  - A1
  - A1:B10
  - Foo!A1
  - doc1!x
  - 'My Sheet'!A1:B10
  - 'My Doc'.x
*/
const ID = "([_A-Za-z][_A-Za-z0-9]*)"
const NAME = "[']([^']+)[']"
const CELL_ID = "([A-Z]+[1-9][0-9]*)"
// These characters will be replaced. Add more if needed.
const INVALID_ID_CHARACTERS = "[^A-Za-z0-9]"

/*
  A reference can point to a variable, a cell, or a range inside the same document
  or another one. To avoid matches inside of other symbols, '\b' (word boundary) is used in the expression.
  `[']` can not be used in combination with '\b'.

  ```
   ( ( \b ID | ['].+['] )[!] | \b)( CELL_ID([:]CELL_ID)? | ID )
  ```
*/
const REF = "(?:(?:(?:(?:\\b"+ID+"|"+NAME+"))[!])|\\b)(?:"+CELL_ID+"(?:[:]"+CELL_ID+")?|"+ID+")"
const REF_RE = new RegExp(REF)
/*
  Transpiles a piece of source code so that it does not contain
  Transclusion expressions anymore, which are usually not valid in common languages.

  @param {string} code
  @param {object} map storage for transpiled symbols so that they can be mapped back later on
  @result
*/
export function transpile(code, map = {}) {
  if (!code) return code
  let re = new RegExp(REF, 'g')
  let symbols = []
  let m
  while ((m = re.exec(code))) {
    symbols.push(_createSymbol(m))
  }
  // Note: we are transpiling without changing the length of the original source
  // i.e. `'My Sheet'!A1:B10` is transpiled into `_My_Sheet__A1_B10`
  // thus the symbol locations won't get invalid by this step
  for (let i = 0; i < symbols.length; i++) {
    const s = symbols[i]
    code = code.substring(0, s.startPos) + s.mangledStr + code.slice(s.endPos)
    map[s.mangledStr] = s
  }
  return code
}

/*

  - `type`: `variable | cell | range`
  - `id`: a qualified id such as `doc1!x`, `sheet1!A1`, `sheet1!A1:A10`
  - `mangledStr`: not longer than the orignal which is used for transpiledCode
  - `scope`: e.g `doc1`, `sheet1`, `'My Document'`"
  - `symbol`: local symbol id such as `x`, `A1`, `A1:A10`
*/
export function parseSymbol(str) {
  let m = REF_RE.exec(str)
  if (!m) throw new Error('Unrecognised symbol format.')
  return _createSymbol(m)
}

/*
  Replaces all characters that are invalid in a variable identifier.

  Note: replacing characters one-by-one retains the original length or the string
  which is desired as this does avoid source-mapping. E.g. when a runtime error
  occurs, the error location can be applied to the original source code without
  any transformation.
*/
export function toIdentifier(str, c = '_') {
  return str.replace(new RegExp(INVALID_ID_CHARACTERS,'g'), c)
}

function _createSymbol(m) {
  const text = m[0]
  const startPos = m.index
  const endPos = text.length + startPos
  const mangledStr = toIdentifier(m[0])
  const scope = m[1] || m[2]
  const anchor = m[3]
  const focus = m[4]
  const varName = m[5]
  let type, name
  if (anchor) {
    if (focus && focus !== anchor) {
      type = 'range'
      name = anchor + ':' + focus
    } else {
      type = 'cell'
      name = anchor
    }
  } else if (varName) {
    type = 'var'
    name = varName
  } else {
    throw new Error('Invalid symbol expression')
  }
  return { type, text, scope, name, mangledStr, startPos, endPos, anchor, focus }
}
