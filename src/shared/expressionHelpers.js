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
const INVALID_ID_CHARACTERS = "[':!$@\\s]"

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
  let re = new RegExp(REF, 'g')
  let m
  while ((m = re.exec(code))) {
    // NOTE: the array indexes used here correspond to the position of the capturing group
    // make sure to update these if you change the structure of the regular expression
    const symbol = m[0]
    // if this is given, the reference is a transclusion
    const docName = m[1] || m[2]
    const focusCell = m[4]
    const varName = m[5]
    // skip variables or single cells referenced within the same doc
    if (!docName) {
      if (varName || !focusCell) continue
    }
    const transpiledSymbol = toIdentifier(symbol)
    map[transpiledSymbol] = symbol
    // TODO: this could be optimized by storing parts and concatenate at the end
    code = code.substring(0, m.index) + transpiledSymbol + code.substring(m.index+transpiledSymbol.length)
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
  const mangledStr = toIdentifier(m[0])
  const scope = m[1] || m[2]
  const anchorCell = m[3]
  const focusCell = m[4]
  const varName = m[5]
  let type, name
  if (anchorCell) {
    if (focusCell && focusCell !== anchorCell) {
      type = 'range'
      name = anchorCell + ':' + focusCell
    } else {
      type = 'cell'
      name = anchorCell
    }
    type = focusCell ? 'range' : 'cell'
  } else if (varName) {
    type = 'var'
    name = varName
  } else {
    throw new Error('Invalid symbol expression')
  }
  return { type, scope, name, mangledStr }
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
