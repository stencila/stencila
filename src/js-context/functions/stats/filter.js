import convertTableToArray from '../types/convertTableToArray'
import convertArrayToTable from '../types/convertArrayToTable'
/**
 *
 * data = {
 *   type: 'table',
 *   data: {
 *     weight: [85, 46],
 *     sex: ['M', 'F']
 *   }
 * }
 * 
 * data | filter('weight<85 and sex==="M"')
 * 
 * @type {Function}
 */
export default function filter(table, clause) {
  let matcher = new Function('row', `
    with(row) {
      return eval('${clause}')
    }
  `)
  let rows = convertTableToArray(table)
  return convertArrayToTable(rows.filter(matcher))
}
