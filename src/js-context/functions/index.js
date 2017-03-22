import type from './types/type'

import csv from './formats/csv'

import random from './prob/random'
import random_uniform from './prob/random_uniform'

import marks from './plot/marks'
import bars from './plot/bars'
import points from './plot/points'

import titles from './plot/titles'
import theme from './plot/theme'

export default {
  'type': type,

  'csv': csv,

  'random': random,
  'random_uniform': random_uniform,

  'marks': marks,
  'bars': bars, 'barplot': bars,
  'points': points, 'pointplot': points, 'scatterplot': points,

  'titles': titles,
  'theme': theme
}
