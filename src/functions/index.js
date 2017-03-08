import type from './types/type'

import csv from './formats/csv'

import bars from './plot/bars'
import points from './plot/points'

export default {
  'type': type,

  'csv': csv,

  'bars': bars,
  'barplot': bars,
  'points': points,
  'pointplot': points,
  'scatterplot': points
}
