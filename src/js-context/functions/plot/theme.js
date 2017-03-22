import type from '../types/type'
import theme_vegalite from './theme_vegalite'

/**
 * A method dispatcher for themeing plots
 */
export default function theme (value, options) {
  let method = {
    'vegalite': theme_vegalite
  } [type(value)]
  if (method) return method(value, options)
  else throw Error('Unknown method for: ' + type(value))
}
