import type from './types/type'
import array from './types/array'
import table from './types/table'

import concat from './types/concat'
import repeat from './types/repeat'

import csv from './formats/csv'

import random from './prob/random'
import random_uniform from './prob/random_uniform'

import filter from './stats/filter'

import plot from './plot/plot'
import {points, circles, squares, ticks, bars, lines, areas} from './plot/plot_types'

import titles from './plot/titles'
import theme from './plot/theme'

export default {
  'type': type,
  'array': array,
  'table': table,

  'repeat' : repeat,
  'concat' : concat,

  'csv': csv,

  'random': random,
  'random_uniform': random_uniform,

  // Statistical aggregators etc
  'filter': filter,

  // Plot generators
  'plot': plot,
  'points': points, 'scatterplot': points,
  'circles': circles,
  'squares': squares,
  'ticks': ticks,
  'bars': bars, 'barplot': bars,
  'lines': lines, 'lineplot': lines,
  'areas': areas,

  // Plot modifiers
  'titles': titles,
  'theme': theme
}
