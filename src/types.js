// Type heirarchy

// Parent of each type
const parentTypes = {
  'any': null,

  'boolean': 'any',
  
  'number': 'any',
  'integer': 'number',

  'string': 'any',

  'object': 'any',

  'array': 'any',
  'array[boolean]': 'array',
  'array[number]': 'array',
  'array[integer]': 'array',
  'array[string]': 'array',
  'array[object]': 'array'
}

// Children of each type
let childrenTypes = {}
for (let type of Object.keys(parentTypes)) {
  let base = parentTypes[type]
  if (!base) continue
  if (childrenTypes[base]) childrenTypes[base].push(type)
  else childrenTypes[base] = [type]
}

// Descendants (children, grandchildren etc) of each type
let descendantTypes = {}
for (let type of Object.keys(parentTypes)) {
  let parent = parentTypes[type]
  while (parent) {
    if (descendantTypes[parent]) descendantTypes[parent].push(type)
    else descendantTypes[parent] = [type]
    parent = parentTypes[parent]
  }
}

export { parentTypes, childrenTypes, descendantTypes }
