import template from './template'

const COLS = {
  'A': {
    name: 'journal',
    type: 'string',
    width: 300
  },
  'B': {
    name: 'excel_files',
    type: 'integer'
  },
  'C': {
    name: 'gene_lists',
    type: 'integer'
  },
  'D': {
    name: 'gene_papers',
    type: 'integer'
  },
  'E': {
    name: 'supp_files',
    type: 'integer'
  },
  'F': {
    name: 'papers_affected',
    type: 'integer'
  },
  'G': {
    name: 'genes_converted',
    type: 'integer'
  },
  'H': {
    name: 'percent_affected',
    type: 'number'
  }
}

const DATA = [
  ['PLoS One', 7783, 2202, 994, 220, 170, 4240],
  ['BMC Genomics', 11464, 1650, 801, 218, 158, 4932],
  ['Genome Res', 2607, 580, 251, 114, 68, 3180],
  ['Nucleic Acids Res', 2117, 540, 315, 88, 67, 1661],
  ['Genome Biol', 2678, 664, 257, 97, 63, 1878],
  ['Genes Dev', 932, 395, 190, 75, 55, 1593],
  ['Hum Mol Genet', 980, 372, 168, 48, 27, 1724],
  ['Nature', 482, 150, 74, 27, 23, 1375],
  ['BMC Bioinformatics', 1790, 235, 152, 26, 21, 534],
  ['RNA', 569, 127, 77, 20, 15, 1341],
  ['Nat Genet', 264, 70, 37, 12, 9, 178],
  ['Bioinformatics', 731, 112, 67, 11, 6, 339],
  ['PLoS Comput Biol', 177, 79, 32, 6, 6, 46],
  ['PLoS Biol', 143, 54, 29, 7, 5, 206],
  ['Mol Biol Evol', 995, 112, 79, 7, 4, 56],
  ['Science', 172, 36, 19, 7, 3, 451],
  ['Genome Biol Evol', 490, 32, 25, 2, 2, 121],
  ['DNA Res', 801, 57, 30, 2, 2, 6]
]

let cells = {}
for (let row = 0; row < 20; row++) {
  for (let col = 0; col < 10; col++) {
    if (DATA[row]) {
      if (DATA[row][col]) cells[(['A','B','C','D','E','F','G'][col]) + (row+1)] = DATA[row][col]
    }
  }
}

for (let row = 0; row < 19; row++) {
  cells[`H${row}`] = `= F${row}/D${row}*100`
}

cells['F20'] = "= sum(F1:F18)"
cells['D20'] = "= sum(D1:D18)"
cells['H20'] = "= F20/D20*100"
cells['I20'] = "= test_between(H20, 0, 100)"

cells['A21'] = "= plot(A1:C18)"

export default function () {
  return template(COLS, cells)
}
