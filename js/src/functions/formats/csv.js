import * as d3 from 'd3'

export default function csv (content) {
  return d3.csvParse(content).map(row => {
    let converted = {}
    for (let field in row) {
      let str = row[field]
      let flt = parseFloat(str)
      converted[field] = isNaN(flt) ? str : flt
    }
    return converted
  })
}
