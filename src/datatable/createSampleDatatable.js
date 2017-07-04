export default function createSampleDatatable() {
  const L = 1000
  const MAX = 100
  const columnNames = ['a','b','c','d','e']
  let data = {}
  columnNames.forEach((name) => {
    let values = []
    for (let i = 0; i < L; i++) {
      values.push(Math.floor(Math.random()*MAX*10)/10)
    }
    let column = {
      name,
      type: 'float',
      measure: 'quant',
      values
    }
    data[name] = column
  })
  return data
}