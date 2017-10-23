import template from './template'

export default function () {
  return template({
    'A': {
      name: 'Var 1',
      type: 'string'
    },
    'B': {
      name: 'Var 1',
      type: 'string'
    }
  },{
    'A1': 'a',
    'A2': 'b',
    'A3': 'c',
    'A4': 'd',
    'A5': 'e',
    'B1': 1,
    'B2': 2,
    'B3': 3,
    'B4': 4,
    'B5': 5
  })
}
