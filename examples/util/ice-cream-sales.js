
import template from './template'

const COLS = {
  'A': {
    name: 'temp'
  },
  'B': {
    name: 'sales'
  },
  'C': {
    name: 'sunny'
  }
}

// These columns are rearranged in the sheet, see below
const DATA = [
  [18,50,'no'],
  [20,126,'yes'],
  [24,118,'yes'],
  [23,126,'yes'],
  [26,280,'yes'],
  [25,102,'no'],
  [20,93,'no'],
  [17,32,'no'],
  [18,103,'yes'],
  [28,246,'yes'],
]

let cells = {}
for (let row = 0; row < 250; row++) {
  for (let col = 0; col < 10; col++) {
    if (DATA[row]) {
      if (DATA[row][col]) {
        // Note reordering of columns here
        cells[(['A','B','C'][col]) + (row+1)] = DATA[row][col]
      }
    }
  }
}

export default function () {
  return template(COLS, cells, 20, 250)
}
