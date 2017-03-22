import array from '../types/array'
import table from '../types/table'

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
export default function filter(value, clause) {
  let matcher = new Function('row', `
    with(row) {
      return eval('${clause}')
    }
  `)
  let rows = array(value)
  return table(rows.filter(matcher))
}
