
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
  [18,'no',50],
  [20,'yes',126],
  [24,'yes',118],
  [23,'yes',126],
  [26,'yes',280],
  [25,'no',102],
  [20,'no',93],
  [17,'no',32],
  [18,'yes',103],
  [28,'yes',246],
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
