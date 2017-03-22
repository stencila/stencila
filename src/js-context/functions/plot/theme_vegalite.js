/**
 * Apply a theme to a Vega-Lite specification object
 *
 * See https://vega.github.io/vega-lite/docs/config.html for themeing options.
 * At time of writing, the docs were out of date with respect to the alpha release so
 * you may need to check https://github.com/vega/vega-lite/blob/master/src/config.ts
 * 
 * @param  {object} vegalite - A `vegalite` object
 * @param  {object} options  - A Vega-Lite `config` object or the name of a standard theme (e.g. `bling`)
 * @return {object}          - A `vegalite` object with theme applied
 */
export default function theme_vegalite (vegalite, options) {
  options = options || {
    cell: {
      width: 400,
      height: 400
    },
    mark: {
      filled: true,
      color: '#000',
      opacity: 0.7
    },
    point: {
      size: 70
    }
  }
  return Object.assign(vegalite, {config: options})
}
