// Type heirarchy

// Parent of each type
const parentTypes = {
  'any': null,

  'syntax': 'any',
  'call': 'syntax',
  'symbol': 'syntax',

  'boolean': 'any',

  'number': 'any',
  'integer': 'number',

  'string': 'any',

  'object': 'any',

  'array': 'any',
  'array[boolean]': 'array',
  'array[number]': 'array',
  'array[integer]': 'array[number]',
  'array[string]': 'array',
  'array[object]': 'array',

  'table': 'any'
}

// Children of each type
let childrenTypes = {}
for (let type of Object.keys(parentTypes)) {
  if (!childrenTypes[type]) childrenTypes[type] = []
  let base = parentTypes[type]
  if (!base) continue
  if (childrenTypes[base]) childrenTypes[base].push(type)
  else childrenTypes[base] = [type]
}

// Descendants (children, grandchildren etc) of each type
let descendantTypes = {}
for (let type of Object.keys(parentTypes)) {
  if (!descendantTypes[type]) descendantTypes[type] = []
  let parent = parentTypes[type]
  while (parent) {
    if (descendantTypes[parent]) descendantTypes[parent].push(type)
    else descendantTypes[parent] = [type]
    parent = parentTypes[parent]
  }
}

export { parentTypes, childrenTypes, descendantTypes }

export function coercedArrayType(arr) {
  let valType = arr.reduce(_mostSpecificType, undefined)
  if (valType === 'any') {
    return 'array'
  } else {
    return `array[${valType}]`
  }
}

function _mostSpecificType(type, next) {
  if (!next) return 'any'
  let nextType = next.type
  if (!type) return nextType
  if (type === nextType) {
    return type
  }
  switch(type) {
    case 'number': {
      if (nextType === 'integer') {
        return 'number'
      }
      break
    }
    case 'integer': {
      if (nextType === 'number') {
        return 'number'
      }
      break
    }
    default:
      //
  }
  return 'any'
}
