import type from './types/type'

import csv from './formats/csv'

import random from './prob/random'
import random_uniform from './prob/random_uniform'

import bars from './plot/bars'
import points from './plot/points'

export default {
  'type': type,

  'csv': csv,

  'random': random,
  'random_uniform': random_uniform,

  'bars': bars,
  'barplot': bars,
  'points': points,
  'pointplot': points,
  'scatterplot': points
}
