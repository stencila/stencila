import { forEach, isNumber } from 'substance'

const ALPHABET = "ABCDEFGHIJKLMNOPQRSTUVWXYZ"

export function getColumnName(col) {
  if (!isNumber(col)) {
    throw new Error('Illegal argument.')
  }
  var name = ""
  while(col !== 0) {
    var mod = col % ALPHABET.length
    col = Math.floor(col/ALPHABET.length)
    name = ALPHABET[mod] + name
    if (col > 0) col--
    else if (col === 0) break
  }
  return name
}

export function getColumnIndex(col) {
  var index = 0
  var rank = 1
  forEach(col, function(letter) {
    index += rank * ALPHABET.indexOf(letter)
    rank++
  })
  return index
}

export function getCellId(row,col) {
  return getColumnName(col)+(row+1)
}

export function getRowCol(id) {
  var match = /^([A-Z]+)([1-9][0-9]*)$/.exec(id)
  return [
    parseInt(match[2], 10)-1,
    getColumnIndex(match[1])
  ]
}

export function kindToSymbol(kind) {
  switch(kind) {
    case 'exp': return '='
    case 'req': return '^'
    case 'man': return '|'
    case 'tes': return '?'
    case 'vis': return '~'
    case 'cil': return '_'
    default: return ''
  }
}

export function symbolToKind(symbol) {
  switch(symbol) {
    case '=': return 'exp'
    case ':': return 'map'
    case '^': return 'req'
    case '|': return 'man'
    case '?': return 'tes'
    case '~': return 'vis'
    case '_': return 'cil'
    default: return ''
  }
}
